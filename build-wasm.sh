#!/bin/sh

cargo install multiversx-sc-meta

TARGET_DIR=$PWD/target

sc-meta all build --target-dir $TARGET_DIR
