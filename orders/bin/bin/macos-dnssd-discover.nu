#!/usr/bin/env nu
# Discover local mDNS/Bonjour services: type → instance (host:port)

# Heading
print ("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | ansi gradient --fgstart '0x40c9ff' --fgend '0xe81cff')
print $"  (ansi cyan)✦(ansi reset)  (ansi default_bold)mDNS Service Discovery(ansi reset)  (ansi purple)✦(ansi reset)"
print ("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | ansi gradient --fgstart '0xe81cff' --fgend '0x40c9ff')
print ''

# Enumerate service types (5s collection window)
print $"(ansi default_italic)Collecting service types \(5s\)...(ansi reset)"
let types = (
  ^timeout 5 dns-sd -B _services._dns-sd._udp local. | complete | get stdout
  | lines
  | parse -r '(?P<proto>_(?:tcp|udp))\.local\.\s+(?P<name>_\S+)'
  | each {|r| $"($r.name).($r.proto)" }
  | sort
  | uniq
)
print $"(ansi default_italic)Found ($types | length) service types, resolving in parallel \(3s\)...(ansi reset)\n"

# Resolve each type in parallel (~3s total instead of N×3s)
let results = ($types | par-each {|t|
  let raw = (^timeout 3 dns-sd -Z $t local. | complete | get stdout)
  let entries = ($raw
    | lines
    | where {|l| ($l =~ 'SRV') and (not ($l | str starts-with ';')) }
    | each {|line|
      let f = ($line | split row -r '\s+' | where {|s| $s != '' })
      let si = ($f | enumerate | where {|e| $e.item == 'SRV' } | first | get index)
      let port = ($f | get ($si + 3))
      let host = ($f | get ($si + 4) | str trim -r -c '.')
      let inst = ($f | get 0
        | str replace $".($t).local." ''
        | str replace $".($t)" ''
        | str replace -a '\032' ' ')
      { instance: $inst, host: $host, port: $port }
    }
    | uniq-by instance host port
    | sort-by instance
  )

  if ($entries | is-not-empty) {
    { type: $t, entries: $entries }
  }
} | compact | sort-by type)

# Resolve unique hostnames to IPs in parallel (~1s)
let all_hosts = ($results | each {|s| $s.entries | get host } | flatten | uniq)
print $"(ansi default_italic)Resolving ($all_hosts | length) hosts...(ansi reset)\n"
let ip_map = ($all_hosts | par-each {|h|
  # ping first line: "PING host (ip): ..." — shows IP even if no reply
  let ip = try {
    ^ping -c 1 -t 1 $h | complete | get stdout
    | lines | first
    | parse -r '\((?P<ip>[0-9.]+)\)' | first | get ip
  } catch { null }
  { host: $h, ip: $ip }
})

# Display results
for svc in $results {
  print $"(ansi cyan_bold)($svc.type)(ansi reset)"
  for e in $svc.entries {
    let resolved = ($ip_map | where {|r| $r.host == $e.host })
    let ip_str = if ($resolved | is-not-empty) and ($resolved | first | get ip) != null {
      $" (ansi cyan)\(($resolved | first | get ip)\)(ansi reset)"
    } else { "" }
    print $"  (ansi default_bold)($e.instance | fill -w 35)(ansi reset) (ansi green)($e.host)(ansi reset):(ansi yellow)($e.port)(ansi reset)($ip_str)"
  }
  print ''
}
