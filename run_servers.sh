#!/bin/bash

BIN_DIR=./target/debug/

unbuffer $BIN_DIR/lock_leader 9001 10 | tee leader-1.log log &
unbuffer $BIN_DIR/lock_leader 9002 11 | tee leader-2.log log &

unbuffer $BIN_DIR/lock_acceptor 9101 20 | tee acceptor-1.log log &
unbuffer $BIN_DIR/lock_acceptor 9102 21 | tee acceptor-2.log log &
unbuffer $BIN_DIR/lock_acceptor 9103 22 | tee acceptor-3.log log &

unbuffer $BIN_DIR/lock_replica 8000 0 | tee replica-2.log log &
unbuffer $BIN_DIR/lock_replica 8001 1 | tee replica-2.log log &
