# Kiro MIDI

This is a MIDI IO library that provides:

- An abstract driver to interface with the MIDI system independently of the OS.
- It works with the MIDI 2.0 protocol internally.
- Transparent handling of connected/disconnected devices based on declarative rules.
- Convenient interfaces to deal with real-time data (callbacks, ring buffers, filtering).
- No need to deal with the low level MIDI protocol as it provides a convenient representation.

***NOTE that this library is still in alpha state and will change its interface.***

You can run the example for a demo:

```shell
cargo run --example receive
```

