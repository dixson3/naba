# naba build (plan-004): the shipped `naba` is the RUST binary. The Go source is kept
# ONLY as the CI parity baseline (proves the goldens are still valid) and is built via the
# `*-go` targets; a follow-up bead retires it. See CI (.github/workflows/ci.yml).

# --- Rust (product / shipped binary) ------------------------------------------------
.PHONY: build test lint fmt clean install parity traceability \
        build-go test-go lint-go parity-go

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
	rm -f naba naba-go

# --- Go (CI parity baseline ONLY — not shipped) -------------------------------------
GO_VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo dev)
GO_COMMIT  ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo none)
GO_DATE    ?= $(shell date -u +%Y-%m-%dT%H:%M:%SZ)
GO_LDFLAGS := -s -w \
	-X github.com/dixson3/naba/internal/cli.Version=$(GO_VERSION) \
	-X github.com/dixson3/naba/internal/cli.Commit=$(GO_COMMIT) \
	-X github.com/dixson3/naba/internal/cli.Date=$(GO_DATE)

# Build the Go baseline binary at ./naba-go (never shipped).
build-go:
	go build -trimpath -ldflags "$(GO_LDFLAGS)" -o naba-go ./cmd/naba

test-go:
	go test ./... -count=1

lint-go:
	golangci-lint run ./...

# Parity suite against the Go baseline (must stay green: goldens are Go-captured).
parity-go: build-go
	NABA_BIN="$(CURDIR)/naba-go" uv run --project tests/parity pytest tests/parity
