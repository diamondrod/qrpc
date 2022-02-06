## @file run.sh
## @overview Launch example gRPC server.

echo "Open restaurant"
docker run --rm --name master_sheff --publish 3160:3160 restaurant:latest
echo "Close restaurant"
