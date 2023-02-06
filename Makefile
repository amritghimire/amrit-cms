.PHONY: check
check:
	cargo check

.PHONY: run
run:
	cargo watch -x 'run -p api_server'

.PHONY: watch
watch:
	cargo watch -x check -x test -x 'run -p api_server'

.PHONY: coverage
coverage:
	cargo tarpaulin --ignore-tests

.PHONY: clippy
clippy:
	cargo clippy -- -D warnings

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: check_fmt
check_fmt:
	cargo fmt -- --check

.PHONY: audit
audit:
	cargo audit

.PHONY: check_all
check_all: check clippy check_fmt audit
