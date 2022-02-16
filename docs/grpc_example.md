# gRPC Example

In this example we use the `restaurant.proto` in `proto/` directory. In order to include this directory, set:
```sh
export QRPC_PROTO_DIR=your_path/qrpc/proto
```

```protobuf
syntax="proto3";

package restaurant;

import "google/protobuf/empty.proto";
import "q.proto";

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

// Message representing an order.
message Order{
  int32 table = 1;
  repeated Menu items = 2;
  q.timestamp ordered_time = 3;
}

// Message representing acceptance.
message Acceptance{
  bool accepted = 1;
  string reason = 2;
}

// Message representing an expense with table ID.
message Expense{
  int32 table = 1;
}

// Message representing order history.
message History{
  q.timestamp time = 1;
  Menu item = 2;
  int64 unit = 3;
  float price = 4;
}

// Message representing a total due.
message Total{
  repeated History history = 1;
  float total = 2;
}

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

## Server

An example gRPC server which uses the same proto file is provided in `examples/restaurant` directory. Here we just run an executable with `cargo` command.
```sh
qrpc]$ cd examples/restaurant
restaurant]$ cargo run
Open restaurant
```

## Client

This is the main portion of this library. We will place some orders to the restaurant server and then go to a cacher.

*Note: bool field appears only if the value is true.*

```q
qrpc]$ q/grpc.q
q).grpc.set_endpoint["http://localhost:3160"]
"endpoint was set"
q).grpc.restaurant.submit[`table`items`ordered_time!(2i; `Menu$`pizza`coke`pizza`sushi; .z.p)]
accepted| 1
q).grpc.restaurant.submit[`table`items`ordered_time!(2i; `Menu$`steak`coke`sushi; .z.p)]
accepted| 1
q).grpc.restaurant.submit[`table`items`ordered_time!(2i; `Menu$`steak`steak`chips`coke`spaghetti`hamburger`chips`salad`pizza`sushi; .z.p)]
reason| "too many items. must be less than 10"
q).grpc.restaurant.cancel[`table`items`ordered_time!(3i; `Menu$`sushi`pizza`pizza; .z.p)]
'no order for the table id: 3
  [0]  .grpc.restaurant.cancel[`table`items`ordered_time!(3i; `Menu$`sushi`pizza`pizza; .z.p)]
       ^
q).grpc.restaurant.cancel[`table`items`ordered_time!(2i; `Menu$`sushi`pizza`pizza; .z.p)]
q)receipt: .grpc.restaurant.finish[enlist[`table]!enlist 2i]
q)receipt
history| +`time`item`unit`price!(2022.02.12D11:14:50.217026000 2022.02.12D11:..
total  | 23.25e
q)receipt `history
time                          item  unit price
----------------------------------------------
2022.02.12D11:14:50.217026000 coke  1    2    
2022.02.12D11:15:03.698417000 steak 1    9.25 
2022.02.12D11:15:03.698417000 coke  1    2    
2022.02.12D11:15:03.698417000 sushi 1    10   
q).grpc.restaurant.finish[enlist[`table]!enlist 2i]
'no order for the table id: 2
  [0]  .grpc.restaurant.finish[enlist[`table]!enlist 2i]
       ^
```
