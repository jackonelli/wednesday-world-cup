# Wednesday world cup

Tournament betting app.

## Setup and build

To get the full app up and running, you need to have

- an installation of `rust` and `cargo`
- an initialised database
- a running `server`
- hosting of the to-wasm-compiled UI.

### Backend setup

The backend consists of the tightly linked `server` and `db` crates. The `db` crate is a pure lib. It provides rust bindings to a `sqlite3` database containing the raw data for the application (teams, games, betters et c.).
The `server` is an executable which needs to be running whenever the application is active. It listens for http requests and responds with or manipulates database data.

First, setup the database, see [`db/README.md`](db/README.md)
With the database set up, the only remaining backend thing is to start the server.

### Create a user
JWT_SECRET=<secret-key> cargo run --bin wwc_cli user create <username> <password> <player_id> "<display_name>"

# Create a bot for a user (outputs JWT token)
JWT_SECRET=<secret-key> cargo run --bin wwc_cli bot create <username> <bot_name> "<bot_display_name>"

# List users/bots
cargo run --bin wwc_cli user list
cargo run --bin wwc_cli bot list <username>

# Start server
JWT_SECRET=<secret-key> cargo run --bin wwc_server

```bash
cd $WWC_ROOT
cargo run --bin wwc_server
```

### UI setup

The UI is a webpage, hosted with some generic web server. The final UI is in html, css and javascript, but this is all generated from rust source code, found in the `ui` crate. We use a special build program to generate WASM from rust.
Specifically, the rust code in `ui` is written with a web framework called [Leptos](https://leptos.dev/), which looks even stranger than normal rust since it uses macros to generate the html.

#### WASM compilation

Install [`trunk`](https://trunkrs.dev/).

For development use `trunk serve`, this will host and automatically rebuild the UI when the source code changes.
```bash
cd $WWC_ROOT/ui
trunk serve
```

## Docs

The `core` crate is reasonably well documented, while the remaining are not.
Adding documentation is a great first issue!

Generate and open documentation:

```bash
cd $WWC_ROOT
cargo doc --workspace --no-deps --document-private-items --open
```

Generate the dependency graph figure (requires [Mermaid-CLI](https://github.com/mermaid-js/mermaid-cli)):

```bash
cd $WWC_ROOT
mmdc -i assets/dep_graph.mmd -o assets/dep_graph.svg
```
