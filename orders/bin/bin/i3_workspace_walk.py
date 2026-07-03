#!/usr/bin/env python3

import json
import subprocess
import argparse

parser = argparse.ArgumentParser(description='walk around i3 workspaces')

parser.add_argument('--next_active', action='store_true')
parser.add_argument('--previous_active', action='store_true')
parser.add_argument('--next', action='store_true')
parser.add_argument('--previous', action='store_true')

args = parser.parse_args()

activeWorkspaces = json.loads(subprocess\
    .Popen('i3-msg -t get_workspaces',
        shell=True,
        stdout=subprocess.PIPE) \
    .stdout.read())

allWorkspaces = [i + 1 for i in range(10)]

presentWorkspace = 1
previousActiveWorkspace = 1
nextActiveWorkspace = 1
previousWorkspace = 1
nextWorkspace = 1

for i in range(len(activeWorkspaces)):
    if activeWorkspaces[i]['focused'] == True:
        activeWSIndex = i
        allWSIndex = activeWorkspaces[i]['num'] - 1

        if i == (len(activeWorkspaces) - 1):
            activeWSIndex -= len(activeWorkspaces)
            allWSIndex -= len(allWorkspaces)

        presentWorkspace = activeWorkspaces[activeWSIndex]['num']
        previousActiveWorkspace = activeWorkspaces[activeWSIndex - 1]['num']
        nextActiveWorkspace = activeWorkspaces[activeWSIndex + 1]['num']

        previousWorkspace = allWorkspaces[allWSIndex - 1]
        nextWorkspace = allWorkspaces[allWSIndex + 1]

targetWorkspace = nextWorkspace

print(args)

if args.next_active:
    targetWorkspace = nextActiveWorkspace
elif args.previous_active:
    targetWorkspace = previousActiveWorkspace
elif args.next:
    targetWorkspace = nextWorkspace
elif args.previous:
    targetWorkspace = previousWorkspace

subprocess.call(['i3-msg', 'workspace', str(targetWorkspace)])
