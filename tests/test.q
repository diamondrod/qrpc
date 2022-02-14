/
* @file test.q
* @overview Test of qrpc.
* @note gRPC server must be launched before running this script.
* ```sh
* restaurant]$ cargo run
* ```
\

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Initial Settings
//++++++++++++++++++++++++++++++++++++++++++++++++++//

// Load test helper functions.
\l tests/test_helper_function.q

// Load qrpc library
\l q/grpc.q

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Tests
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% Atom %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

atoms: `bool_f`int_f`long_f`real_f`symbol_f`timestamp_f`month_f`date_f`datetime_f`timespan_f`minute_f`second_f`time_f!(1b; 42i; 7; 1.23e; `kdb; .z.p; 2022.01m; 2022.01.27; .z.z; 1D23:45:01.23456789; 12:34; 12:34:56; 12:34:56.789);
wrong_atoms: `bool_f`int_f`long_f`real_f`symbol_f`timestamp_f`month_f`date_f`datetime_f`timespan_f`minute_f`second_f`time_f!(1b; 42h; 7; 1.23e; `kdb; .z.p; 2022.01m; 2022.01.27; .z.z; 1D23:45:01.23456789; 12:34; 12:34:56; 12:34:56.789);

.test.ASSERT_ERROR["non-existing message for encode"; .grpc.encode; (`example.Something; atoms); "no such message"]
.test.ASSERT_ERROR["wrong type"; .grpc.encode; (`example.Atoms; wrong_atoms); "not an int"]

encoded: .grpc.encode[`example.Atoms; atoms];
.test.ASSERT_ERROR["non-existing message for decode"; .grpc.decode; (`example.Something; encoded); "no such message"]
.test.ASSERT_EQ["atom"; .grpc.decode[`example.Atoms; encoded]; atoms]

only_bool: `bool1`bool2!10b;
encoded: .grpc.encode[`example.OnlyBool; only_bool];
.test.ASSERT_EQ["only bool"; .grpc.decode[`example.OnlyBool; encoded]; enlist[`bool1]!enlist 1b]

only_int: `int1`int2!3 4i;
encoded: .grpc.encode[`example.OnlyInt; only_int];
.test.ASSERT_EQ["only int"; .grpc.decode[`example.OnlyInt; encoded]; only_int]

only_long: `long1`long2!1 2;
encoded: .grpc.encode[`example.OnlyLong; only_long];
.test.ASSERT_EQ["only long"; .grpc.decode[`example.OnlyLong; encoded]; only_long]

only_real: `real1`real2!1.23 4.56e;
encoded: .grpc.encode[`example.OnlyReal; only_real];
.test.ASSERT_EQ["only real"; .grpc.decode[`example.OnlyReal; encoded]; only_real]

only_float: `float1`float2!1.2 3.456;
encoded: .grpc.encode[`example.OnlyFloat; only_float];
.test.ASSERT_EQ["only float"; .grpc.decode[`example.OnlyFloat; encoded]; only_float]

only_symbol: `symbol1`symbol2!`a`b;
encoded: .grpc.encode[`example.OnlySymbol; only_symbol];
.test.ASSERT_EQ["only symbol"; .grpc.decode[`example.OnlySymbol; encoded]; only_symbol]

only_timestamp: `timestamp1`timestamp2!`timestamp$123456789 234567890;
encoded: .grpc.encode[`example.OnlyTimestamp; only_timestamp];
.test.ASSERT_EQ["only timestamp"; .grpc.decode[`example.OnlyTimestamp; encoded]; only_timestamp]

only_month: `month1`month2!2000.03 2015.12m;
encoded: .grpc.encode[`example.OnlyMonth; only_month];
.test.ASSERT_EQ["only month"; .grpc.decode[`example.OnlyMonth; encoded]; only_month]

only_date: `date1`date2!2012.12.25 2021.03.08;
encoded: .grpc.encode[`example.OnlyDate; only_date];
.test.ASSERT_EQ["only date"; .grpc.decode[`example.OnlyDate; encoded]; only_date]

only_datetime: `datetime1`datetime2!2013.09.18T12:34:56.789 2018.02.03T12:34:56.789;
encoded: .grpc.encode[`example.OnlyDatetime; only_datetime];
.test.ASSERT_EQ["only datetime"; .grpc.decode[`example.OnlyDatetime; encoded]; only_datetime]

only_timespan: `timespan1`timespan2!`timespan$123456789 234567890;
encoded: .grpc.encode[`example.OnlyTimespan; only_timespan];
.test.ASSERT_EQ["only timespan"; .grpc.decode[`example.OnlyTimespan; encoded]; only_timespan]

only_minute: `minute1`minute2!00:03 21:08;
encoded: .grpc.encode[`example.OnlyMinute; only_minute];
.test.ASSERT_EQ["only minute"; .grpc.decode[`example.OnlyMinute; encoded]; only_minute]

only_second: `second1`second2!00:12:34 15:30:02;
encoded: .grpc.encode[`example.OnlySecond; only_second];
.test.ASSERT_EQ["only datetime"; .grpc.decode[`example.OnlySecond; encoded]; only_second]

only_time: `time1`time2!12:34:56.789 06:09:46.029;
encoded: .grpc.encode[`example.OnlyTime; only_time];
.test.ASSERT_EQ["only time"; .grpc.decode[`example.OnlyTime; encoded]; only_time]

//%% List %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

lists: 2 #/: atoms;
encoded: .grpc.encode[`example.Lists; lists];
.test.ASSERT_EQ["list"; .grpc.decode[`example.Lists; encoded]; lists]

