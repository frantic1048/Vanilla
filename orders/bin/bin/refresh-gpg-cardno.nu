#!/usr/bin/env nu

# Make gpg-agent pick up the currently inserted smart card.
# https://security.stackexchange.com/questions/165286/how-to-use-multiple-smart-cards-with-gnupg
def main [] {
  ^gpg-connect-agent "scd serialno" "learn --force" /bye
}
