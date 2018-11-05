#!/bin/bash

BIN_DIR=./target/debug/

unbuffer $BIN_DIR/lock_server leader 0 | tee leader-1.log all.log &
unbuffer $BIN_DIR/lock_server leader 1 | tee leader-2.log all.log &

unbuffer $BIN_DIR/lock_server acceptor 0 | tee acceptor-1.log all.log &
unbuffer $BIN_DIR/lock_server acceptor 1 | tee acceptor-2.log all.log &
unbuffer $BIN_DIR/lock_server acceptor 2 | tee acceptor-3.log all.log &

unbuffer $BIN_DIR/lock_server replica 0 | tee replica-2.log all.log &
unbuffer $BIN_DIR/lock_server replica 1 | tee replica-2.log all.log &
