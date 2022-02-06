# Type Mapping

| q/kdb+         | protobuf              |
|----------------|-----------------------|
| bool           | bool                  |
| int            | int32/sint32          |
| long           | int64/sint64          |
| real           | float                 |
| float          | double                |
| symbol         | q.symbol              |
| timestamp      | q.timestamp           |
| month          | q.month               |
| date           | q.date                |
| datetime       | q.datetime            |
| timespan       | q.timespan            |
| minute         | q.minute              |
| second         | q.second              |
| time           | q.time                |
| bool list      | repeated bool         |
| byte list      | bytes                 |
| int list       | repeated int32/sint32 |
| long list      | repeated int64/sint64 |
| real list      | repeated float        |
| float list     | repeated double       |
| string         | string                |
| symbol list    | repeated q.symbol     |
| timestamp list | repeated q.timestamp  |
| month list     | repeated q.month      |
| date list      | repeated q.date       |
| datetime list  | repeated q.datetime   |
| timespan list  | repeated q.timespan   |
| minute list    | repeated q.minute     |
| second list    | repeated q.second     |
| time list      | repeated q.time       |
| dictionary     | message/map[*1]       |
| table          | repeated message      |

**Note:**
[*1]: In protobuf, only bool, int32, sint32, int64, sint64, string are allowed as a key type of map.
