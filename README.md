# deemak

## How to run locally

1. Clone the repository

```bash
git clone https://github.com/databasedIISc/deemak.git
cd deemak
```

2. Simply run the following command to start terminal version -

Note that we need to pass the world directory as the first argument.

```bash
cargo run sekai
```

or
run the following command to start web version.

```bash
cargo run sekai --web
```

Then, open your browser and navigate to: http://localhost:8000
- To change the port, you go to .env file and change the `BACKEND_PORT` value (default BACKEND_PORT=8001).    
- To run in debug mode, you can do -

```bash
cargo run sekai --debug # OR
cargo run sekai --web --debug
```

## Contribution

Please fork the repository and make PRs to the main branch. We will review and merge them.
