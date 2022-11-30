# Jason: a JSON parser built with Rust

I wanted to learn more about parsing and ASTs so I built a JSON parser with Rust.

Its design is vaguely based on `rustc-lexer` and `rustc-parser`.

It's not fully compliant with the JSON spec yet, but I hope to eventually pass all the tests laid out in https://github.com/nst/JSONTestSuite.
