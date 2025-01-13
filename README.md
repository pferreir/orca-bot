# Orcabot

This is a Mastodon bot which runs Orca source code and produces a video.

Example: https://fedi.turbofish.cc/@ubik/113822982174235813


It uses the [uxn](https://100r.co/site/uxn.html) [version of Orca](https://git.sr.ht/~rabbits/orca-toy), which is emulated thanks to the great [raven](https://github.com/mkeeter/raven/) emulator. Any ROM can be used, which means that this project could actually be repurposed to execute any other uxn ROM.

First line should mention `@orcabot` and include `#run`. The rest should be Orca code. Lines should have the same length. Maximum dimensions for the grid can be set. `= (instrument, octave, note)` can be used to play sounds.

Made by [Pedro Ferreira](https://fedi.turbofish.cc/@ubik).

Orca is a two-dimensional esoteric programming by Hundred Rabbits. Learn more about Orca on their site:
https://100r.co/site/orca.html

---

TO DO: Better deployment instructions