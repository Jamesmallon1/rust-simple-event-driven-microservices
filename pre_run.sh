#!/bin/bash

# start the zookeeper server
zookeeper-server-start /usr/local/etc/kafka/zookeeper.properties &
ZOO_PID=$!

# start kafka in background
kafka-server-start /usr/local/etc/kafka/server.properties


# kill zookeeper following kafka shutdown
kill $ZOO_PID