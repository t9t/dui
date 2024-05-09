#!/bin/bash

set -eu

THEPATH="${1}"

cargo run "${THEPATH}"
du -shl -B1 --apparent-size "${THEPATH}"
