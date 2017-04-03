A web interface for running Pony code.

# Running your own Pony-Playpen

## System Requirements

Currently needs to be run on a system with access to Docker.

## Running the web server

First, create the Docker image that playpen will use:

```
docker build docker -t ponylang-playpen
```

Next, spin up the server.

```
cargo run --bin playpen
```

You should now be able to browse http://127.0.0.1:8080 and interact.
