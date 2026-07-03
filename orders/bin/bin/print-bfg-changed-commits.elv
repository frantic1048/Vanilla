#!/usr/bin/env elvish

use str
use path

var script_name = (path:base (src)[name])

fn usage []{
  echo proceeds object-id-map.old-new.txt logged by BFG repo cleaner
  echo prints old commit object hash to stdout.
  echo
  echo Usage:
  echo "\t" $script_name path-to-object-id-map.old-new.txt
  echo
  echo "\t" each line of path-to-object-id-map.old-new.txt
  echo "\t" looks like "("old_object_hash new_object_hash")":
  echo
  echo "\t" 02fe4b871cde55a6e2cd33857632f4ba6031b477 940689fef04e7ef9fea61be75e40d281e18b1355
}

if (< (count $args) 1) {
  usage
  exit
}

cat $args[0] | each [line]{
  var old_object_hash = [(str:split ' ' $line)][0]
  if (==s (git cat-file -t $old_object_hash) 'commit') {
    echo (git cat-file -t $old_object_hash) $old_object_hash
  }
}
