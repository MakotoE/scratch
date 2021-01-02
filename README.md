# Scratch From Scratch

## Dependencies
- Rust nightly
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

## Dev build
```none
wasm-pack build --dev --target web
simple-http-server --index --try-file index.html # cargo install simple-http-server
# Go to http://localhost:8000/
```
