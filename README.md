# Build your own Programmatic Incremental Build System

A programming tutorial on building your own programmatic incremental build system in Rust, aiming to teach the concepts of programmatic incremental build systems.
Live hosted version at: <https://gohla.github.io/pibs/>

## Requirements

Install mdBook and several preprocessors:

```shell
cargo install mdbook mdbook-admonish mdbook-external-links
cargo install --path mdbook-diff2html
```

If you have [`cargo install-update`](https://github.com/nabijaczleweli/cargo-update) installed, you can instead install and/or update the external binaries with:

```shell
cargo install-update mdbook mdbook-admonish mdbook-external-links
```

## Building

To test all the code fragments and generate outputs in `src/gen` which the tutorial uses, first run:

```shell
cargo run --bin stepper
```

Then, to build the tutorial once, run:

```shell
mdbook build
```

Or to interactively build, run:

```shell
mdbook serve
```

## Generate source code

To generate all source code into `dst`, run:

```shell
cd stepper
cargo run --bin stepper -- step-all -d dst --skip-cargo --skip-outputs
```

## Stack & Structure

The book is built with [mdBook](https://rust-lang.github.io/mdBook/).
We use the following external mdBook preprocessors:

- [mdbook-admonish](https://github.com/tommilligan/mdbook-admonish)
- [mdbook-external-links](https://github.com/jonahgoldwastaken/mdbook-external-links)

Structure:

- `book.toml`: main mdBook configuration file.
- `src`: book source code.
  - `src/SUMMARY.md`: main mdBook file with the table of contents for the book.
  - `src/gen`: generated diffs and cargo outputs for the book. Part of `src` for change detection.
  - `src/*`: markdown files and code (diff) fragments of the book.
- `theme`: customization of the default theme
- `mdbook-admonish.css`: CSS for the `mdbook-admonish` preprocessor. Can be updated by running `mdbook-admonish install`.
- `stepper`: command-line application (in Rust) that checks all source code (additions, insertions, diffs) by stepping over them in order and building them with cargo, ensuring that the code in the book is actually valid. It also generates diffs between source code fragments and produces outputs (such as cargo stdout) and stores them in `src/gen`.
  - `stepper/src/app.rs`: stepper instructions. Modify this to modify what/how the source code fragments of the book are checked.
- `mdbook-diff2html`: mdBook preprocessor that renders diffs with [Diff2Html](https://github.com/rtfpessoa/diff2html)
- `diff2html-ui-base.min.js`: Diff2Html browser-side implementation
- `diff2html.min.css`: Diff2Html CSS (customized, see below)

### Diff2Html

Modifications to get it working:

- Initialize the default mdBook [theme](https://rust-lang.github.io/mdBook/format/theme/index.html) into `theme`.
- Replace highlight.js with the newest version, [11.8.0](https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/highlight.min.js) at the time of writing.
  - Place that file in `theme/highlight.js`.
  - I did not replace the highlight.js theme (which would go in the `theme/highlight.css` file), as it seems to be working.
  - See [this page](https://cdnjs.com/libraries/highlight.js) for version specific downloads.
- Install Diff2Html JS and CSS files
  - [Download diff2html-ui-base.min.js](https://cdn.jsdelivr.net/npm/diff2html@3.4.42/bundles/js/diff2html-ui-base.min.js) into `src/diff2html-ui-base-.min.js`.
  - [Download diff2html.min.css](https://cdn.jsdelivr.net/npm/diff2html@3.4.42/bundles/css/diff2html.min.css) into `src/diff2html.min.css`.
  - Added those to custom JS and CSS files in `book.toml`.
- Styling modifications
  - Remove things that we don't need to override: favicon, fonts.
  - Copy `ayu-highlight.css` into `theme/ayu-highlight.css`.
  - Modify files to change and fix styling. Changes are denoted with `CHANGE`.
  - Note that we're modifying some generated files, so updating will be difficult.
