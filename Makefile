.PHONY: build
build:
	cargo build --release
	mv target/release/arbitrum-claim ./

.PHONY: run
run: build
	./arbitrum-claim

.SILENT:
.DEFAULT_GOAL := run