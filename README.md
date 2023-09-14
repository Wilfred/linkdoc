# Link Doctor

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

## Known bugs

ID-relative links are treated naively. If there's a link `#foo` on a page `/bar`,
we end up querying `http://example.com/#foo` instead of
`http://example.com/bar#foo`. We should either ignore ID-relative
links or check for a matching ID on the same page. (We could even do
the same for external links, verifying that the ID still exists.)

We don't check for broken links in `<img>` tags.
