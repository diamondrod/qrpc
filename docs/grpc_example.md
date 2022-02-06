# gRPC Example

In this example we use the `example_service.proto` in `proto/` directory.
```protobuf
syntax="proto3";

package example_service;

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
  rpc Submit(Order) returns (Acceptance);
  rpc Finish(Expense) returns (Total);
}
```

## Server

An example gRPC server which uses the same proto file is provided in `examples/restaurant` directory. You can run the server following the `README` there. Here we just launch a container.
```sh
qrpc]$ cd examples/restaurant
restaurant]$ ./scripts/build.sh
restaurant]$ ./scripts/run.sh
Open restaurant
```

## Client

This is the main portion of this library. We will place some orders to the restaurant server and then go to a cacher.
```q
qrpc]$ q/grpc.q
q).grpc.set_endpoint["http://localhost:3160"]
"endpoint was set"
q).grpc.submit[`table`items`ordered_time!(2i; `Menu$`pizza`coke`pizza`sushi; .z.p)]
accepted| 1
q).grpc.submit[`table`items`ordered_time!(2i; `Menu$`steak`coke`sushi; .z.p)]
accepted| 1
q)receipt: .grpc.finish[enlist[`table]!enlist 2i]
q)receipt
history| +`time`item`unit`price!(2022.02.06D13:46:15.572580000 2022.02.06D13:..
total  | 48.25e
q)receipt `history
time                          item  unit price
----------------------------------------------
2022.02.06D13:46:15.572580000 pizza 2    7.5  
2022.02.06D13:46:15.572580000 sushi 1    10   
2022.02.06D13:46:15.572580000 coke  1    2    
2022.02.06D13:46:19.538015000 coke  1    2    
2022.02.06D13:46:19.538015000 sushi 1    10   
2022.02.06D13:46:19.538015000 steak 1    9.25 
q).grpc.finish[enlist[`table]!enlist 2i]
'no order for the table id: 2
  [0]  .grpc.finish[enlist[`table]!enlist 2i]
       ^
```
