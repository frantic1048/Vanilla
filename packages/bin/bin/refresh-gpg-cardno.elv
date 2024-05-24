#!/usr/bin/env elvish

# https://security.stackexchange.com/questions/165286/how-to-use-multiple-smart-cards-with-gnupg
gpg-connect-agent "scd serialno" "learn --force" /bye
