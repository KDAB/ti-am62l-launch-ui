#!/bin/bash
HOST=$1

ssh -t $HOST "\
  echo 1 | sudo tee -a /sys/devices/system/cpu/cpu2/online > /dev/null; \
  echo 1 | sudo tee -a /sys/devices/system/cpu/cpu3/online > /dev/null;\
"
