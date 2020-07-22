#!/usr/bin/env bash
cd ..
firefox http://localhost:8000/ 2>/dev/null &
python3 -m http.server
