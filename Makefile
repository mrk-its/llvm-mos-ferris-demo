CLANG = ../llvm-mos/build/bin/clang

all: target/mos-unknown-none/debug/ferris

create_ferris: tools/create_ferris.c
	${CLANG} tools/create_ferris.c -o create_ferris

src/ferris.dat: create_ferris
	./create_ferris > src/ferris.dat

target/mos-unknown-none/debug/ferris: src/ferris.dat
	cargo +mos build -vv

clean:
	cargo +mos clean
	rm -f create_ferris src/ferris.dat
