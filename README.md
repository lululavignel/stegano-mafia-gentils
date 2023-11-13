# Slightly secure chat

A slightly more secure site

## How to use

Build rust
```
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/stegano_project.wasm --out-dir .
wasm-pack build --release --target web
```

```
$ cd code
$ npm install
$ npm start
```

And point your browser to `http://localhost:3000`. Optionally, specify
a port by supplying the `PORT` env variable.