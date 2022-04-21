# Pre-requisites 
Download latest version here : [https://github.com/totetmatt/bonzo_tool_scripts/releases](https://github.com/totetmatt/bonzo_tool_scripts/releases)

Unzip the `bts` binary in the same directory as your Bonzomatic.

```
/some_directory
    |- Bonzomatic.exe
    |- bts.exe
```

*It can be unzipped anywhere, but I would strongly recommend to put bts binary at the same place where bonzomatic is*

Open a Terminal and go to the directory has been `bts` unziped.

# Record bonzomatic localy
Use terminal to run command
```bts bonzo-record --handle your_handle```

This command will run a local server and a bonzomatic sender instance that will connect to this local server.

The local server will store then shaders within the `./shaders` directory

- `./shaders/*.glsl` are last .glsl sent by Bonzomatic
- `./shaders/*.json` are dumps of sessions that can be replayed with `bts`

When finished you can close Bonzomatic and kill the server with `ctrl + c` or any process killing method

# Replay a bonzomatic session localy

When you recored, yo ucan replay the session via the .json file generated.

(If you don't have one, you can have a demo one here: https://raw.githubusercontent.com/totetmatt/bonzo_tool_scripts/master/demo/recorder_totetmatt_1643391091653.json )

```bts bonzo-replay  PATH_TO_JSON_FILE```

You can change the speed for the reaply using ```--update-interval```

```bts bonzo-replay  --update-interval 10 PATH_TO_JSON_FILE ```


When finished you can close Bonzomatic and kill the server with `ctrl + c` or any process killing method