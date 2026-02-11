#!/usr/bin/env bash

set -xeo pipefail

sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
