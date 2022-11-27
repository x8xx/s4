all: parser/parser.wasm pipeline/pipeline.wasm init_table_entry.s4bin pktgen/target/release/libpktgen.so

parser/parser.wasm: parser/parser.rs
	./parser/build.sh

pipeline/pipeline.wasm: pipeline/pipeline.rs
	./pipeline/build.sh

init_table_entry.s4bin: gen_table_entry.py
	python3 gen_table_entry.py

pktgen/target/release/libpktgen.so: pktgen/src/lib.rs
	cd ./pktgen && cargo build --release

clean:
	rm -f *.s4bin ./parser/parser.wasm ./pipeline/pipeline.wasm

.PHONY: clean
