# Examples

In the examples below we use example proto file `example.proto` wchich exists under `proto/` directory.

```protobuf
syntax="proto3";

package example;

import "q.proto";

// Message composed of scalar types.
message Scalar{
  bool bool_f = 1;
  int32 int_f = 2;
  int64 long_f = 3;
  float real_f = 4;
  double float_f = 5;
  q.symbol symbol_f = 6;
  q.timestamp timestamp_f = 7;
  q.month month_f = 8;
  q.date date_f = 9;
  q.datetime datetime_f = 10;
  q.timespan timespan_f = 11;
  q.minute minute_f = 12;
  q.second second_f = 13;
  q.time time_f = 14;
}

// Inner message contained in `Outer`.
message Inner{
  int64 inner_muscle = 1;
  q.symbol inner_mind = 2;
}

// Nested message.
message Outer{
  bool out_law = 1;
  Inner inner = 2;
}
```

## Scalar Example

```q
q)encoded: .grpc.encode[`example.Scalar; `bool_f`int_f`long_f`real_f`symbol_f`timestamp_f`month_f`date_f`datetime_f`timespan_f`minute_f`second_f`time_f!(1b; 42i; 7; 1.23e; `kdb; .z.p; 2022.01m; 2022.01.27; .z.z; 1D23:45:01.23456789; 12:34; 12:34:56; 12:34:56.789)]
q).grpc.decode[`example.Scalar; encoded]
bool_f     | 1b
int_f      | 42i
long_f     | 7
real_f     | 1.23e
float_f    | 0f
symbol_f   | `kdb
timestamp_f| 2022.01.27D07:37:53.770462000
month_f    | 2022.01m
date_f     | 2022.01.27
datetime_f | 2022.01.27T07:37:53.770
timespan_f | 1D23:45:01.234567890
minute_f   | 12:34
second_f   | 12:34:56
time_f     | 12:34:56.789
```

## Nested Message

```q
q)encoded: .grpc.encode[`example.Outer; `out_law`inner!(1b; `inner_muscle`inner_mind!(150; `silent))];
q).grpc.decode[`example.Outer; encoded]
out_law| 1b
inner  | `inner_muscle`inner_mind!(150;`silent)
```
