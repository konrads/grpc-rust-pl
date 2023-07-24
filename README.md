# gRPC playground in Rust

[![build](../../workflows/build/badge.svg)](../../actions/workflows/build.yml)

Files of interest:

- [payments.proto](proto/payments.proto) - protobuf interface, includes `service`, `message`, `optional`, `repeated`, `empty`
- [build.rs](build.rs) - needed for tonic to compile protobuf artifacts
- [server.rs](src/server.rs) - tonic implemented protobuf server
- [client.rs](src/client.rs) - clap CLI for the server
