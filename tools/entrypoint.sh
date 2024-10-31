#!/bin/bash

nohup scd 2>&1 >/tmp/scd.log &

sudo /usr/sbin/sshd -D
