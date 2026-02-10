VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo dev)
COMMIT  ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo none)
DATE    ?= $(shell date -u +%Y-%m-%dT%H:%M:%SZ)
LDFLAGS := -s -w \
	-X github.com/dixson3/naba/internal/cli.Version=$(VERSION) \
	-X github.com/dixson3/naba/internal/cli.Commit=$(COMMIT) \
	-X github.com/dixson3/naba/internal/cli.Date=$(DATE)

.PHONY: build test lint clean

build:
	go build -trimpath -ldflags "$(LDFLAGS)" -o naba ./cmd/naba

test:
	go test ./... -count=1

lint:
	golangci-lint run ./...

clean:
	rm -f naba
