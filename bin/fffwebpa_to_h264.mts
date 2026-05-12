#!/usr/bin/env -S deno run --allow-env --allow-read --allow-run

/// <reference types="npm:@types/node" />

import process from "node:process";
import $ from "jsr:@david/dax@0.44.1";
import assert from "node:assert/strict";

const printUsage = () => {
  const usageText = `Convert WebP animations to H.264 MP4
Usage:
  fffwebpa_to_h264 <webp_animation1> [<webp_animation2> ...]
  Examples:
  fffwebpa_to_h264 abc.webp # produces abc.webp.mp4
`;
  console.log(usageText);
};

assert.ok(
  await $.commandExists("ffmpeg"),
  "'ffmpeg' command not found. Please install ffmpeg to use this tool.",
);

const main = async () => {
  const args = process.argv.slice(2);
  if (args.length < 1) {
    printUsage();
  }

  const tasks: Array<
    {
      inputPath: string;
      outputPath: string;
    }
  > = args.map((arg) => {
    const inputPath = arg;
    const outputPath = `${arg}.mp4`;

    return {
      inputPath,
      outputPath,
    };
  });

  const progress = $.progress("Converting inputs", {
    length: tasks.length,
    noClear: true,
  });

  await progress.with(async () => {
    for (const task of tasks) {
      const { inputPath, outputPath } = task;

      /**
 *
 * ffmpeg -i input.webp \
    -filter_complex "[0:v]scale=trunc(iw/2)*2:trunc(ih/2)*2[v];color=white[bg];[bg][v]scale2ref[bg][v];[bg][v]overlay=format=auto:shortest=1,setsar=1" \
    -c:v libx264 -preset slow -crf 20 -pix_fmt yuv420p -movflags +faststart output.mp4
 */

      const ffmpegOptions = [
        "-i",
        inputPath,
        "-filter_complex",
        "[0:v]scale=trunc(iw/2)*2:trunc(ih/2)*2[v];color=white[bg];[bg][v]scale2ref[bg][v];[bg][v]overlay=format=auto:shortest=1,setsar=1",
        "-c:v",
        "libx264",
        "-preset",
        "slow",
        "-crf",
        "20",
        "-pix_fmt",
        "yuv420p",
        "-tune",
        "animation",
        "-movflags",
        "+faststart",
        outputPath,
      ];

      progress.message(inputPath);

      await $`ffmpeg ${ffmpegOptions}`
        .printCommand();
      progress.increment();
    }
  });
};

await main();
