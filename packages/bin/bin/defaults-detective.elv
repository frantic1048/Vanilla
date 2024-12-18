#!/usr/bin/env elvish

# MEMO:
# nushell does not support signal trap or similar feature
# https://github.com/nushell/nushell/issues/8360
use os
use str
use file

var tmpdir = (os:temp-dir 'defaults-detective-*')

set before-exit = [
  $@before-exit
  {
    echo (styled 'Cleaning up' bold blue) $tmpdir
    if (os:is-dir $tmpdir) {
      rm -r $tmpdir
    } else {
      echo $tmpdir does not exist, skipping cleanup
    }
  }
]

echo (styled 'Discovering domains' bold blue)

# TODO:
# use `defaults export DOMAIN - >domain.plist`
# diff plist (plist have extra type info)
# print defaults command to reflect changes

# Domains that are not useful for system configuration
var maskedDomains = [
  com.apple.biomesyncd
  com.apple.CallHistorySyncHelper
  com.apple.cseventlistener
  com.apple.DuetExpertCenter.AppPredictionExpert
  com.apple.FaceTime
  com.apple.HIToolbox
  com.apple.knowledge-agent
  com.apple.madrid
  com.apple.mmcs
  com.apple.photolibraryd
  com.apple.photos.shareddefaults
  com.apple.proactive.PersonalizationPortrait
  com.apple.routined
  com.apple.spaces
  com.apple.spotlightknowledge
  com.apple.tipsd
  com.apple.xpc.activity2
  ContextStoreAgent
]

var domains = []
peach {|domain|
  if (and ?(e:defaults read $domain >/dev/null 2>&1) (not (has-value $maskedDomains $domain))) {
    echo '->' (styled $domain bold green)
    set domains = (conj $domains $domain)
  }
} [(str:split ', ' (e:defaults domains))]

peach {|domain|
  touch $tmpdir/$domain{.old, ''}
  e:defaults read $domain > $tmpdir/$domain
  # e:defaults export $domain - > $tmpdir/$domain
} $domains

echo (styled 'Detecting changes' bold blue)

while $true {
  each {|domain|
    cp $tmpdir/$domain{'', .old}
    e:defaults read $domain > $tmpdir/$domain
    # e:defaults export $domain - > $tmpdir/$domain

    # for shorter path display
    with pwd = $tmpdir {
      var difference = (file:pipe)
      var result = ?(e:git diff -U0 --word-diff=porcelain --no-index --no-prefix -- $domain{.old, ''} > $difference[w])
      if (and (has-key $result[reason] exit-status) (== $result[reason][exit-status] 1)) {
        file:close $difference[w]
        var changes = [(str:trim-space (slurp < $difference[r]) | to-lines)][4..-1]
        set changes = ['~' $@changes]
        # multiple sections like this:
        # '@@ -8 +8 @@'
        # '     ArcadePayoutIntervalStartDate = '
        # '-"2024-12-12 22:40:34'
        # '+"2024-12-13 10:41:13'
        # '  +0000";'
        # '~'
        #
        # end with empty line
        each {|change|
          if (==s $change '~') {
            print &sep='' "\n" (styled $domain bold '#e7cb21' 'bg-#111111') " "
          } elif (str:has-prefix $change ' ') {
            print (str:trim-space $change[1..])
          } elif (str:has-prefix $change '-') {
            print (styled $change[1..] bold red)
          } elif (str:has-prefix $change '+') {
            print (styled $change[1..] bold green)
          }
        } $changes
        file:close $difference[r]
      }
    }

  } $domains

  sleep 1s
}
