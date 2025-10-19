# UI - Seed + Trunk

## Prerequisites

Install Trunk:
```bash
cargo install trunk
```

## Development

Build and serve with hot-reloading:
```bash
trunk serve
```

The application will be available at http://127.0.0.1:8080

## Production Build

Build optimized release version:
```bash
trunk build --release
```

Output will be in the `dist/` directory.
