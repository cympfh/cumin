all:
	cargo test
	make install

install:
	cargo install --path . --force
	@echo "Enjoy!"
	@echo "=> $(HOME)/.cargo/bin/cuminc"
