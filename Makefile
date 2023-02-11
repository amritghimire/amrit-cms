export DB_USER=${POSTGRES_USER:=postgres}
export DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
export DB_NAME="${POSTGRES_DB:=amrit_cms}"
export DB_PORT="${POSTGRES_PORT:=5432}"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}

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

.PHONY: test
test:
	cargo test

.PHONY: check_all
check_all: check clippy check_fmt audit test

.PHONY: init_db
init_db:
	./scripts/init_db.sh

.PHONY: sqlx
sqlx:
	sqlx $(filter-out $@,$(MAKECMDGOALS))

%:
	@:
