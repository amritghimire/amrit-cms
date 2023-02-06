.PHONY: check
check:
	cargo watch -x check

.PHONY: run
run:
	cargo watch -x 'run -p api_server'

.PHONY: watch
watch:
	cargo watch -x check -x test -x 'run -p api_server'


