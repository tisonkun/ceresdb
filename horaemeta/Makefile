GO_TOOLS_BIN_PATH := $(shell pwd)/.tools/bin
PATH := $(GO_TOOLS_BIN_PATH):$(PATH)
SHELL := env PATH='$(PATH)' GOBIN='$(GO_TOOLS_BIN_PATH)' $(shell which bash)
ROOT = $(shell pwd)

COMMIT_ID := $(shell git rev-parse HEAD)
BRANCH_NAME := $(shell git rev-parse --abbrev-ref HEAD)
BUILD_DATE := $(shell date +'%Y/%m/%dT%H:%M:%S')

default: build

install-tools:
	@mkdir -p $(GO_TOOLS_BIN_PATH)
	@(which golangci-lint && golangci-lint version | grep '1.54') >/dev/null 2>&1 || curl -sSfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh | sh -s -- -b $(GO_TOOLS_BIN_PATH) v1.54.2

META_PKG := github.com/apache/incubator-horaedb-meta
PACKAGES := $(shell go list ./... | tail -n +2)
PACKAGE_DIRECTORIES := $(subst $(META_PKG)/,,$(PACKAGES))

check:
	@ echo "gofmt ..."
	@ gofmt -s -l -d $(PACKAGE_DIRECTORIES) 2>&1 | awk '{ print } END { if (NR > 0) { exit 1 } }'
	@ echo "golangci-lint ..."
	@ golangci-lint run $(PACKAGE_DIRECTORIES) --config .golangci.yml

test:
	@ echo "go test ..."
	@ go test -timeout 5m -coverprofile=coverage.txt -covermode=atomic $(PACKAGES)

build:
	@ go build -ldflags="-X main.commitID=$(COMMIT_ID) -X main.branchName=$(BRANCH_NAME) -X main.buildDate=$(BUILD_DATE)" -o bin/horaemeta-server ./cmd/horaemeta-server
