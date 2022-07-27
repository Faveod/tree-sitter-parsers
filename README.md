# Tree-Sitter Build Parsers

This repository contains a script to create your favourite shared library
`tree-sitter-{parser}`.

The official repositories don't have a `Makefile` for generated parsers,
so this is like a special `Makefile`.

We also have the libraries pushed as artifacts through CI/CD. Check [Rleseases]().

I created this for [`grenadier`](https://github.com/stackmystack/grenadier), the
ruby `tree-sitter` bindings.

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
