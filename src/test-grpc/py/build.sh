#!/bin/bash

declare -a services=("../../protos/")
for SERVICE in "${services[@]}"; do
    echo $SERVICE
    DESTDIR='./'
    mkdir -p $DESTDIR
    python -m grpc_tools.protoc \
        --proto_path=$SERVICE/ \
        --python_out=$DESTDIR \
        --grpc_python_out=$DESTDIR \
        $SERVICE/*.proto
done