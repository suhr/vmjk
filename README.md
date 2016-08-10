# Virtual Midi Janko keyboard

![](https://ipfs.pics/ipfs/QmQMHAkrLBuARjXfPBoq46TUvy2wxvY4kmD73hiuW6Mm8o)

VMJK is a virtual midi keyboard using janko key layout, written in Rust.

## Build

To build VMJK you need CSFML and rust.

```
% cargo build --release
```

## Keybindings

![](http://i.stack.imgur.com/VJEZC.jpg)

- <kbd>\\</kbd>, <kbd>z</kbd>, <kbd>x</kbd>, <kbd>c</kbd>... → C, D, E, F♯... 
- <kbd>\\</kbd>, <kbd>a</kbd>, <kbd>w</kbd>, <kbd>3</kbd>... → C, C♯, D, D♯..
- <kbd>Space</kbd> switches octaves: (C4) <kbd>Space</kbd> → (C3) <kbd>Space</kbd> → (C4)
- <kbd>Enter</kbd> releases all notes
