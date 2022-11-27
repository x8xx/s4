#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname $0); pwd)
cd $SCRIPT_DIR

rustc -O --emit=obj  --target wasm32-wasi parser.rs
wasm2wat parser.o | sed '$ s/)$/\(export "parse" \(func $parse\)\)\)/' > tmp.wat
cat tmp.wat
wat2wasm tmp.wat -o parser.wasm
rm -f tmp.wat
rm -f parser.o
