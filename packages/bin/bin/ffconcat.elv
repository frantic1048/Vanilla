#!/bin/env elvish

use re
#use whtsky
use str
use path

var temp_file = (mktemp -p ./)
var files = [(fd -e wmv --regex '[[:digit:]]{2}\.wmv')]
#var files = (whtsky:filter [name]{ put (re:match '[\d]{2}\.wmv' $name) } [*.wmv])
var list =  (str:join "\n" [(each [f]{ put file" '"$f"'" } $files)])

echo Files to concat:
pprint (all $files)
echo
echo $list >$temp_file

var output_file_name = (re:replace '[\d]{2}\.wmv' '_full.wmv' (path:base [*.wmv][0]))

ffmpeg -f concat -safe 0 -i $temp_file -c copy $output_file_name

rm -v $temp_file
