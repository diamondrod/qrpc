/
@file grpc.q
@overview Define gRPC interface for q.
\

/
@brief Encode q dictionary to protobuf encoded bytes.
@param `message_type`: Message type with package name prefix, e.g., `example.Scalar`.
@param `data`: q dictionary.
@example
```
q)encoded: .grpc.encode[`example.Scalar; `bool_f`int_f`long_f`real_f`symbol_f`timestamp_f`month_f`date_f`datetime_f`timespan_f`minute_f`second_f`time_f!(1b; 42i; 7; 1.23e; `kdb; .z.p; 2022.01m; 2022.01.27; .z.z; 1D23:45:01.23456789; 12:34; 12:34:56; 12:34:56.789)]
```
\
.grpc.encode: `libqrpc 2: (`encode; 2);

/
@brief Decode protobuf encoded bytes to q dictionary.
@param `message_type`: Message type with package name prefix, e.g., `example.Scalar`.
@param `bytes`: Protobuf encoded bytes.
@example
```
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
\
.grpc.decode: `libqrpc 2: (`decode; 2);
