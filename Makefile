DOCKER_COMPOSE_EXISTS = $(shell which docker-compose > /dev/null && echo 1 || echo 0)

RUN_LOCAL_DB = docker-compose -f docker-compose.local.yml up -d && docker exec quests_db bash -c "until pg_isready; do sleep 1; done" > /dev/null && sleep 5
LOCAL_DB = $(shell docker ps | grep quests_db > /dev/null && echo 1 || echo 0)

CARGO_RUN_SERVER_WATCH = RUST_LOG=debug cargo watch -x 'run --bin quests_server --'
CARGO_RUN_SERVER = RUST_LOG=debug cargo run --bin quests_server --

WATCH_EXISTS = $(shell which cargo-watch > /dev/null && echo 1 || echo 0)
INSTALL_WATCH = cargo install cargo-watch

export DATABASE_URL=postgres://postgres:postgres@localhost:3500/quests_db # due to docker-compose.local.yml

rundb:
ifeq ($(DOCKER_COMPOSE_EXISTS), 1)
	@$(RUN_LOCAL_DB)
else
	@$(ERROR) "Install Docker in order to run the local DB"
	@exit 1;
endif

destroydb:
	-@docker stop quests_db
	-@docker rm quests_db
	-@docker volume rm quests_quests_db_volume
# run tests locally
test-db:
ifeq ($(LOCAL_DB), 1)
	@make destroydb
	@make rundb
	-@cargo test --package quests_db
	@docker stop quests_db
	@make destroydb
else
	@make rundb
	-@cargo test --package quests_db
	@make destroydb
endif

# run tests locally
test-server:
ifeq ($(LOCAL_DB), 1)
	@make destroydb
	@make rundb
	-@cargo test --package quests_server
	@docker stop quests_db
	@make destroydb
else
	@make rundb
	-@cargo test --package quests_server
	@make destroydb
endif

# run tests locally
test-definitions:
	-@cargo test --package quests_definitions

# run tests locally
tests: test-db test-server test-definitions

run-devserver:
ifeq ($(WATCH_EXISTS), 1)
	@make rundb
	@$(CARGO_RUN_SERVER_WATCH)
else
	@echo "cargo-watch not found. installing..."
	@$(INSTALL_WATCH)
	@make rundb
	@$(CARGO_RUN_SERVER_WATCH)
endif

run-server:
	@make rundb
	@$(CARGO_RUN)