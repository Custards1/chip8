# chip8
A chip8 emulator for the desktop, play all your favorite chip8 games!
# Building
*** Currently ONLY works on x86_64 linux***


You must have a copy of the development version of SDL2 >= 2.0.8 installed on your system aswell as a modern version of cargo.
# Installation
Clone this repo and open it in a terminal and run:
```
$ cargo install --path .
```
# Usage
```
USAGE:
    chip8 [OPTIONS] <game>

ARGS:
    <game>    the path to the chip8 rom to run

OPTIONS:
    -c, --cosmic     run the emulator in cosmic vip mode (default: false)
    -h, --help       Print help information
    -V, --version    Print version information
```
# Games
Don't forget to try out some games! Head on over to https://github.com/kripod/chip8-roms to download some games to play!

# Controls
Note: Not all games expect the same keypad!


```
Yours     Chip8
1 2 3 4 | 1 2 3 C
Q W E R | 4 5 6 D
A S D F | 7 8 9 E
Z X C V | A 0 B F
```
