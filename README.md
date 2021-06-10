# Wednesday world cup

Tournament betting app.

## Project structure

![Dependency graph](assets/dep_graph.svg)

### `core`

The core library defines all the tournament types, traits and logic.

### `ui`

The user interface is la pièce de résistance! A frontend written entirely in rust (okok, there is some html and css as well but not a single line of javasript is used in this product).
It compiles to a wasm module, creating a webpage where everything is displayed.
It is built with a framework called [Seed](https://seed-rs.org/).

### `db`

The database library exposes a rust interface to read and write data to a `sqlite3` database.

### `server`

The executable `wwc_server` is a very simple http server. The wasm `ui` cannot, for sand-boxing reasons, interact directly with the database.
Instead, the `wwc_server` acts as a bridge to enable the `ui` to make database calls through a http api.
The intention is to have an as ~stupid~ simple as possible combination of `db` and `server` and leave the complexity for the UI.
This means storing a raw, basic representation of the data in the database and having the server provide access to it as is.
The data will then be deserialized into more complex structures, directly in the UI.

### `cli`

Creates a command line interface `wwc_cli`. Convenient way of initialising the database with teams, games and betters.
The CLI has no restrictions like the `ui` and can communicate directly with `db` and so have an explicit dependency on `db`.

### `data`

interface to handle external data sources.

## Setup and build

To get the full app up and running, you need to have

- an installation of `rust` and `cargo`
- an initialised database
- a running `server`
- hosting of the to-wasm-compiled UI.

### Backend setup

The backend consists of the tightly linked `server` and `db` crates. The `db` crate is a pure lib and it provides rust bindings to a `sqlite3` database containing the raw data for the application (teams, games, betters et c.). The `server` is an executable which needs to be running whenever the application is active. It listens for http requests and responds with database data.

First, we setup the database.
This requires the `diesel-cli`, get it with:

```bash
# This requires an install sqlite lib on your system.
cargo install diesel_cli --no-default-features --features sqlite
# Optionally, install with bundled sqlite c-lib. Path of least resistance for Windows users
cargo install diesel_cli --no-default-features --features "sqlite-bundled"
```

Set environment variable `DATABASE_URL=<path_to_db>`, perhaps like this:

```bash
# Preferably in a ".env" file.
export WWC_ROOT=$(pwd)
export DATABASE_URL=$WWC_ROOT/<path_to_db>
```

The variable `WWC_ROOT` is not strictly necessary, but I will use it here to reference the repo root.
To create and fill the db with data, run:

```bash
cd $WWC_ROOT/db
diesel setup
cd $WWC_ROOT
cargo run --bin wwc_cli add all
```

Now, the database is set up and the only remaining thing is to start the server.
The server expects a config file `Rocket.toml` in the repo root.
An actual config is placed in `server/Rocket.toml`, which is symlinked to the repo root.
If there is an issue with the symlinking, simply copy the actual file from `server/` to the repo root.

```bash
cd $WWC_ROOT
cargo run --bin wwc_server
```

### UI setup

The UI is a webpage that we host with a generic server program. The final UI is in html, css and javascript, but this is all generated from rust source code, found in the `ui` crate. We use a special build program to generate 'web assembly _WASM_' from rust.
Specifically, the rust code in `ui` is written with a web framework called [Seed](https://seed-rs.org/), which looks even stranger than normal rust since it uses macros to generate the html.

#### Local hosting

Any webserver will do. I find [`miniserve`](https://github.com/svenstaro/miniserve) to be very convenient when developing.
Download the executable or simply install with `cargo install miniserve`

#### WASM compilation

Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/#).
Again, some pre-built binaries are provided but the `cargo install` option works just as well.

```bash
cd $WWC_ROOT/ui
wasm-pack build --target web --out-name wwc_ui --dev
# host the `ui` directory, e.g. with
miniserve --index index.html $WWC_ROOT/ui
```

### Optional

Install [`cargo-make`](https://github.com/sagiegurari/cargo-make#installation).
The repo contains some `Makefile.toml`s, which defines some long and tedious build commands which can be accessed with `cargo-make`.
I don't love `cargo-make` but it is kind of helpful to document all the build commands.

## Docs

Generate and open documentation:

```bash
cd $WWC_ROOT
cargo doc --workspace --no-deps --document-private-items --open
```

Generate the dependency graph:

```bash
cd $WWC_ROOT
mmdc -i assets/dep_graph.mmd -o assets/dep_graph.svg
```

## Design ideas

### Dumb backend, smart frontend.

The frontend user interface `ui` is written in the same language as the `core` library (rust), which both compile to webassembly [_wasm_](https://webassembly.org/).
This enables a trivial backend, which does no more than serve up raw data and leaves all calculations to be done in the browser.

### Type safety

Many of the data in this lib: goals scored, number of games, team rank, group point, et c, are in principle (non-negative) integers.
Having many data types with shared representation but wildly differing semantics is to beg for bugs.
To avoid that -- and in fact to make this class of bugs unrepresentable -- this lib consistently implements types with the newtype pattern, e.g.

```rust
#derive[..., Add, ...]
pub struct GoalCount(u32);
```

It is more verbose to implement in the first place but when it's in place it's ergonomic and hard to misuse.

One particularly nice consequence is that the newtype pattern opts out of all the trait implementations of the wrapped type,
i.e. the newtype does not "inherit" any functionality from the inner type.
In the `GoalCount` example above, the new type does not have any functionality that we associate with an unsigned integer, like arithmetic, unless we specifically enable it. Enabling requires a trait implementation for the desired functionality, either through a manual implementation or with a derive macro.

Above you see how the `Add` trait (i.e. enabling the `+` operator) is auto-impl. with the [`derive_more`](https://crates.io/crates/derive_more) crate
(NB. the auto impl _only_ allows for addition where both values are of type `GoalCount`).
The `Sub` trait on the other hand is manually implemented to reflect the fact that the difference between two `GoalCount`'s is not a `GoalCount` but a `GoalDiff`:

```rust
impl Sub for GoalCount {
    type Output = GoalDiff;
    fn sub(self, other: Self) -> Self::Output {
        GoalDiff(self.0 as i32 - other.0 as i32)
    }
}
```

Similarly, addition for the type `TeamRank` has no semantic meaning and subsequently does not impl `Add` with any types.
Trying to add two `TeamRank` values will result in a compilation error, catching potential logic bugs which would have gone undetected if everything was of the standard integer type.

### Parse, don't validate

Another benefit of the [Type safety design](#type-safety) is that we can realise the [parse, don't validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/) idea.
Rust is a stickler for error handling: potential errors must be dealt with explicitly.

Consider the score of a playoff game. It can be represented by two non-negative integers, except that some combinations of integers would be invalid since a playoff game must have a winner.

Every function that takes such a score tuple as input, would require validation of the score, returning an error when it encounters a draw result. This pollutes the API and failure to perform this validation could cause weird results (not really for this example due to rust's inherent type safety but for more complex examples it certainly would).

Instead of validating a general tuple we use the newtype pattern to create

```rust
# NB. This is a simplified version of the actual type in the `core` crate
pub struct PlayoffScore(GoalCount, GoalCount);

impl PlayoffScore {
    pub fn try_new(home: GoalCount, away: GoalCount) -> Result<Self, PlayoffError> {
        if home == away {
            Err(PlayoffError::NoWinner)
        } else {
            Ok(PlayoffScore(home, away))
        }
    }
}
```

By making the inner types private (by not adding public identifiers: `PlayoffScore(pub GoalCount, pub GoalCount)`), the only way to construct a `PlayoffScore` is via the `try_new` constructor which _parses_ the goal counts into a validated score.
Every function that deals with playoff scores can now take input of the `PlayoffScore` type and not have to worry about equal results and the API is spared `Result<T, E>` return types in every function.

### Functional

Currently, not purely functional due to a borrow-checker work-around.
