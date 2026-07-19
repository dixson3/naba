# naba build: the shipped `naba` is a single Rust binary. (The legacy Go source and its
# `*-go` parity-baseline targets were retired once Rust parity was trusted — see CI.)

# --- Rust (product / shipped binary) ------------------------------------------------
.PHONY: build test lint fmt clean install parity traceability

# Default: build the shipped Rust binary at ./naba (copied from target/release/naba).
build:
	cargo build --release
	cp -f target/release/naba naba

test:
	cargo test

lint:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --check

install:
	cargo install --path .

# Parity suite against the shipped (Rust) binary.
parity: build
	NABA_BIN="$(CURDIR)/target/release/naba" uv run --project tests/parity pytest tests/parity

# SPEC<->test traceability check (Issue 5.3).
traceability:
	uv run tests/parity/check_traceability.py

clean:
	cargo clean
	rm -f naba
