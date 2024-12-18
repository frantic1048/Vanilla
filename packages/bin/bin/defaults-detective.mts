#!/usr/bin/env -S deno run --allow-env --allow-read --allow-run

import chalk from "npm:chalk@5.3.0";
import { parse } from "npm:fast-plist@0.1.3";
import { $ } from "npm:execa@9.5.2";
// @ts-types="npm:@types/deep-diff@1.0.5"
import diff from "npm:deep-diff@1.0.2";
import process from "node:process";

const log = process.stdout.write.bind(process.stdout);
const logError = process.stderr.write.bind(process.stderr);

const styleBlueHeading = chalk.rgb(250, 250, 250).bgRgb(37, 136, 230);
const styleGreen = chalk.rgb(125, 217, 26);
const styleRed = chalk.rgb(232, 46, 90);
/**
 * Domains(paths) that are not useful for system configuration
 */
const maskedDomains: (string | RegExp | [domain: string, path: string])[] = [
  /^\d+$/,
  "com.apple.biomesyncd",
  "com.apple.CallHistorySyncHelper",
  "com.apple.cseventlistener",
  "com.apple.DuetExpertCenter.AppPredictionExpert",
  "com.apple.FaceTime",
  "com.apple.HIToolbox",
  ["com.apple.iChat", "LastIMDNotificationPostedDate"],
  "com.apple.knowledge-agent",
  "com.apple.madrid",
  "com.apple.mmcs",
  "com.apple.photolibraryd",
  "com.apple.photos.shareddefaults",
  "com.apple.proactive.PersonalizationPortrait",
  "com.apple.routined",
  "com.apple.spaces",
  "com.apple.spotlightknowledge",
  "com.apple.tipsd",
  "com.apple.xpc.activity2",
  "ContextStoreAgent",
];

log(chalk.bold("=> Discovering domains...\n"));
const allDomains = [];
try {
  allDomains.push(...(await $`defaults domains`).stdout.split(", "));
} catch (error) {
  logError(`${String(error)}\n`);
}

const readableDomains: string[] = [];
await Promise.all(
  allDomains.map(async (domain) => {
    if (
      maskedDomains.some((m) => {
        if (typeof m === "string") {
          return m === domain;
        } else if (m instanceof RegExp) {
          return m.test(domain);
        } else {
          return false;
        }
      })
    ) {
      log(`[ignore] ${domain}\n`);
      return;
    }

    try {
      await $`defaults read ${domain}`;
      readableDomains.push(domain);
      log(`${styleGreen("[ok]")} ${domain}\n`);
    } catch {
      // defaults read will throw error if the domain is not readable
    }
  }),
);

const domainPlists: Record<
  string,
  {
    new: string;
    old: string;
  }
> = {};

log(chalk.bold("=> Loading domains...\n"));
await Promise.all(
  readableDomains.map(async (domain) => {
    const plist = parse((await $`defaults export ${domain} -`).stdout);
    domainPlists[domain] = {
      new: plist,
      old: plist,
    };
    log(`${styleGreen("[ok]")} ${domain}\n`);
  }),
);

log(chalk.bold("=> Start listening to defaults changes...\n"));
while (true) {
  await Promise.all(
    readableDomains.map(async (domain) => {
      const plist = parse((await $`defaults export ${domain} -`).stdout);
      domainPlists[domain].old = domainPlists[domain].new;
      domainPlists[domain].new = plist;

      const diffResult = diff(
        domainPlists[domain].old,
        domainPlists[domain].new,
      );
      if (diffResult) {
        log(
          diffResult
            .map(
              ({
                kind,
                path,
                lhs,
                rhs,
              }: {
                kind: string;
                path: string[];
                lhs: string;
                rhs: string;
              }) => {
                const prefix = `\n${styleBlueHeading(` ${domain} `)} `;
                const pathString = path.join(".");
                if (
                  maskedDomains.some((m) =>
                    Array.isArray(m) && m[0] === domain && m[1] === pathString
                  )
                ) {
                  return;
                } else if (kind === "N") {
                  return `${prefix}${styleGreen(pathString)}\n ++ ${
                    styleGreen(rhs)
                  }`;
                } else if (kind === "D") {
                  return `${prefix}${styleRed(pathString)}\n -- ${
                    styleRed(lhs)
                  }`;
                } else if (kind === "E") {
                  return `${prefix}${chalk.bold(pathString)}\n -- ${
                    styleRed(lhs)
                  }\n ++ ${styleGreen(rhs)}`;
                }
              },
            )
            .join("\n"),
        );
        log("\n");
      }
    }),
  );

  // rate limit
  await new Promise((resolve) => setTimeout(resolve, 1000));
}
