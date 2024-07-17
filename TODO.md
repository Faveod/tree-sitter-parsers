# TODO

## Bugs

- [ ] the application will fail if tmp/ (the output dir) is empty. I thought I fixed that

## CI

- [ ] Cross-platform builds: support all tree-sitter platforms
- [ ] Produce single parsers
- [ ] Produce linux/mac distribution packages for the parsers and tsp

## Code / Libs

- [ ] Use [rust's env](https://doc.rust-lang.org/std/env/consts/constant.DLL_EXTENSION.html)
      for DDL names instead of cfg.
- [ ] Use reqwest+rustls instead of depending on curl or wget.
  - Can't use git2 natively because it's not async by design, so I'd have to use the blocking pool of tokio.
    I don't want that.

## Configurtion

- [ ] Investigate a figment replacement / custom impl to merge diferent configuration
    sources.

## Features

- [ ] Package
  - [ ] Linux
  - [ ] Mac
  - [ ] Tar / Zip

## Options

- [ ] --sys-ts, false by default
  - [ ] Use [TREE_SITTER_LIBDIR](https://github.com/tree-sitter/tree-sitter/blob/4f97cf850535a7b23e648aba6e355caed1f10231/cli/loader/src/lib.rs#L177)
        by default
  - [ ] Use pkgconfig for sys libs

## Tests

- [ ] Figure out some cli testing framework
  - [ ] Test --force
  - [ ] Test --sys-ts
  - [ ] verify --static using `nm`
- [ ] Config
  - [ ] with default config
    - [ ] You can always download a parser even if it's not in the config.
    - [ ] Verify it's actually HEAD when the parser is not in the config using git in the test.
  - [ ] with custom config file.
    - [ ] ask for parsers defined in the config file
    - [ ] ask for parsers !defined in the config file and verify they're from the repo's HEAD.

## UX
