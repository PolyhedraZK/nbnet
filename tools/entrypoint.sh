#!/bin/bash

scd -d >>/tmp/scd.log 2>&1

sudo /usr/sbin/sshd -D
