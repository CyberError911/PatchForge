.PHONY: build test fmt clean

build:
	@cargo build --workspace

release:
	@cargo build --workspace --release

test:
	@cargo test --workspace

fmt:
	@cargo fmt --all

clean:
	@cargo clean
