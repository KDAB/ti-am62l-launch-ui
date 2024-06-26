This benchmark:
- was done running the `build_deploy_and_run_benchmark_on_target.sh` with the line to run the benchmark UI commented
- serves as a Distro specific baseline
- shows that some tasks drive the CPU load up
- the main tasks identified is: bb-code-server
- I'll disable that service
