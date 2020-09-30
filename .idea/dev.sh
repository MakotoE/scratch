#!/usr/bin/env bash
cd ..
firefox http://localhost:8000/ 2>/dev/null &
simple-http-server --index --nocache --try-file index.html
