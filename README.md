# YoukeBox-Rust

This is the YoukeBox backend, written in rust.

## What is `The YoukeBox`?

The YoukeBox is an application that multiple people can use to build music playlists.
Users can create a music-room and submit songs from youtube to that room. 

## Routes

Each route has a prefix: "/api/$version/"

**GET**

* /playlist 
    * Display the global playlist
* /playlist/<room>
    * Display the playlist for <room>
* /youtube/<query>
    * Search songs on youtube
* /rooms
    * Display all the rooms
* /rooms/search/<query>
    * Display the rooms with a filter

**POST**

* /playlist
    * Add a song to the global playlist
* /playlist/<room>
    * Add a song to the playlist for <room>
* /playlist/<room>/skip
    * Skip a song in <room>
* /rooms
    * Add a new room

## What does the YoukeBox support atm?

* Pseudo live-streaming
* Creating rooms
* Skipping songs

## Coming soon

* Music ordering
* Accounts support with room administrators