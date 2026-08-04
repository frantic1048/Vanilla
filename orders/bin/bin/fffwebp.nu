#!/usr/bin/env nu

const auto_resize_target_views = [
  {name: "5K DCI-like (~17:9)", width: 5120, height: 2700}
  {name: "5K UHD (16:9)", width: 5120, height: 2880}
  {name: "5K (16:10)", width: 5120, height: 3200}
]

# This generalizes the previous fixed 6144x6144 cap, whose rationale was:
#
# 6144x6144 make sure whether portrait or landscape,
# it will be large enough to fill a typical
# 4k screen (3840x2160 or 3840x2400) without scaling up.
#
# 6144:3840 = 16:10
# 3840:2400 = 16:10
# 3840:2160 = 16:9
#
# For rotation-safe cover, the image's short edge must be at least the longest
# target-view edge. `^` resizes by short edge; `>` prevents upscaling.
def auto-resize-geometry []: nothing -> string {
  let required_short_edge = (
    $auto_resize_target_views
    | each {|view| [$view.width $view.height] | math max }
    | math max
  )
  $"($required_short_edge)x($required_short_edge)^>"
}

def format-seconds [seconds: number]: nothing -> string {
  if $seconds >= 3600 {
    let hours = (($seconds / 3600) | math floor | into int)
    let minutes = ((($seconds mod 3600) / 60) | math floor | into int)
    $"($hours)h ($minutes)m"
  } else if $seconds >= 60 {
    let minutes = (($seconds / 60) | math floor | into int)
    let remaining_seconds = ($seconds mod 60)
    $"($minutes)m ($remaining_seconds | into string --decimals 0)s"
  } else {
    $"($seconds | into string --decimals 1)s"
  }
}

# Convert one or more images to WebP in parallel.
def main [
  --keep-time (-t) # Keep each input file's modification time on its output.
  --auto-resize # Downsize without reducing coverage for configured target views.
  ...images: string # Images to convert. Outputs are named <input>.webp.
] {
  if ($images | is-empty) {
    let script_name = ($env.CURRENT_FILE | path basename)
    print --stderr $"Usage: ($script_name) [OPTIONS] image1.png image2.jpg ..."
    exit 1
  }

  let total = ($images | length)
  let max_workers = ([1 ((sys cpu | length) / 2 | math floor | into int)] | math max)
  let workers = ([$total $max_workers] | math min)
  let image_label = if $total == 1 { 'image' } else { 'images' }
  let worker_label = if $workers == 1 { 'worker' } else { 'workers' }
  let started_at = (date now)

  print $"Converting ($total) ($image_label) with ($workers) ($worker_label)..."

  let results = ($images | par-each --threads $workers {|image|
    let output = $"($image).webp"
    let resize_args = if $auto_resize {
      [-resize (auto-resize-geometry)]
    } else {
      []
    }
    let magick_args = [
      $image
      ...$resize_args
      -quality 95
      -define webp:method=6
      -define webp:auto-filter=true
      -define webp:thread-level=1
      -define webp:pass=10
      $output
    ]

    let conversion = (^magick ...$magick_args | complete)

    let timestamp = if $conversion.exit_code == 0 and $keep_time {
      ^touch -r $image $output | complete
    } else {
      {exit_code: 0, stdout: '', stderr: ''}
    }

    {
      image: $image
      output: $output
      success: ($conversion.exit_code == 0 and $timestamp.exit_code == 0)
      error: ([$conversion.stderr $timestamp.stderr] | where {|message| $message | is-not-empty } | str join (char newline))
    }
  } | enumerate | each {|entry|
    let completed = ($entry.index + 1)
    let result = $entry.item
    let elapsed_seconds = (((date now) - $started_at) / 1sec)
    let rate = if $elapsed_seconds > 0 { $completed / $elapsed_seconds } else { 0.0 }
    let remaining_seconds = if $rate > 0 { ($total - $completed) / $rate } else { 0.0 }
    let percent = ($completed * 100 / $total)
    let state = if $result.success { 'done' } else { 'FAILED' }

    print $"[($completed)/($total) ($percent | into string --decimals 0)%] ($state) ($result.image) · (format-seconds $elapsed_seconds) elapsed · ($rate | into string --decimals 2) image/s · (format-seconds $remaining_seconds) left"
    $result
  } | collect)

  let failures = ($results | where success == false)
  let elapsed_seconds = (((date now) - $started_at) / 1sec)
  let successful = ($total - ($failures | length))

  if ($failures | is-not-empty) {
    print --stderr ""
    for failure in $failures {
      print --stderr $"Failed: ($failure.image)"
      if ($failure.error | is-not-empty) {
        print --stderr ($failure.error | str trim)
      }
    }
    print --stderr $"($successful)/($total) ($image_label) converted in (format-seconds $elapsed_seconds)."
    exit 1
  }

  print $"All ($total) ($image_label) converted in (format-seconds $elapsed_seconds)."
}
