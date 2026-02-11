#!/usr/bin/env elvish
use re

script_name=(path-base (src)[name])

backup_suffix=".utf8everywhere.bak"

to_charset="UTF8"

fn usage []{
  echo Usage:
  echo "\t" $script_name TextFile1.txt TextFile2.txt ...
  echo
  echo "\t" each file will be replaced by utf8 encoded version
  echo "\t" and original file will be renamed to FILENAME.utf8everywhere.bak
}

if (== (count $args) 0) {
  usage
  exit
}

# process each input
put $@args | each [f]{

  # could be:
  # CHARSET with confidence [0,1]
  # no result
  chardet_result = (replaces &max=1 $f": " "" (chardetect $f))

  detected = (re:match "confidence" $chardet_result)
  detected_charset = ""
  detected_confidence = 0

  if $detected {
    splits_result = [(splits " with confidence " $chardet_result)]
    detected_charset = $splits_result[0]
    detected_confidence = $splits_result[1]
  }

  if (!=s 'utf-8' $detected_charset) {
    echo "file\t\t"$f
    echo "detected\t"(to-string $detected)
    echo "charset\t\t"$detected_charset
    echo "confidence\t"$detected_confidence
  }

  if (and $detected (> $detected_confidence 0.5)) {
    if (==s 'GB2312' $detected_charset) {
      mv $f $f$backup_suffix
      echo iconv -f $detected_charset -t $to_charset $f$backup_suffix -o $f
    }
  }
}