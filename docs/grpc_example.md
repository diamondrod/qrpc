# gRPC Example

## Example1 Restaurant

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

An example gRPC server which uses the same proto file is provided in `examples/restaurant` directory. Here we just run an executable with `cargo` command. The server will run on a port 3160.
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
q).grpc.set_endpoint[`restaurant; "http://localhost:3160"]
"endpoint was set for package: restaurant"
q).grpc.restaurant.submit[`table`items`ordered_time!(2i; `.grpc.restaurant.Menu$`pizza`coke`pizza`sushi; .z.p)]
accepted| 1
q).grpc.restaurant.submit[`table`items`ordered_time!(2i; `.grpc.restaurant.Menu$`steak`coke`sushi; .z.p)]
accepted| 1
q).grpc.restaurant.submit[`table`items`ordered_time!(2i; `.grpc.restaurant.Menu$`steak`steak`chips`coke`spaghetti`hamburger`chips`salad`pizza`sushi; .z.p)]
reason| "too many items. must be less than 10"
q).grpc.restaurant.cancel[`table`items`ordered_time!(3i; `.grpc.restaurant.Menu$`sushi`pizza`pizza; .z.p)]
'no order for the table id: 3
  [0]  .grpc.restaurant.cancel[`table`items`ordered_time!(3i; `.grpc.restaurant.Menu$`sushi`pizza`pizza; .z.p)]
       ^
q).grpc.restaurant.cancel[`table`items`ordered_time!(2i; `.grpc.restaurant.Menu$`sushi`pizza`pizza; .z.p)]
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

## Example1 Ticketing Machine

In this example we use the `ticket.proto` in `proto/` directory. In order to include this directory, set:
```sh
export QRPC_PROTO_DIR=your_path/qrpc/proto
```

Note that there is a method `Cancel` which is spelled samely as the one in the restaurant example. qrpc loads this method under `.grpc.ticket` namespace to avoid name collision.

```protobuf
syntax="proto3";

package ticket;

import "google/protobuf/empty.proto";
import "q.proto";

// Class of seat
enum Class{
  no_preference = 0;
  stand = 1;
  arena = 2;
  vip = 3;
}

// Information to apply to an event.
message Application{
  // Name of a customer.
  string name = 1;
  // Event date.
  q.date date = 4;
  // The number of seats to reserve.
  int32 number = 5;
  // Class of seat.
  Class class = 6;
}

// Information of a reserved ticket, or a information to cancel a flight.
message TicketInfo{
  // Name of a customer.
  string name = 1;
  // Seat ID.
  repeated q.symbol seats = 2;
  // Flight date.
  q.date date = 3;
}

// Failure response to a flight request.
message ReservationFailure{
  string message = 1;
}

// Response to a flight request.
message Processed{
  oneof result{
    // Reservation success.
    TicketInfo ticket = 1;
    // Reservation failure.
    ReservationFailure failure = 2;
  };
}

// Message to notify that cancellation was completed.
message Cancelled{
  string message = 1;
}

// Table of available seats by seat class.
message AvailableSeats{
  map<string, int32> inventory = 1;
}

// Service to issue a ticket.
service TicketingMachine{
  // Customer requests seats and get a result.
  rpc Reserve(Application) returns (Processed);
  // Customer cancels seats.
  rpc Cancel(TicketInfo) returns (Cancelled);
  // Customer requests current status of the inventory.
  rpc GetAvailableSeats(google.protobuf.Empty) returns (AvailableSeats);
}
```

## Server

An example gRPC server which uses the same proto file is provided in `examples/ticketing_machine` directory. Here we just run an executable with `cargo` command. The server will run on a port 3130.
```sh
qrpc]$ cd examples/ticketing_machine
restaurant]$ cargo run
Ticketing machine was booted
```

## Client

This is the main portion of this library. We will place some orders to the restaurant server and then go to a cacher.

```q
qrpc]$ q/grpc.q
q).grpc.set_endpoint[`ticket; "http://localhost:3130"]
"endpoint was set for package: ticket"
q).grpc.ticket.get_available_seats[]
         | stand arena vip
---------| ---------------
inventory| 3     2     1  
q).grpc.ticket.reserve[`name`date`number`class!("Peter"; 2022.03.16; 4i; `.grpc.ticket.Class$`arena)]
       | message                                       
-------| ----------------------------------------------
failure| "we cannnot reserve 4 seats. Only 2 available"
q)ticket: .grpc.ticket.reserve[`name`date`number`class!("Peter"; 2022.03.16; 2i; `.grpc.ticket.Class$`arena)]
      | name    seats date      
------| ------------------------
ticket| "Peter" a1 a2 2022.03.16
q)ticket `ticket
name | "Peter"
seats| `a1`a2
date | 2022.03.16
q).grpc.ticket.reserve[`name`date`number`class!("John"; 2022.03.16; 1i; `.grpc.ticket.Class$`arena)]
       | message                                       
-------| ----------------------------------------------
failure| "we cannnot reserve 1 seats. Only 0 available"
q)// Let's cancel Peter's seat!!
q).grpc.ticket.cancel[`name`seats`date!("John"; enlist `a1; 2022.03.16)]
message| "a1 could not be cancelled."
q)// Peter cancels his seat for John
q).grpc.ticket.cancel[`name`seats`date!("Peter"; enlist `a1; 2022.03.16)]
message| "a1 were cancelled."
q).grpc.ticket.reserve[`name`date`number`class!("John"; 2022.03.16; 1i; `.grpc.ticket.Class$`arena)]
      | name   seats date      
------| -----------------------
ticket| "John" a1    2022.03.16
q).grpc.ticket.get_available_seats[]
         | vip stand
---------| ---------
inventory| 1   3    
q).grpc.ticket.reserve[`name`date`number`class!("Mr.J"; 2022.03.16; 1i; `.grpc.ticket.Class$`vip)]
      | name   seats date      
------| -----------------------
ticket| "Mr.J" v1    2022.03.16
q).grpc.ticket.get_available_seats[]
         | stand
---------| -----
inventory| 3   
```
