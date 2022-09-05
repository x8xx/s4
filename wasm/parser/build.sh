#!/bin/bash

rustc -O --emit=obj  --target wasm32-wasi parser.rs
wasm2wat parser.o | sed '$ s/)$/\(export "parse" \(func $parse\)\)\)/' > tmp.wat
wat2wasm tmp.wat -o parser.wasm
rm -f tmp.wat
rm -f parser.o
