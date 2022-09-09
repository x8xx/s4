# S4 (Simple Software Switch Skeleton)
- 実験
- お遊び
- おもちゃ

## Dependencies
DPDK 22.03

LLVM 12

## DPDK
```
# setup nic
dpdk-devbind.py -b uio_pci_generic 0000:0x:00.0

# setup hugepages
echo 1 > /sys/kernel/mm/hugepages/hugepages-1048576kB/nr_hugepages
dpdk-hugepages.py --setup 1G

```

## switch\_dp
```
cd ./switch_dp
cargo build && sudo ./target/debug/switch_dp -c 0xf --vdev=net_tap0,iface=test1 -- --rx-cores 1 --fib-cores 2 -i net_tap0 -p ../wasm/parser/parser.wasm
```

## wasm
```
# setup
rustup target add wasm32-wasi

rustc -O --emit=obj --target wasm32-wasi parser.rs
```
