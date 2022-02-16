# Test

Unit test utilizes `example.proto` and `restaurant.proto` under `proto/`. In order to include this directory, set:
```sh
export QRPC_PROTO_DIR=your_path/qrpc/proto
```

Test items are categorized into two parts:
- Message conversion test
- gRPC test

First gRPC server must be launched with `restaurant` example.
```sh
restaurant]$ cargo run
Restaurant was opened
```

Then run test from the root directory.
```sh
qrpc]$ cargo build
qrpc]$ cp target/debug/libqrpc.so .
qrpc]$ q tests/test.q 
test result: ok. 35 passed; 0 failed
```
