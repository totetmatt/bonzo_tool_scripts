# Bonzo Tool Scripts (BTS)
Set of tool scripts for bonzomatic


![alt text](./doc/imgs/main.jpg "Title")

## Recorder
```
Record an entry from a websocket entrypoint

USAGE:
    bts.exe recorder [OPTIONS] --host <HOST> --room <ROOM> <HANDLE>

ARGS:
    <HANDLE>    Handle

OPTIONS:
    -h, --help                   Print help information
        --host <HOST>            Host or Host:Port
    -p, --protocol <PROTOCOL>    Protocol [default: ws]
        --room <ROOM>            Room
    -V, --version                Print version information
```

## Replayer
```
Replay a saved entry to a websocket entrypoint

USAGE:
    bts.exe replayer [OPTIONS] --host <HOST> --room <ROOM> <HANDLE> <FILE>

ARGS:
    <HANDLE>    Handles
    <FILE>      Input Json file

OPTIONS:
    -h, --help                                 Print help information
        --host <HOST>                          Host or Host:Port
    -p, --protocol <PROTOCOL>                  Protocol [default: ws]
        --room <ROOM>                          Room
        --update-interval <UPDATE_INTERVAL>    udpateInterval (ms) [default: 300]
    -V, --version                              Print version information
```
## Todo :

- async queue for multi record (optim)
- Use client non blocking for replayer, so write and read don't interfere
- Timeline ui to control the replay

I'm learning Rust and a potato developer on this language. Feel free to PR so I can learn :)
