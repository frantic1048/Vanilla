#!/usr/bin/env python3
#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""
zipfix — 解压那些因文件名过长 / 特殊字符 / 编码混乱而无法用 ark/unar 解开的 zip。

思路：不依赖系统解压器，直接用 Python 的 zipfile 读取每个 entry 的原始名字，
对名字做变换（剥前缀 / 字符串替换 / 重新解码 / 截断），再写到磁盘或导出成新 zip。

用法示例
--------
# 1) 先看看里面有啥（不解压），顺便预览变换后的名字
uv run zipfix.py a.zip --list

# 2) 剥掉一个公共长前缀后解压到 out/
uv run zipfix.py a.zip -o out --strip-prefix "some/really/long/prefix/"

# 3) 简单字符串替换（可多次），把非法字符换掉
uv run zipfix.py a.zip -o out --replace "：" "_" --replace "  " " "

# 4) 文件名是 GBK 乱码（zip 来自 Windows 中文环境），重新按 gbk 解码
uv run zipfix.py a.zip -o out --from-encoding gbk

# 5) 名字仍然太长？自动把每段路径截断到 200 字节
uv run zipfix.py a.zip -o out --max-len 200

# 6) 不想解压，只想生成一个名字干净的新 zip
uv run zipfix.py a.zip --to-zip clean.zip --strip-prefix "junk/"

先 --list 预览（默认 dry-run），确认无误再加 -o / --to-zip 真正执行。
"""

from __future__ import annotations

import argparse
import os
import sys
import unicodedata
import zipfile


def redecode(name: str, from_encoding: str | None) -> str:
    """zipfile 默认对非 UTF-8 flag 的 entry 用 cp437 解码，常导致中文乱码。
    这里把它按 cp437 编码回字节，再用指定编码重新解码。"""
    if not from_encoding:
        return name
    try:
        return name.encode("cp437").decode(from_encoding)
    except (UnicodeEncodeError, UnicodeDecodeError):
        return name  # 已经是正确 unicode（UTF-8 flag），原样返回


def transform(name: str, args: argparse.Namespace) -> str:
    new = redecode(name, args.from_encoding)

    if args.strip_prefix and new.startswith(args.strip_prefix):
        new = new[len(args.strip_prefix):]

    for old, repl in args.replace:
        new = new.replace(old, repl)

    if args.ascii_only:
        new = (
            unicodedata.normalize("NFKD", new)
            .encode("ascii", "ignore")
            .decode("ascii")
        )

    if args.sanitize:
        bad = '<>:"\\|?*' + "".join(chr(c) for c in range(32))
        new = new.replace("/", "\0")  # 暂存路径分隔符
        new = "".join("_" if ch in bad else ch for ch in new)
        new = new.replace("\0", "/")

    if args.max_len:
        new = "/".join(_truncate_component(c, args.max_len) for c in new.split("/"))

    return new.lstrip("/")


def _truncate_component(comp: str, max_bytes: int) -> str:
    """按 UTF-8 字节长度截断单个路径组件，尽量保留扩展名。"""
    if len(comp.encode("utf-8")) <= max_bytes:
        return comp
    root, dot, ext = comp.rpartition(".")
    ext = (dot + ext) if dot else ""
    budget = max_bytes - len(ext.encode("utf-8"))
    out = []
    used = 0
    for ch in (root or comp):
        b = len(ch.encode("utf-8"))
        if used + b > budget:
            break
        out.append(ch)
        used += b
    return "".join(out) + ext


def safe_join(base: str, rel: str) -> str:
    """防止 zip-slip（../ 逃逸）。"""
    dest = os.path.normpath(os.path.join(base, rel))
    if not (dest == base or dest.startswith(base + os.sep)):
        raise ValueError(f"路径逃逸被拦截: {rel!r}")
    return dest


def main() -> int:
    p = argparse.ArgumentParser(
        description="重命名 zip 内文件名后解压 / 导出（应对超长名、特殊字符、乱码）。"
    )
    p.add_argument("zip", help="源 zip 文件")
    p.add_argument("-o", "--out", help="解压目标目录")
    p.add_argument("--to-zip", help="不解压，导出成名字干净的新 zip")
    p.add_argument("--list", action="store_true", help="只列出 原名 -> 新名 预览")
    p.add_argument("--strip-prefix", default="", help="从每个名字开头剥掉的前缀")
    p.add_argument(
        "--replace", nargs=2, action="append", default=[],
        metavar=("OLD", "NEW"), help="字符串替换，可多次指定",
    )
    p.add_argument("--from-encoding", help="按此编码重新解码乱码文件名，如 gbk / shift_jis")
    p.add_argument("--max-len", type=int, default=0, help="每段路径名最大字节数，超出则截断")
    p.add_argument("--sanitize", action="store_true", help="把文件系统非法字符替换成 _")
    p.add_argument("--ascii-only", action="store_true", help="把名字转成纯 ASCII（丢弃非 ASCII）")
    args = p.parse_args()

    if not zipfile.is_zipfile(args.zip):
        print(f"错误: {args.zip} 不是有效的 zip 文件", file=sys.stderr)
        return 1

    with zipfile.ZipFile(args.zip) as zf:
        infos = zf.infolist()
        mapping = [(i, transform(i.filename, args)) for i in infos]

        # 预览模式（无输出目标时默认走这里）
        if args.list or (not args.out and not args.to_zip):
            for info, new in mapping:
                flag = "" if new else "  [! 变换后为空，将跳过]"
                print(f"{info.filename!r}\n  -> {new!r}{flag}")
            print(f"\n共 {len(mapping)} 个条目。加 -o DIR 解压，或 --to-zip FILE 导出。")
            return 0

        # 重名检测
        seen: dict[str, str] = {}
        for info, new in mapping:
            if new and new in seen and not new.endswith("/"):
                print(f"警告: 变换后重名 {new!r} (来自 {info.filename!r} 和 {seen[new]!r})",
                      file=sys.stderr)
            seen[new] = info.filename

        if args.to_zip:
            with zipfile.ZipFile(args.to_zip, "w", zipfile.ZIP_DEFLATED) as out:
                for info, new in mapping:
                    if not new:
                        continue
                    out.writestr(new, zf.read(info))
            print(f"已导出 {args.to_zip}")
            return 0

        base = os.path.abspath(args.out)
        os.makedirs(base, exist_ok=True)
        count = 0
        for info, new in mapping:
            if not new:
                print(f"跳过(变换后为空): {info.filename!r}", file=sys.stderr)
                continue
            dest = safe_join(base, new)
            if info.is_dir() or new.endswith("/"):
                os.makedirs(dest, exist_ok=True)
                continue
            os.makedirs(os.path.dirname(dest) or base, exist_ok=True)
            with zf.open(info) as src, open(dest, "wb") as dst:
                while chunk := src.read(1 << 16):
                    dst.write(chunk)
            count += 1
        print(f"已解压 {count} 个文件到 {base}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
