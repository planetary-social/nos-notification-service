.PHONY: ci
ci: test clippy build fmt check_repository_unchanged

.PHONY: check_repository_unchanged
check_repository_unchanged: 
	_tools/check_repository_unchanged.sh

.PHONY: build
build:
	cargo build

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: test
test:
	cargo test

.PHONY: clippy
clippy:
	cargo clippy -- -D warnings

.PHONY: run
run:
	cargo run
