#!/usr/bin/env bash

sudo tee -a /etc/sysctl.conf <<EOF
kern.maxfiles=10485760
kern.maxfilesperproc=1048576
EOF
