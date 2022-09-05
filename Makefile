PROFILE ?= dev

.PHONY: build
build:
	cargo build --profile $(PROFILE)

.PHONY: check
check:
	cargo test      \
		--workspace \
		--bins      \
		--lib

.PHONY: lint
lint:
	cargo clippy --no-deps -- -D warnings

.PHONY: format
format:
	cargo fmt -- --check --config format_code_in_doc_comments=true

.PHONY: ci
ci: build check lint format

.PHONY: clean
clean:
	cargo clean
