# Slightly secure chat

A slightly more secure site

## How to use

Build rust
```
rustup target add wasm32-unknown-unknown            //if needed
cargo build --target wasm32-unknown-unknown --release   // if on windows
cargo build                                             // on linux - CANCEL
cargo install wasm-bindgen-cli                      // if needed
wasm-bindgen target/wasm32-unknown-unknown/release/stegano_project.wasm --out-dir .
cargo install wasm-pack                             // if needed
wasm-pack build --release --target web
```

```
$ cd code
$ npm install
$ npm start
```

And point your browser to `http://localhost:3000`. Optionally, specify
a port by supplying the `PORT` env variable.