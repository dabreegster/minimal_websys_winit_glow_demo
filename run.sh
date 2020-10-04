#!/bin/bash

wasm-pack build --dev --target web
cp index.html pkg
cd pkg
python3 -m http.server 8000
