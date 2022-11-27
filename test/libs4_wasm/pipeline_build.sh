#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname $0); pwd)
cd $SCRIPT_DIR

rustc -O --emit=obj  --target wasm32-wasi pipeline.rs
wasm2wat pipeline.o | sed '$ s/)$/\(export "run_pipeline" \(func $run_pipeline\)\)\)/' > tmp.wat
cat tmp.wat
wat2wasm tmp.wat -o pipeline.wasm
rm -f tmp.wat
rm -f pipeline.o
