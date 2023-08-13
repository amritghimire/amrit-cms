DB_USER ?= postgres
DB_PASSWORD ?= password
DB_NAME ?=amrit_cms
DB_PORT ?= 5432
DB_HOST ?= localhost

DATABASE_URL:=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}

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
	DATABASE_URL="$(DATABASE_URL)" cargo tarpaulin --ignore-tests

.PHONY: clippy
clippy:
	DATABASE_URL="$(DATABASE_URL)" cargo clippy -- -D warnings

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
	DATABASE_URL="$(DATABASE_URL)" cargo test

.PHONY: check_all
check_all: check clippy check_fmt check_sqlx test

.PHONY: init_db
init_db:
	./scripts/init_db.sh

.PHONY: migrate
migrate:
	SKIP_DOCKER=true ./scripts/init_db.sh

.PHONY: sqlx
sqlx:
	sqlx $(filter-out $@,$(MAKECMDGOALS))

.PHONY: prepare_sqlx
prepare_sqlx:
	cargo sqlx prepare --merged -- --lib

.PHONY: check_sqlx
check_sqlx:
	cargo sqlx prepare --check --merged -- --lib

.PHONY: docker
docker:
	docker build --tag amrit_cms --file Dockerfile .
	docker run --env DATABASE_URL="$(DATABASE_URL)" -p 8080:8080  amrit_cms

%:
	@:
