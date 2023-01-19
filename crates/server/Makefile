CARGO_RUN_WATCH = RUST_LOG=debug cargo watch -x 'run --'
CARGO_RUN = RUST_LOG=debug cargo run --

WATCH_EXISTS = $(shell which cargo-watch > /dev/null && echo 1 || echo 0)
INSTALL_WATCH = cargo install cargo-watch

dev:
ifeq ($(WATCH_EXISTS), 1)
	@$(CARGO_RUN_WATCH)
else
	@echo "cargo-watch not found. installing..."
	@$(INSTALL_WATCH)
	@$(CARGO_RUN_WATCH)
endif

run: 
	@$(CARGO_RUN)