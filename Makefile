test:
	cargo test

install:
	cargo install --path . --force
	@echo "Enjoy!"
	@echo "=> $(HOME)/.cargo/bin/cuminc"

wasm:
	wasm-pack build --target no-modules
