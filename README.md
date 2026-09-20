# discord-fend
A discord bot for the arbitrary-precision unit-aware calculator [fend](https://github.com/printfn/fend)

<img width="409" height="279" alt="image" src="https://github.com/user-attachments/assets/b1f3a080-8bd9-4c8e-b889-f48db62580bd" />

Read fend's [manual](https://printfn.github.io/fend/documentation/) for detailed usage information

## Installation
Currently, the only permission required is `Send Messages` for the basic command functionality.

A public instance is hosted on @LFS6502's Raspberry Pi, which can be installed to a guild via
[this link](https://discord.com/oauth2/authorize?client_id=1538715574467559564).

## Features
The features that are working currently, and that I hope to get working later:
- [x] ANSI color highlighting
- [x] Interactive REPL sessions
- [x] Context retention between steps of multi-line prompts, and multi-message REPL sessions
- [x] Temporary context retention between prompts in a channel
- [x] Edit listening
- [x] Slash commands
- [ ] Exchange rates
- [ ] Time and date

## Running locally
Copy `.env.example` to `.env` and fill the fields, then simply run it via `cargo run --release`.

Note: Run with `--features debug` to enable debug printing.
(This is mostly used in development, and might not have much useful information)

