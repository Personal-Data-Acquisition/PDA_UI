# PDA_UI
The User Interface for the PDA project. Composed of a Rust WASM client built with egui (in `ui`), and a Rocket server (in `server`).

## Build Instructions
To build the WASM client:  
In `ui` run `./build`, or run `wasm-pack build -t web` and replace `server/pkg` with `ui/pkg`.

To run the server:  
In `server`, run `cargo run` and open the address provided in your browser.

## Repository Organization
This UI repository is split into two sub-projects, one each housed in the `ui` and `server` directories. The `ui` direcotry builds a front end specification in the [EGUI](https://github.com/emilk/egui?tab=readme-ov-file) library. This is compiled to a package which is consumed by the `server` directory to host the website through the [Rocket](https://rocket.rs/) web framework.

The `src` folder within `ui` defines each page in the GUI in its own file, which are all consumed by `lib.rs` to generate an EGUI app in a similar fashion to the templates provided by the library's authors. The `utils.rs` file is responsible for handling helper constructs, such as a wrapper to a value provided by the server.

The `server` folder contains two files, `main.rs` and `sql_parsing.rs`. The former is responsible for the traditional request handling expected of a web-server. This task utilizes the tools offered by `sql_parsing.rs` to access an SQLite database hosted on the root of the machine and provide values to the UI to be displayed.

## Potential Feature Enhancements
* Currently, the data displays on the home page graph points in an ascending order. The x values of these points could be updated to timestamps instead of simple relative ordering.

* The log page displays lots of data but lacks a convinent export method. An option could be developed to generate a csv from the data present on this page

* Acceleration data is powerful and through some simple calculus could be used to generate velocity and displacement data for the user

* The config panel has its function skeleton established, but there are numerious opportunities to add more functionality to this page