#!/bin/bash

if [ -d "$HOME/miri" ]; then
    rm -rf $HOME/miri
fi

if [ -d "$HOME/miri_asterinas" ]; then
    rm -rf $HOME/miri_asterinas
fi

git clone --single-branch -b kern_miri https://github.com/asterinas/atc25-artifact-evaluation.git ~/miri
git clone --single-branch -b miri_asterinas https://github.com/asterinas/atc25-artifact-evaluation.git ~/miri_asterinas

pushd ~/miri

git reset --hard 58f0aab9f9c7798670913b050624cbe665c4cd7f
rustup install nightly-2024-11-04
rustup override set nightly-2024-11-04
rustup component add cargo rust-src rustc-dev llvm-tools rustfmt clippy

./miri install
export PATH=$PATH:$HOME/.rustup/toolchains/nightly-2024-11-04-x86_64-unknown-linux-gnu/bin

popd

pushd ~/miri_asterinas

git reset --hard 59dca48f8b2d9d3e5edd8ef89443417b57749682
rustup override set nightly-2024-11-04
make install_osdk
make build

pushd ostd
MIRIFLAGS="-Zmiri-disable-stacked-borrows -Zmiri-ignore-leaks" cargo osdk miri run
popd

popd
# Cleanup
rm -rf ~/miri
rm -rf ~/miri_asterinas
