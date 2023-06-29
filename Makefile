build:
	cd md2html && cargo build
	./md2html/target/debug/md2html data/
lint:
	cd md2html && cargo fmt --check
