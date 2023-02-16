DOCKER_COMPOSE_EXISTS = $(shell which docker-compose > /dev/null && echo 1 || echo 0)

RUN_LOCAL_DB = docker-compose -f docker-compose.local.yml up -d && docker exec quests_db bash -c "until pg_isready; do sleep 1; done" > /dev/null && sleep 5
LOCAL_DB = $(shell docker ps | grep quests_db > /dev/null && echo 1 || echo 0)

CARGO_RUN_SERVER_WATCH = RUST_LOG=debug cargo watch -x 'run --bin quests_server --'
CARGO_RUN_SERVER = RUST_LOG=debug cargo run --bin quests_server --

WATCH_EXISTS = $(shell which cargo-watch > /dev/null && echo 1 || echo 0)
INSTALL_WATCH = cargo install cargo-watch

export DATABASE_URL=postgres://postgres:postgres@localhost:5432/quests_db # due to docker-compose.local.yml

runservices:
ifeq ($(DOCKER_COMPOSE_EXISTS), 1)
	@$(RUN_LOCAL_DB)
else
	@$(ERROR) "Install Docker in order to run the local DB"
	@exit 1;
endif

destroyservices:
	-@docker stop quests_db
	-@docker stop quests_redis
	-@docker rm quests_db
	-@docker rm quests_redis 
	-@docker volume rm quests_quests_db_volume
# run tests locally
test-db:
ifeq ($(LOCAL_DB), 1)
	@make destroyservices
	@make runservices
	-@cargo test --package quests_db
	@docker stop quests_db
	@make destroyservices
else
	@make runservices
	-@cargo test --package quests_db
	@make destroyservices
endif

# run tests locally
test-message-broker:
ifeq ($(LOCAL_DB), 1)
	@make destroyservices
	@make runservices
	-@cargo test --package quests_message_broker
	@docker stop quests_db
	@make destroyservices
else
	@make runservices
	-@cargo test --package quests_message_broker
	@make destroyservices
endif


# run tests locally
test-server:
ifeq ($(LOCAL_DB), 1)
	@make destroyservices
	@make runservices 
	-@cargo test --package quests_server
	@docker stop quests_db
	@make destroyservices
else
	@make runservices
	-@cargo test --package quests_server
	@make destroyservices
endif

# run tests locally
test-definitions:
	-@cargo test --package quests_definitions

# run tests locally
tests: test-db test-server test-message-broker test-definitions # TODO: change to setup services only once for all packages and run cargo test for the whole project

run-devserver:
ifeq ($(WATCH_EXISTS), 1)
	@make runservices
	@$(CARGO_RUN_SERVER_WATCH)
else
	@echo "cargo-watch not found. installing..."
	@$(INSTALL_WATCH)
	@make runservices
	@$(CARGO_RUN_SERVER_WATCH)
endif

run-server:
	@make runservices
	@$(CARGO_RUN)
