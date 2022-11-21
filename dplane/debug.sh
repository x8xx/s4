#!/bin/bash

# cargo build --verbose && sudo ./target/debug/switch_dp -c 0xf --vdev=net_tap0,iface=test1 -- --rx-cores 1 --fib-cores 2 -i net_tap0 -p ../wasm/parser/parser.wasm
cargo build --verbose && sudo ./target/debug/switch_dp -c 0x7f --vdev=net_tap0,iface=test1 -- -c ./testdata/switch_config.yaml
