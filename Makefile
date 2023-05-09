DOCKER_COMPOSE_EXISTS = $(shell which docker-compose > /dev/null && echo 1 || echo 0)

RUN_SERVICES = docker-compose -f docker-compose.local.yml up -d && docker exec quests_db bash -c "until pg_isready; do sleep 1; done" > /dev/null && sleep 5
LOCAL_DB = $(shell docker ps | grep quests_db > /dev/null && echo 1 || echo 0)

CARGO_RUN_SERVER_WATCH = RUST_LOG=debug cargo watch -x 'run --bin quests_server --'
CARGO_RUN_SERVER = RUST_LOG=debug cargo run --bin quests_server --

WATCH_EXISTS = $(shell which cargo-watch > /dev/null && echo 1 || echo 0)
INSTALL_WATCH = cargo install cargo-watch

export DATABASE_URL=postgres://postgres:postgres@localhost:5432/quests_db # due to docker-compose.local.yml

runservices:
ifeq ($(DOCKER_COMPOSE_EXISTS), 1)
	@$(RUN_SERVICES)
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

test-db: TEST_PROJECT=-p quests_db
test-db: tests 

test-message-broker: TEST_PROJECT=-p quests_message_broker
test-message-broker: tests

test-server: TEST_PROJECT=-p quests_server
test-server: tests

test-system: TEST_PROJECT=-p quests_system
test-system: tests

test-protocol:
	-@cargo test --package quests_protocol

# run tests locally
tests: 
ifeq ($(LOCAL_DB), 1)
	@make destroyservices
	@make runservices 
	-@cargo test $(TEST_PROJECT)
	@docker stop quests_db
	@make destroyservices
else
	@make runservices
	-@cargo test $(TEST_PROJECT)
	@make destroyservices
endif


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
	@$(CARGO_RUN_SERVER)
