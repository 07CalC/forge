# fyrer 

lightweight tool to run multiple dev servers concurrently


## features

- run multiple dev servers concurrently
- installer cmd for each server runs before starting the server
- specify working directory for each server
- set environment variables for each server
- easy to use YAML configuration file
- logs output of each server with name prefix
- cross-platform (Linux, macOS, Windows)
- hot reload 
- ignore specific files or directories for hot reload

## Installation

### install using cargo:
  
```bash
cargo install fyrer
```

### build from source:

```bash
git clone https://github.com/07calc/fyrer
cd fyrer
cargo build --release
cargo install --path .
```

## Usage

run from `fyrer.yml` file:

```bash
fyrer
```

example config file `fyrer.yml`:

```yaml

installers:
  - dir: ./project1
    cmd: pip install -r requirements.txt

servers:
  - name: server1
    cmd: python -m http.server 8000
    dir: ./project1
    env:
      PORT: 8000
      ENV: dev
  - name: server2
    cmd: npm start
    dir: ./project2
    watch: true # enable hot reload
    ignore: 
      - "node_modules/**"
      - "*.db"
```
