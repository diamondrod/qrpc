# qRPC

![](docs/images/qrpc_logo.svg?sanitize=true)

gRPC client for q/kdb+.

## Introduction

q/kdb+ implements propriate protocol to communicate among q processes and HTTP(1) protocol. There are some client library for q which implements the propriate protocol but t is not suitable to integrate in a large system. On the other hand it is rare that HTTP becomes a main protocol in this age.

When your processes have limited number of message types and furthermore you want to check the validity of contents, HTTP is not suitable for it. Then gRPC came to play, which is based on HTTP2 and afford to specify solid message types.

Users of q/kdb+ might have moarned that they could not use gRPC in q and used HTTP without any options hacking erroneous messages leaning deeply against a chair-back. This library is a gRPC client implementation of q to pour water ("of life", if you will) to such a people.

## Features

As protobuf message is strictly typed based on proto files, `qrpc` needs to compile source proto files at building a shared library. For this reason users have to put their proto files to use for gRPC under a directory specified by an environmental variable `QRPC_PROTO_DIR`.

And lo, and behold, `qrpc` automatically generates Rust code for gRPC client methods defined in `service` in the user input, and of course, q code to load the shared library. For example, services below;
```protobuf
// Service mocking a restaurant order system.
service Restaurant{
  // Customer submits an order and a kitchen returns a response.
  rpc Submit(Order) returns (Acceptance);
  // Customer finish a meal handing an expense and a restaurant displays a total due
  //  with an order history.
  rpc Finish(Expense) returns (Total);
  // Customer forcefully cancels an order.
  rpc Cancel(Order) returns (google.protobuf.Empty);
}
```
will be loaded into q as:
```q
// Load gRPC client method submit.
.grpc.submit: `libqrpc 2: (`submit; 1);

// Load gRPC client method finish.
.grpc.finish: `libqrpc 2: (`finish; 1);

// Load gRPC client method cancel.
.grpc.cancel: `libqrpc 2: (`cancel; 1);
```

`qrpc` also supports `enum` message. For example, enum message
```protobuf
// Available menu.
enum Menu{
  smile = 0;
  pizza = 1; 
  spaghetti = 2;
  salad = 3;
  steak = 4;
  sushi = 5;
  hamburger = 6;
  chips = 7;
  coke = 8;
}
```
will define enum variable `Menu` in  the same file as the generated one for service:
```q
// Source of enum message Menu.
Menu: `smile`pizza`spaghetti`salad`steak`sushi`hamburger`chips`coke;
```

## Table of Contents

1. [Type Mapping](docs/type_mapping.md)
2. [Message Examples](docs/message_examples.md)
3. [gRPC Example](docs/grpc_example.md)
