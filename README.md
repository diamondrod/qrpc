# qRPC

gRPC client for q/kdb+.

## Introduction

q/kdb+ implements propriate protocol to communicate among q processes and HTTP(1) protocol. There are some client library for q which implements the propriate protocol but t is not suitable to integrate in a large system. On the other hand it is rare that HTTP becomes a main protocol in this age.

When your processes have limited number of message types and furthermore you want to check the validity of contents, HTTP is not suitable for it. Then gRPC came to play, which is based on HTTP2 and afford to specify solid message types.

Users of q/kdb+ might have moarned that they could not use gRPC in q and used HTTP without any options hacking erroneous messages leaning deeply against a chair-back. This library is a gRPC client implementation of q to pour water ("of life", if you will) to such a people.

## table of Contents

1. Type Mapping
2. [Examples](docs/examples.md)
