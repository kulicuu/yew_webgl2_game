# Yew App / Chat / Game

## Summary

This is a quick app prototype which has Websocket chat and network game functions.

For the sake of simplicity, the game is a rough clone of the 80's PC "Spacewar".  Currently there is the ship and torpedo firing.  Next will be collisions, maybe a gravitational well.

Then will hook it up to the Actix server and make a multi-player version.

## How to run the game

The basic environment requirements are the same as for a general Yew app -- that's Rust and Trunk.   
https://github.com/yewstack/yew

From project root, run `trunk serve`.  
Go to localhost:9090

**update** Or for the most current: go to localhost:9090/game_3


## How to play the game:

Use the left-arrow and right-arrow keys to rotate the spaceship.

Use the up-arrow key to fire the rocket motor to move the spaceship.

Use the spacebar to fire the torpedos.

**update**
There is a new version at localhost:9090/game_4 which is coded in components/game_404.rs.


This has two players, though the keybindings are at the moment in Dvorak for the left hand of the keyboard. 'z', 'c', 's' and 'spacebar' for the left user, and arrows and the numeric keypad '0' for the right handed user.  

In Dvorak layout, keys marked z, c, and s, are ';', 'j', and 'o'.  Until I do a proper layout for normal, this is what we have.  It's just a placeholder until networked play.  

I'm also planning the resurrection of the mouse handler, in order to use the mouse for vehicle orientation and maybe one of shooting or engine impulse.

Yesterday I was working on shader transforms but although it compiled I think it used too much memory, anyways WebGl warnings/errors occured about transform being bound to a non-transform object.

That example can be seen here. I was planning to adapt this for the vehicle explosions.  https://github.com/tsherif/webgl2examples/blob/master/particles.html






![til](./assets/sample.gif)

