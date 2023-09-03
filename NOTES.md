
# NOTES
cargo install diesel_cli --no-default-features --features sqlite

## Setup
```bash
echo DATABASE_URL=zptess.db > .env
```

Make sure to specify the migration fir in diesel.toml and also
the schema file if generating in a submodule
```
[migrations_directory]
dir = "migrations/sqlite"

[print_schema]
file = "src/database/schema.rs"
```


```bash
# This will create our database (if it didn’t already exist), 
# and create an empty migrations directory that we can use to manage our schema (more on that later).
mkdir migrations
diesel setup 
```
```bash

# create a migration
diesel migration generate  <migration name>
```


```bash
# apply migration manually (not really needed)
# This is needed the firs time to build the schema
# zptess run migrations automatically and has the migrations embeded in the binary
diesel migration run 
```

diesel migration revert 
