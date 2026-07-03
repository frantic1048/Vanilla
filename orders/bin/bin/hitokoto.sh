#!/usr/bin/env bash

#输出一条一言的内容

#来源
#http://hitokoto.us/

echo $(node -p "($(curl -ks https://api.hitokoto.us:214/rand?charset=utf-8)).hitokoto")

