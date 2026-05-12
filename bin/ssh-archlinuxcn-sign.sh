#!/bin/bash

# forwarding gpg-agent
ssh -R /run/user/1046/gnupg/S.gpg-agent:/home/chino/.gnupg/S.gpg-agent.extra -o StreamLocalBindUnlink=yes frantic1048@build.archlinuxcn.org
