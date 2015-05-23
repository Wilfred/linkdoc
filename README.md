# Link Doctor

A simple Rust program for checking for dead links. Inspired by
[linkchecker](http://wummel.github.io/linkchecker/).

GPL v2 or later license.

## Usage

This project must be run with Rust nightly, due to html5ever depending
on it.

```bash
$ cargo build
$ curl http://www.example.com | cargo run
```
