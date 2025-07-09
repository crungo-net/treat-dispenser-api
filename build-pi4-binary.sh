#!/bin/bash

set -e

cross build --target=aarch64-unknown-linux-musl --release
cp target/aarch64-unknown-linux-musl/release/treat-dispenser-api /mnt/nas/treat-dispenser-api