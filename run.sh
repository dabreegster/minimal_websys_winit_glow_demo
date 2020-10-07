#!/bin/bash

set -e
wasm-pack build --dev --target web
cp index.html black_humor.txt pkg
cd pkg
python3 -m http.server 8000
