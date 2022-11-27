#!/bin/bash

SCRIPT_DIR=$(cd $(dirname $0); pwd)
cd $SCRIPT_DIR

_benchmark() {
    dir=$1
    cd $dir
    make

    RUNAPP=$2
    if [ "$RUNAPP" == "s4" ]
    then
        sudo ../../../dplane/target/release/s4dp -c 0x7f --vdev=net_tap0,iface=test1 -- -c ./switch_config.yml
        # sudo ../../../dplane/target/debug/s4dp -c 0x7f --vdev=net_tap0,iface=test1 -- -c ./switch_config.yml
    fi

    if [ "$RUNAPP" == "e_pktgen" ]
    then
        sudo ../../e_pktgen/target/release/e_pktgen -c 0xf --vdev=net_tap0,iface=test1 -- ./e_pktgen_conf.yml ./pktgen/target/release/libpktgen.so
    fi
}


if [ "$#" -eq 2 ]; then
    _benchmark ./testdata/$1 $2
else
    for dir in `ls -d ./testdata/*/ | sed 's/ //g'`
    do
        _benchmark $dir $1
    done
fi
