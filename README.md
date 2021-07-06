life
====

This is a fairly simple implementation of [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) and other [Life-like cellular automata](https://en.wikipedia.org/wiki/Life-like_cellular_automaton) in Rust.

## Binary
The visualization can run natively or on the web.
A working demo is live at [life.cgm616.me](https://life.cgm616.me).
While the visualization is running, the following keys can control the simulation:
- **Escape.** Open the settings pane, where you can change the rule and speed.
- **Space.**  Pause the simulation.
- **Right arrow.** Move the simulation one step forward while paused.
- **N.** Generate a new random initial state (a "soup").

## Library
The library itself (`lib.rs` and submodules) can compile and run any Life-like cellular automaton from a rule definition in the format `B_/S_`, where each `_` is a string of digits between zero to eight.
The digits after `B` define when cells are "born" (switch to alive from dead) and the digits after `S` define when cells "survive" (remain alive).
For example, Conway's Game of Life has the rule `B3/S23`.

