# Example gRPC Server

`restaurant` is an example gRPC server prepared for the `restaurant_customer` example which is compiled as a shared library loaded in q/kdb+.

## Build and Start Container

This process will run on a port 3160.
```sh
restaurant]$ docker build -t restaurant:latest .
restaurant]$ docker run --rm restaurant:latest
```
