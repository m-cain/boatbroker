# Boat Broker

A web application for a Boat Brokerage.

## Getting Started

```sh
cargo install --path .

# dev tools
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
pre-commit install
```

## Working with the Database

```sh
cargo install sqlx-cli --no-default-features --features native-tls,postgres
sqlx database create
sqlx migrate run
cargo sqlx prepare
```
