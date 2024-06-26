#!/bin/bash
# TODO we can probably do things nicer throughout this script
HOST=$1
BACKEND=$2
ITERATION_PREFIX=$3

BROWSER=firefox
REMOTE_TARGET_DIR=/home/debian # default user for BeaglePlay distro is 'debian'

# system-info-collector
## get sources
git clone https://github.com/qarmin/system-info-collector.git
cd system-info-collector
git checkout deb447ccc02cd3e3f26b4df3c32536ac3e10362e
## cross build
cross build --target aarch64-unknown-linux-gnu --release

cd ..

# benchmark ui
## cross build
cross build --target aarch64-unknown-linux-gnu --release

# deploy binaries
scp -O ./target/aarch64-unknown-linux-gnu/release/AM62L_benchmark_ui $HOST:$REMOTE_TARGET_DIR
scp -O ./system-info-collector/target/aarch64-unknown-linux-gnu/release/system_info_collector $HOST:$REMOTE_TARGET_DIR

# setup benchmark run
## clean up previous run
ssh $HOST -C "rm system_data.csv; rm system_data_plot.html"
## disable CPU cores
# ssh -t $HOST "for i in /sys/devices/system/cpu/cpu[2-3]*/online; do sudo echo 0 > "'$i'"; done" #TODO fix sudo

# run benchmark
## start system-info-collector
ssh $HOST 'nohup bash -c "( ( ./system_info_collector -l debug -a collect-and-convert -m memory-used -m memory-free -m memory-available -m cpu-usage-total -m cpu-usage-per-core -c 0.5 &>/dev/null ) & )"'
## wait
ssh $HOST -C "sleep 10s"
## start binary under test
ssh $HOST 'nohup bash -c "( ( DISPLAY=:0 SLINT_BACKEND='$BACKEND' SLINT_FULLSCREEN=:1 SLINT_STYLE=fluent-dark ./AM62L_benchmark_ui &>/dev/null ) & )"'
# wait
ssh $HOST -C "sleep 20s"
# terminate binary under test
ssh $HOST -C "killall AM62L_benchmark_ui"
# wait
ssh $HOST -C "sleep 10s"
# terminate system-info-collector
ssh $HOST -C "killall system_info_collector"

# tear down benchmark run
## enable CPU cores
# ssh -t $HOST "for i in /sys/devices/system/cpu/cpu[2-3]*/online; do sudo echo 1 > "'$i'"; done" #TODO fix sudo
## get benchmark data
mkdir data
scp -O $HOST:$REMOTE_TARGET_DIR/system_data.csv ./data/"$ITERATION_PREFIX"_"$BACKEND".csv
scp -O $HOST:$REMOTE_TARGET_DIR/system_data_plot.html ./data/"$ITERATION_PREFIX"_"$BACKEND".html
## open system-info-collector plot
$BROWSER ./data/"$ITERATION_PREFIX"_"$BACKEND".html
