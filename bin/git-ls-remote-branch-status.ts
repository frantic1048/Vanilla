#!/usr/bin/env deno run --allow-env --allow-read --allow-run

import { $ } from "npm:execa";
import _ from "npm:lodash";
import chalk from "npm:chalk";
import { parseArgs } from "node:util";
import { argv } from "node:process";

const usage = `
Usage:
  git ls-remote-branch-status [options]

Options:
  -h, --help         Show help
  -u, --user <user>  Specify the branch author, default is current git user, can match part of the name or email
  -a, --allUser      Show all users' branches, this option will ignore -u option
`;

const args = parseArgs({
  args: argv.slice(2),
  options: {
    help: {
      type: "boolean",
      short: "h",
    },
    /**
     * MEMO:
     * Match against the canonical user name?
     * https://git-scm.com/docs/git-check-mailmap
     */
    user: {
      type: "string",
      short: "u",
      default: (await $`git config user.name`).stdout.trim(),
    },
    allUser: {
      short: "a",
      type: "boolean",
    },
  },
});

if (args.values.help) {
  console.log(usage);
} else {
  await lsRemoteBranches();
}

async function lsRemoteBranches() {
  const remoteName = "origin";
  const mainBranch = "develop";
  const ignoredBranches = ["develop", "master", "HEAD"];

  const remoteMainBranch = `${remoteName}/${mainBranch}`;

  await $("git", ["fetch", "--prune", remoteName]);

  const rawRefInfo = (
    await $("git", [
      "for-each-ref",
      ...ignoredBranches.map(
        (branch) => `--exclude=refs/remotes/${remoteName}/${branch}`
      ),
      `--sort=ahead-behind:${remoteMainBranch}`,
      "--sort=author",
      // lstrip=3: refs/remotes/origin/my-branch -> my-branch
      `--format=%(ahead-behind:${remoteMainBranch}),%(refname:lstrip=3),%(authorname)%(authoremail)`,
      `refs/remotes/${remoteName}`,
    ])
  ).stdout;

  interface IRefInfo {
    aheadBehind: {
      ahead: number;
      behind: number;
    };
    refName: string;
    author: string;
  }
  const refInfos: IRefInfo[] = rawRefInfo.split("\n").map((line: string) => {
    const [rawAheadBehind, refName, author] = line.split(",", 3);
    const aheadBehind = rawAheadBehind.split(" ").map(Number);
    return {
      aheadBehind: {
        ahead: aheadBehind[0],
        behind: aheadBehind[1],
      },
      refName,
      author,
    };
  });

  const refInfosByAuthor: Record<string, IRefInfo[]> = _.groupBy(
    refInfos,
    "author"
  );

  for (const [author, refInfos] of Object.entries(refInfosByAuthor)) {
    if (!args.values.allUser && !author.includes(args.values.user)) {
      continue;
    }
    console.log(chalk.bold.bgBlue.white(author));
    const sortedRefInfos = refInfos.sort((a, b) => {
      // ahead asc, behind desc
      if (a.aheadBehind.ahead === b.aheadBehind.ahead) {
        return b.aheadBehind.behind - a.aheadBehind.behind;
      }
      return a.aheadBehind.ahead - b.aheadBehind.ahead;
    });
    for (const refInfo of sortedRefInfos) {
      const { behind, ahead } = refInfo.aheadBehind;
      console.log(
        [
          (behind > 0 ? chalk.redBright.bold : chalk.red.dim)(
            `${(String(behind) + "⇣").padStart(5)}`
          ),
          (ahead > 0 ? chalk.greenBright.bold : chalk.green.dim)(
            `${(String(ahead) + "⇡").padEnd(5)}`
          ),
          ahead > 0
            ? refInfo.refName
            : chalk.yellow.bold.italic(refInfo.refName),
        ].join(" ")
      );
    }
  }
}
