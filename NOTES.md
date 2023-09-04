
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


## TESS Photometer features

| Model   |      readings transport     |  readings payload  |   info transport   |  info payload  | 
|---------|:---------------------------:|-------------------:|:------------------:|:---------------|
| TESS-W  | udp:2255 or serial:9600 (1) | JSON or propietary | HTTP:80            | HTML           |
| TESS-P  | serial:9600                 | JSON               | serial             | propietary (2) |
| TAS     | serial:9600                 | JSON               | serial             | propietary (2) |
| TESS4C  | udp:2255                    | JSON               | HTTP:80            | HTML           |

*Notes*:
1. Only for selected photometers (i.e. the reference photometer)
2. Comand/Response protocol with custom text