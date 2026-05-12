#!/usr/bin/env python3
# -*- coding: utf-8 -*-

#TTS via espeak

import os

while(1):
  os.system('espeak -v zh -s 200 ' + '" ' + input() + ' "')
