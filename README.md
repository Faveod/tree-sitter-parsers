# Tree-Sitter Build Parsers

[![cd](https://github.com/Faveod/tree-sitter-parsers/actions/workflows/cd.yml/badge.svg)](https://github.com/Faveod/tree-sitter-parsers/actions/workflows/cd.yml)

This repository contains a script to create your favourite shared library
`tree-sitter-{parser}`.

The official repositories don't have a `Makefile` for generated parsers, so this
is like a special `Makefile`.

We also have the libraries pushed as artifacts through CI/CD. Check
[Rleseases]().

This repository was created for
[`ruby-tree-sitter`](https://github.com/Faveod/ruby-tree-sitter).
## Usage

``` console
./lang javascript
./lang java ruby php
```

## Tree-Sitter version

The script will clone `tree-sitter` and then checkout some tag (I will try to
keep it updated to latest).  But if it finds a `tree-sitter` dir, it will not do
anything to it.  **so if you want to build agains a specific tag or commit, just
`cd` into `tree-sitter` and do yout thing**.

## Artefacts

The output of `./lang LANGUAGE` is in `lib/libtree-sitter-LANGUAGE.{dylib, so}`.

# Supported Archs

## Linux

- x86_64-linux-gnu
- arm-linux-gnueabi
- arm-linux-gnueabihf
- aarch64-linux-gnu
- mipsel-linux-gnu
- powerpc64le-linux-gnu

## Macos

- x86_64-intel

NO SUPPORT FOR ARM (M1, M2, etc). Will land sometime soon, maybe. This requres a
patch to `tree-sitter`'s makefile itself.

## Windows

no.
