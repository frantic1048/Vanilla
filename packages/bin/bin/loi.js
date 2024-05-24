#!/usr/bin/env node

/**
 * loi.js
 *
 * Extract video frames with subtitle according to .ass subtitle file.
 *
 * usage:
 *   loi.js VIDEO_FILE SUBTITLE_FILE
 *
 * requirement:
 *   - Node.js
 *   - ffmpeg executable in PATH
 *
 */

const read = (...argv) =>
  require('fs').readFileSync(...argv).toString()

const execffmpeg = (args) =>
  require('child_process').spawnSync('ffmpeg', args, {stdio: [0, 0, 0]})

const log = (i) => console.log(i)

const usage = `
Extract video frames with subtitle according to .ass subtitle file.

Usage: loi.js VIDEO_FILE SUBTITLE_FILE
`

let videoPath
let subtitlePath

if (process.argv.length !== 4) {
  log('unsatisfied number of input files, exiting...')
  log(process.argv)
  log(usage)
} else {
  videoPath = process.argv[2]
  subtitlePath = process.argv[3]
}

/**
 * UNUSED ...
 * escape input string for ffmpeg
 * @see man ffmpeg-utils
 * @see https://ffmpeg.org/ffmpeg-utils.html#Quoting-and-escaping
 * @param  {[type]} s [description]
 * @return {[type]}   [description]
 */
function ffescape (s) {
  const specialChar = /'|\\| /g
  return s.replace(specialChar, '\\$&')
}

/**
 * convert 00:00:00.000 to 00.000 format
 * @param  {string} time 00:00:00.000 format time
 * @return {string}      00.000 format time
 */
function normalizeTimestamp (time) {
  const [h, m, s] = time
    .split(':')
    .map(parseFloat)

  // add 0.2s shift
  // to ensure fade-in subtitle fully appeared
  const shift = 0.2

  const resultTime = h * 60 * 60 + m * 60 + s + shift
  return `${resultTime.toFixed(2)}`
}

function extractASSDialogues (subtitle) {
  const filter = /^Dialogue/
  const newline = /\r\n|\r|\n/
  const extra = /{.*?}/g
  const dialogues = subtitle
    .split(newline)
    .filter(v => filter.test(v))
    .map(v => {
      const s = v.replace(extra, '').split(',')
      // s[1] is start time
      // s[2] is end time
      return [s[1], s[9]]
    })
    .filter(v => v[0] !== '0:00:00.00')
    .map(v => [normalizeTimestamp(v[0]), v[1]])

  return dialogues
}

function captureDialogues (dialogues) {
  for (const d of dialogues) {
    log(`${d[0]} ${d[1]}`)
    const args = [
      '-loglevel', 'warning',
      '-y',
      '-ss', `${d[0]}`,
      '-i', `${videoPath}`,
      '-vf',

      /**
       * workaround for rendering correct subtitle
       * when using input seeking
       * https://trac.ffmpeg.org/ticket/2067#comment:15
       */
      `setpts=PTS+${d[0]}/TB,subtitles=${subtitlePath},setpts=PTS-STARTPTS`,
      '-vframes', '1',
      `${d[1]}.png`
    ]
    execffmpeg(args)
  }
}

const dialogues = extractASSDialogues(read(subtitlePath))
captureDialogues(dialogues)
