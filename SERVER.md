# Server
The server command will start a conventional websocket broker : 
* Sender can send data using `/room/handle`
* Grabber can get data from sender grabbing from `/room/handle`

However, some improvement has been implemented :
* There can be only one `/room/handle` used by one sender at a time. If a new sender try to join `/room/handle` already in use, it will be disconnected.
* There is special entrypoint that can be used out of bonzomatic.
  * `/room` : it will receive all sender data within the selected room 
  * `/_room` : it send all user handle currently connected on the room
  * `/_`: it send all user handle currently connected on the server