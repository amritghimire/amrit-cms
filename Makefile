DB_USER ?= postgres
DB_PASSWORD ?= password
DB_NAME ?=amrit_cms
DB_PORT ?= 5432
DB_HOST ?= localhost

DATABASE_URL:=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}

.PHONY: check
check:
	cargo check

.PHONY: dev
dev:
	cargo watch -x 'run -p api_server --no-default-features --features local'

.PHONY: run
run: build_frontend
	cd frontend/client && trunk build
	cargo run -p api_server --no-default-features --features local

.PHONY: run_api
run_api:
	cargo run -p api_server --no-default-features --features local


.PHONY: release
release: build_frontend_release
	cargo build --release  --no-default-features --features local

.PHONY: watch
watch:
	cargo watch -x check -x test -x 'run -p api_server'

.PHONY: watch_frontend
watch_frontend:
	cd frontend/client && trunk watch

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

.PHONY: migrate
migrate:
	SQLX_OFFLINE=true DATABASE_URL="$(DATABASE_URL)" cargo run -p api_server -- migrate

.PHONY: check_all
check_all: check clippy check_fmt check_sqlx test

.PHONY: init_db
init_db:
	./scripts/init_db.sh

.PHONY: sqlx
sqlx:
	sqlx $(filter-out $@,$(MAKECMDGOALS))

.PHONY: prepare_sqlx
prepare_sqlx:
	cargo sqlx prepare --workspace -- --lib --tests

.PHONY: check_sqlx
check_sqlx:
	cargo sqlx prepare --check --workspace -- --lib --tests

.PHONY: docker
docker:
	docker build --tag amrit_cms --file Dockerfile .
	docker run --env DATABASE_URL="$(DATABASE_URL)" -p 8080:8080  amrit_cms

.PHONY: build_frontend
build_frontend: build_tailwind
	cd frontend/client && trunk build

.PHONY: build_frontend_release
build_frontend_release: build_tailwind
	cd frontend/client && trunk build --release


.PHONY: build_tailwind
build_tailwind:
	echo 1
%:
	@:
