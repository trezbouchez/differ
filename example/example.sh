#!/bin/bash
set -e

cd ..
cargo build
cd example
../target/debug/differ ./monkey_before.tiff ./monkey_after.tiff ./monkey_patched.tiff ./monkey_edits.txt
