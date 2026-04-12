CARGO ?= cargo
CARGO_HOME ?= $(CURDIR)/.cargo-home
INSTALL_BIN ?= $(HOME)/.local/bin
BINARY := target/release/gitea-cli

.PHONY: test fmt install-local

test:
	CARGO_HOME="$(CARGO_HOME)" $(CARGO) test

fmt:
	CARGO_HOME="$(CARGO_HOME)" $(CARGO) fmt

install-local:
	CARGO_HOME="$(CARGO_HOME)" $(CARGO) build --release
	mkdir -p "$(INSTALL_BIN)"
	cp "$(BINARY)" "$(INSTALL_BIN)/gitea-cli"
