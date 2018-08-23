metaswitch-server
=================

A simple server that caches Nintendo Switch metacritic scores and performs smart lookups.
The score cache is updated hourly.


Getting Started
---------------

Build the project (ensure [rust & cargo](https://rustup.rs/) is installed)
    
    $ cargo build --release


Usage
-----

Run the server (starts server on port 9000)

    $ ./target/release/metaswitch-server

Run http query 

    $ curl 'http://localhost:9000/lookup?game=<game-name>'


Sample Response Payload
-----------------------

Request

    $ curl 'http://localhost:9000/lookup?game=Minecraft%3A%20Nintendo%20Switch%20Edition%20-%20Digital%20Version'

Respose

```json
{
  "name": "Minecraft: Switch Edition",
  "href": "http://www.metacritic.com/game/switch/minecraft-switch-edition",
  "score": 86,
  "score_detail": "positive",
  "user_score": 7.3,
  "user_score_detail": "mixed",
  "stem": "minecraft"
}
```
