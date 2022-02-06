# Example gRPC Server

`restaurant` is an example gRPC server prepared for the `restaurant_customer` example which is compiled as a shared library loaded in q/kdb+.

## Build and Run

This process will run on a port 3160 with a name `master_sheff`. The container should be accessed with `http://localhost:3160`.
```sh
restaurant]$ ./scripts/build.sh
restaurant]$ ./scripts/run.sh
```

or you can simply run with an ordinary `run`:
```sh
restaurant]$ cargo run
```
