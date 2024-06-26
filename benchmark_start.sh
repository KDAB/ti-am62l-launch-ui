#!/bin/bash
HOST=$1
ITERATION_PREFIX=$2

./build_deploy_and_run_benchmark_on_target.sh $HOST winit-software $ITERATION_PREFIX
./build_deploy_and_run_benchmark_on_target.sh $HOST linuxkms-software $ITERATION_PREFIX
./build_deploy_and_run_benchmark_on_target.sh $HOST linuxkms-skia-software $ITERATION_PREFIX
