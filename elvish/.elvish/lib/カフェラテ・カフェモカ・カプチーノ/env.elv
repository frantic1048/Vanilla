# env vars
paths = [
  {~}/bin
  {~}/npm-global/bin
  {~}/.gem/ruby/2.2.0/bin
  {~root}/.composer/vendor/bin
  $@paths
]

E:NODE_PATH = (joins : [
  {~}/npm-global/lib/node_modules
  /usr/lib/node_modules
  (splits : $E:NODE_PATH)
])

E:VISUAL = "nano"
