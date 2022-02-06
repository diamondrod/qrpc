## @file build.sh
## @overview Build example gRPC server container.

##cd scripts
docker build -t restaurant:latest --file scripts/Dockerfile .
