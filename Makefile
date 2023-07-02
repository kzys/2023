build:
	cd md2html && cargo build
	./md2html/target/debug/md2html --site-url https://2023.8-p.info data/
lint:
	cd md2html && cargo fmt --check
