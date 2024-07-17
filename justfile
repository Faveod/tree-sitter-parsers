#!/usr/bin/env -S just --justfile
alias t := test
alias c := check
alias b := build
alias w := watch
alias r := run

build *args:
  cargo build {{args}}

check: fmt clippy test
  #!/usr/bin/env bash
  set -euxo pipefail
  git diff --no-ext-diff --quiet --exit-code
  VERSION=`sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`
  grep "^\[$VERSION\]" CHANGELOG.md

ci: lint
  cargo test --all

clippy:
  cargo clippy --all --all-targets -- --deny warnings

clippy-fix *args:
  cargo clippy --fix {{args}}

clippy-fix-now:
  @just clippy-fix --allow-dirty --allow-staged

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

lint: clippy fmt-check

log:
  #!/usr/bin/env bash
  latest=$$(sed -rne '/^## tsp v[0-9]+\.[0-9]+-[0-9]+-g/{s///;s/ \(.*//;p;q;}' NEWS.md); \
  git log --reverse "$latest..HEAD"

notes:
  grep '<sup>master</sup>' README.md

outdated:
  cargo outdated -R

publish:
  #!/usr/bin/env bash
  set -euxo pipefail
  rm -rf tmp/release
  git clone # TODO
  cd tmp/release
  ! grep '<sup>master</sup>' README.md
  VERSION=`sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`
  git tag -a $VERSION -m "Release $VERSION"
  git push origin $VERSION
  cargo publish
  cd ../..
  rm -rf tmp/release

run:
  cargo run

setup:
  rustup component add clippy
  cargo install cargo-check cargo-edit cargo-outdated cargo-watch

test *args:
  cargo test {{args}}

update-changelog:
  echo >> CHANGELOG.md
  git log --pretty='format:- %s' >> CHANGELOG.md

upgrade-check:
  cargo upgrade --verbose --dry-run

upgrade:
  carfo upgrade --verbose

contributors:
  git shortlog -sne | sed -r 's/^[[:space:]]*[0-9]+[[:space:]]*//' | sort > Contributors

watch +args='test':
  cargo watch --clear --exec '{{ args }}'
