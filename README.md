# Link Doctor

[![Build Status](https://travis-ci.org/Wilfred/linkdoc.svg?branch=master)](https://travis-ci.org/Wilfred/linkdoc)

A simple Rust program for checking for dead links. Inspired by
[linkchecker](http://wummel.github.io/linkchecker/).

GPL v2 or later license.

## Usage

This project must be run with Rust nightly, due to html5ever depending
on it.

```bash
$ cargo build
$ cargo run http://www.wilfred.me.uk
```
