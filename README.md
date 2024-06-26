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

1. run `bb_code_server_stop.sh`to disable bb-code-server-service, e.g.
    ```
    $ ./bb_code_server_stop.sh beagleplay
    ```
2. run `cpu_cores_disable.sh`to disable 2 of the 4 CPU cores, e.g.
    ```
    $ ./cpu_cores_disable.sh beagleplay
    ```
3. run `benchmark_start.sh` to perform all relevant benchmarks, e.g.
    ```
    $ ./benchmark_start.sh beagleplay itr-42
    ```
4. wait for the benchmark plots to be opened in the browser on your host machine
