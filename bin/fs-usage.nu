#!/usr/bin/env nu
df -h -B MiB -x tmpfs -x device -x run /dev/mapper/waifu* | lines | split column --regex '\s+' | str trim | headers | update cells -c [1MiB-blocks, Used, Available] { into filesize }
