#!/bin/sh
mkdir -p headers
cbindgen --lang c  --crate zstd_read_line --output headers/zstd_read_line.h
