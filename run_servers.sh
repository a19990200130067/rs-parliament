#!/bin/bash

BIN_DIR=./target/debug/

$BIN_DIR/lock_leader 9001 10 &
$BIN_DIR/lock_leader 9002 11 &

$BIN_DIR/lock_acceptor 9101 20 &
$BIN_DIR/lock_acceptor 9102 21 &
$BIN_DIR/lock_acceptor 9103 22 &

$BIN_DIR/lock_replica 8000 0 &
$BIN_DIR/lock_replica 8001 1 &
