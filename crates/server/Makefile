CARGO_RUN_WATCH = RUST_LOG=debug cargo watch -x 'run --'
CARGO_RUN = RUST_LOG=debug cargo run --

WATCH_EXISTS = $(shell which cargo-watch > /dev/null && echo 1 || echo 0)
INSTALL_WATCH = cargo install cargo-watch

DOCKER_COMPOSE_EXISTS = $(shell which docker-compose > /dev/null && echo 1 || echo 0)

RUN_LOCAL_DB = docker-compose up -d && docker exec quests_db bash -c "until pg_isready; do sleep 1; done" > /dev/null && sleep 5
LOCAL_DB = $(shell docker ps | grep quests_db > /dev/null && echo 1 || echo 0)

dev:
ifeq ($(WATCH_EXISTS), 1)
	@make rundb
	@$(CARGO_RUN_WATCH)
else
	@echo "cargo-watch not found. installing..."
	@$(INSTALL_WATCH)
	@make rundb
	@$(CARGO_RUN_WATCH)
endif

run: 
	@make rundb
	@$(CARGO_RUN)

destroydb:
	@docker stop quests_db
	@docker rm quests_db
	@docker volume rm quests_db_volume

query:
	@docker exec -it quests_db bash -c "psql -U postgres -d quests_db"
