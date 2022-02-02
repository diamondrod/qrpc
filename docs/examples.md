# Examples

In the examples below we use example proto file `example.proto` wchich exists under `proto/` directory.

```protobuf
syntax="proto3";

package example;

import "q.proto";

// Message composed of scalar types.
message Atoms{
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

// Message composed of list types.
message Lists{
  repeated bool bool_f = 1;
  repeated int32 int_f = 2;
  repeated int64 long_f = 3;
  repeated float real_f = 4;
  repeated double float_f = 5;
  repeated q.symbol symbol_f = 6;
  repeated q.timestamp timestamp_f = 7;
  repeated q.month month_f = 8;
  repeated q.date date_f = 9;
  repeated q.datetime datetime_f = 10;
  repeated q.timespan timespan_f = 11;
  repeated q.minute minute_f = 12;
  repeated q.second second_f = 13;
  repeated q.time time_f = 14;
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

// Table row.
message Row{
  q.symbol host = 1;
  sint32 port = 2;
  q.timespan running = 3;
  string user = 4;
}

// Message containing a table.
message Table{
  repeated Row rows = 1;
}

// Message composed of maps.
message Mappy{
  map<string, int32> id = 1;
  map<int64, q.month> xday = 2; 
  map<bool, Inner> physical = 3;
}
```

## Atom Example

```q
q)atoms: `bool_f`int_f`long_f`real_f`symbol_f`timestamp_f`month_f`date_f`datetime_f`timespan_f`minute_f`second_f`time_f!(1b; 42i; 7; 1.23e; `kdb; .z.p; 2022.01m; 2022.01.27; .z.z; 1D23:45:01.23456789; 12:34; 12:34:56; 12:34:56.789);
q)encoded: .grpc.encode[`example.Atoms; atoms]
q).grpc.decode[`example.Atoms; encoded]
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

## List Example

```q
q)atoms: `bool_f`int_f`long_f`real_f`symbol_f`timestamp_f`month_f`date_f`datetime_f`timespan_f`minute_f`second_f`time_f!(1b; 42i; 7; 1.23e; `kdb; .z.p; 2022.01m; 2022.01.27; .z.z; 1D23:45:01.23456789; 12:34; 12:34:56; 12:34:56.789);
q)lists: 2 #/: atoms;
q)encoded: .grpc.encode[`example.Lists; lists]
q).grpc.decode[`example.Lists; encoded]
bool_f     | 11bint_f      | 42 42i
long_f     | 7 7
real_f     | 1.23 1.23e
float_f    | `float$()
symbol_f   | `kdb`kdb
timestamp_f| 2022.01.29D10:03:41.725881000 2022.01.29D10:03:41.725881000
month_f    | 2022.01 2022.01m
date_f     | 2022.01.27 2022.01.27
datetime_f | 2022.01.29T10:03:41.725 2022.01.29T10:03:41.725
timespan_f | 1D23:45:01.234567890 1D23:45:01.234567890
minute_f   | 12:34 12:34
second_f   | 12:34:56 12:34:56
time_f     | 12:34:56.789 12:34:56.789
```

## Nested Message

```q
q)encoded: .grpc.encode[`example.Outer; `out_law`inner!(1b; `inner_muscle`inner_mind!(150; `silent))];
q).grpc.decode[`example.Outer; encoded]
out_law| 1b
inner  | `inner_muscle`inner_mind!(150;`silent)
```

## Table

```q
q)processes: ([] host: `shinjuku.com`ikebukuro.com; port: 10000 12000i; running: 0D01:36:15.379632000 2D15:37:22.638791000; user: ("Daniel"; "Ezekiel")); 
q)encoded: .grpc.encode[`example.Table; enlist[`rows]!enlist processes]
q).grpc.decode[`example.Table; encoded]
rows| +`host`port`running`user!(`shinjuku.com`ikebukuro.com;10000 12000i;0D01..
q)first value .grpc.decode[`example.Table; encoded]
host          port  running              user     
--------------------------------------------------
shinjuku.com  10000 0D01:36:15.379632000 "Daniel" 
ikebukuro.com 12000 2D15:37:22.638791000 "Ezekiel"
```

## Map Example

*Note: Map field does not keep the order of keys.*

```q
q)people: `id`xday`physical!(`Joshua`Mark`John!7 2 4i; 1 2 3!1978.06 2012.08 2018.02m; 10b!(`inner_muscle`inner_mind!(3000; `blue); `inner_muscle`inner_mind!(4000; `happy)))
q)encoded: .grpc.encode[`example.Mappy; people]
q).grpc.decode[`example.Mappy; encoded]
id      | `Joshua`John`Mark!7 4 2i
xday    | 1 3 2!1978.06 2018.02 2012.08m
physical| 10b!+`inner_muscle`inner_mind!(3000 4000;`blue`happy)
```
