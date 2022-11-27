all: parser/parser.wasm pipeline/pipeline.wasm init_table_entry.s4bin

parser/parser.wasm:
	./parser/build.sh

pipeline/pipeline.wasm:
	./pipeline/build.sh

init_table_entry.s4bin:
	python3 gen_table_entry.py

clean:
	rm -f *.s4bin ./parser/parser.wasm ./pipeline/pipeline.wasm

.PHONY: clean
