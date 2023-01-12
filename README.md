# S4 (Super Speed Software Switch)
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

dpdk-hugepages.py -p 1G --setup 2G
cat /sys/kernel/mm/hugepages/hugepages-1048576kB/nr_hugepages
# 2

```
