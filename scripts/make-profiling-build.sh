#!/usr/bin/env bash
set -euo pipefail

wasm-pack build --profiling store
