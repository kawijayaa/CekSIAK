#!/bin/bash

RUSTFLAGS='-Z threads=8' cargo +nightly build --release
sudo docker-compose up -d
