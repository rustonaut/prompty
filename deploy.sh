#!/usr/bin/bash

set -e

cargo build --release
cp ./target/release/prompty /opt/data/.bin/prompty