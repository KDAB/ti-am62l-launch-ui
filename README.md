# AM62L - Benchmark

## About

This repo contains a slint demo based on the `slint-rust-template`.
The purpose of this UI is to do initial benchmarking in the TI AML62L out-of-the-box-experience project
with BeaglePlay boards.

## Prerequisites

- requires rust
- requires cross
- requires ssh access to BeaglePlay without a password (add your ssh key to `~/.ssh/authorized_keys`)
- you probably want to add an entry for your BeaglePlay board in your ssh config
- check if `BROWSER` and `REMOTE_TARGET_DIR` in `build_deploy_and_run_benchmark_on_target.sh` are suitable for your setup

## Usage

1. run `cpu_cores_disable.sh`to disable 2 of the 4 CPU cores, e.g.
    ```
    $ ./cpu_cores_disable.sh beagleplay
    ```
2. run `build_deploy_and_run_benchmark_on_target.sh` to perform a benchmark, e.g.
    ```
    $ ./build_deploy_and_run_benchmark_on_target.sh beagleplay winit-software
    ```
3. wait for the benchmark plot to be opened in the browser on your host machine
