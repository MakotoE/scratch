[Try scratch-vm-rs](https://makotoe.github.io/scratch-vm-rs/), a WIP Rust implementation of [scratch-vm](https://github.com/LLK/scratch-vm/).

# Dependencies
- Rust nightly
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

# Dev build
```none
wasm-pack build --dev --target web
python3 -m http.server
# Go to http://localhost:8000/
```
