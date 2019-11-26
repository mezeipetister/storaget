 
.PHONY: release, test

release:
	cargo build --release
	# strip target/release/website

build:
	cargo build

test:
	cargo test