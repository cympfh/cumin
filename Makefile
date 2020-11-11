all:
	cargo test
	make install

install:
	cargo install --path . --force
