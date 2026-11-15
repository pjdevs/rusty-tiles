# Rusty Tiles

A small MIDI piano visualizer built with Rust and Bevy. Play a note on your MIDI keyboard and watch it scroll up the screen.

## Run

Requires Rust and Cargo.

```sh
cargo run --release
```

## MIDI keyboard

Plug or unplug your keyboard while the app is running. It detects connections and disconnections automatically, so you can start it before connecting a keyboard.

Uses the first available MIDI input.

## License

Licensed under [MIT](LICENSE.md).
