#!/bin/bash

source ../utils/bench_utils.sh

declare -a BENCH_NAMES=(
    "sqlite/ext2_benchmarks"
)

declare -a BENCH_METHODS=(
    "guest_only"
)

runInputBenchs

collectResults "sqlite"

mv ../../../test/benchmark/aster_output.txt ../result/sqlite/
mv ../../../test/benchmark/linux_output.txt ../result/sqlite/
