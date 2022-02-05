all: test
	make install

test:
	cargo test

install:
	cargo install --path . --force
	@echo "Enjoy!"
	@echo "=> $(HOME)/.cargo/bin/cuminc"
