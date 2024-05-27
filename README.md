# PDA_UI
The User Interface for the PDA project. Composed of a Rust WASM client built with egui (in `ui`), and a Rocket server (in `server`).

## Build Instructions
To build the WASM client:  
In `ui` run `./build`, or run `wasm-pack build -t web` and replace `server/pkg` with `ui/pkg`.

To run the server:  
In `server`, run `cargo run` and open the address provided in your browser.
