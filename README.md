# Wednesday world cup

Tournament betting app.

## Design idea

### Functional
Currently, not purely functional due to a borrow-checker work-around.

### Dumb backend, smart frontend.
The frontend user interface `ui` is written in the same language as the `core` library (rust), which both compile to webassembly [_wasm_](https://webassembly.org/).
This enables a trivial backend, which does no more than serve up raw data and leaves all calculations to be done in the browser.

## Project structure

![Dependency graph](assets/dep_graph.svg)

### `core`
The core library defines all the tournament types, traits and logic.

### `ui`
The user interface is la pièce de résistance! A frontend written entirely in rust (okok, there is some html and css as well but not a single line of javasript is used in this product).
It compiles to a wasm module, creating a webpage where everything is displayed.

### `db`
The database library exposes a rust interface to read and write data to a `sqlite3` database.

### `server`
The executable `wwc_server` is a very simple http server. The wasm `ui` cannot, for sand-boxing reasons, interact directly with the database. Instead, the `wwc_server` acts as a bridge to enable the `ui` to make database calls through a http api.

### `cli`
Creates executable `wwc_cli`. Convenient way of initialising the database with teams, games and betters. `wwc_cli` has no restrictions like the `ui` and can communicate directly with `db` and so have an explicit dependency on `db`.

### `data`
interface to handle external data sources.

## Setup
The frontend is built with a framework calles [Seed](https://seed-rs.org/).
It requires a nightly version of the rust compiler:

```
rustup override set nightly
```
### Database setup
requires the `diesel-cli`, get it with:
```
cargo install diesel_cli --no-default-features --features sqlite
```

```
export WWC_ROOT=$(pwd)
cd WWC_ROOT/db/
echo DATABASE_URL=test.db > .env
export DATABASE_URL=$WWC_ROOT/db/test.db
diesel setup
diesel migration run
```

Generate the dependency graph:
```
mmdc -i assets/dep_graph.mmd -o assets/dep_graph.svg
```
