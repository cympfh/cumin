test:
	cargo test
	bash ./examples/test.sh

install:
	cargo install --path . --force
	@echo "Enjoy!"
	@echo "=> $(HOME)/.cargo/bin/cuminc"

wasm:
	wasm-pack build --target no-modules