//%% Table %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

processes: ([] host: `shinjuku.com`ikebukuro.com; port: 10000 12000i; running: 0D01:36:15.379632000 2D15:37:22.638791000; user: ("Daniel"; "Ezekiel")); 
encoded: .grpc.encode[`example.Table; enlist[`rows]!enlist processes];
.test.ASSERT_EQ["table"; .grpc.decode[`example.Table; encoded] `rows; processes]

//%% Map %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

sort_dictionary: {[map] (asc key map)!map asc key map};
people: `id`xday`physical!(`Joshua`Mark`John!7 2 4i; 1 2 3!1978.06 2012.08 2018.02m; 10b!(`inner_muscle`inner_mind!(3000; `blue); `inner_muscle`inner_mind!(4000; `happy)));
encoded: .grpc.encode[`example.Mappy; people];
decoded: .grpc.decode[`example.Mappy; encoded];
.test.ASSERT_EQ["map - symbol"; sort_dictionary decoded `id; sort_dictionary people `id]
.test.ASSERT_EQ["map - long"; sort_dictionary decoded `xday; sort_dictionary people `xday]
.test.ASSERT_EQ["map - bool"; sort_dictionary decoded `physical; sort_dictionary people `physical]

//%% OneOf %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

encoded: .grpc.encode[`example.OneOf; `static`int_f!(1b; 42i)];
.test.ASSERT_EQ["oneof - int"; .grpc.decode[`example.OneOf; encoded]; `static`int_f!(1b; 42i)]

encoded: .grpc.encode[`example.OneOf; `static`month_f!(1b; 2022.02m)];
.test.ASSERT_EQ["oneof - month"; .grpc.decode[`example.OneOf; encoded]; `static`month_f!(1b; 2022.02m)]

encoded: .grpc.encode[`example.OneOf; `static`symbol_f!(1b; `strong)];
.test.ASSERT_EQ["oneof - symbol"; .grpc.decode[`example.OneOf; encoded]; `static`symbol_f!(1b; `strong)]

//%% Enum &&//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

encoded: .grpc.encode[`example.Basket; `desserts`price`snack!(`fruit$`apple`banana; 103.2; `vegetable$`tomato)];
.test.ASSERT_EQ["enum"; .grpc.decode[`example.Basket; encoded]; `desserts`price`snack!(`fruit$`apple`banana; 103.2; `vegetable$`tomato)]

//%% gRPC %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

.test.ASSERT_EQ["endpoint"; .grpc.set_endpoint["http://localhost:3160"]; "endpoint was set"];
.test.ASSERT_EQ["order1"; .grpc.submit[`table`items`ordered_time!(2i; `Menu$`pizza`coke`pizza`sushi; 2000.02.01D12:00:30.123456)]; enlist[`accepted]!enlist 1b]
.test.ASSERT_EQ["order2"; .grpc.submit[`table`items`ordered_time!(2i; `Menu$`steak`coke`sushi; 2000.02.01D12:00:40.123456)];  enlist[`accepted]!enlist 1b]
.test.ASSERT_EQ["order3"; .grpc.submit[`table`items`ordered_time!(2i; `Menu$`steak`steak`chips`coke`spaghetti`hamburger`chips`salad`pizza`sushi; 2000.02.01D12:05:30.123456)]; enlist[`reason]!enlist "too many items. must be less than 10"]
.test.ASSERT_ERROR["order - error"; .grpc.cancel; enlist `table`items`ordered_time!(3i; `Menu$`sushi`pizza`pizza; .z.p); "no order for the table id: 3"]
.test.ASSERT_EQ["cancel"; .grpc.cancel[`table`items`ordered_time!(2i; `Menu$`sushi`pizza`pizza; .z.p)]; (::)]

receipt: .grpc.finish[enlist[`table]!enlist 2i]
history: ([] time: 2000.02.01D12:00:30.123456 2000.02.01D12:00:40.123456 2000.02.01D12:00:40.123456 2000.02.01D12:00:40.123456; item: `Menu$`coke`steak`coke`sushi; unit: 4#1; price: 2 9.25 2 10e);
.test.ASSERT_EQ["finish - history"; `time`item xasc receipt `history; `time`item xasc history]
.test.ASSERT_EQ["finish - total"; receipt `total; 23.25e]

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Result
//++++++++++++++++++++++++++++++++++++++++++++++++++//

.test.DISPLAY_RESULT[];
