#!/bin/sh

export DATABASE_URL="sqlite://$PWD/archk.db"
# export RUST_LOG=archk_api=trace,tower_http=trace