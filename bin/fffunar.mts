#!/usr/bin/env -S deno run --allow-env --allow-read --allow-run

/// <reference types="npm:@types/node" />

import process from "node:process";
import $ from "jsr:@david/dax@0.44.1";
import assert from "node:assert/strict";

const printUsage = () => {
  const usageText = `Extract archive files with optional password in filename
Usage:
  fffunar <archive_file1> [<archive_file2> ...]
  Examples:
  fffunar abc.zip
  fffunar abc[=password].zip
`;
  console.log(usageText);
};

// archive name:
//   - abc.zip
// with password(inside the [=...]):
//   - abc[=password].zip

assert.ok(
  await $.commandExists("unar"),
  "'unar' command not found. Please install unar to use this tool."
);

const main = async () => {
  const args = process.argv.slice(2);
  if (args.length < 1) {
    printUsage();
  }

  const archives: Array<{
    filePath: string;
    outputPath: string;
    password?: string;
  }> = args.map((arg) => {
    const passwordMatch = arg.match(/\[=(.*?)]/);
    // strip extension and password part for output path
    const outputPath = arg
      .replace(/\[=.*?]/, "")
      .replace(/\.[^/.]+$/, "")
      .trim();

    return {
      filePath: arg,
      password: passwordMatch ? passwordMatch[1] : undefined,
      outputPath,
    };
  });

  const progress = $.progress("Extracting archives", {
    length: archives.length,
    noClear: true,
  });

  await progress.with(async () => {
    for (const archive of archives) {
      const { filePath, password } = archive;
      const passwordOption = password ? ["-p", password] : [];
      const outputOption = ["-o", archive.outputPath];

      progress.message(filePath);
      await $`unar -q -s ${passwordOption} ${outputOption} ${filePath}`;
      progress.increment();
    }
  });
};

await main();
