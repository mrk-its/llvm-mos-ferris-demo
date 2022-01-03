CLANG = clang

ferris.xex: target/mos-unknown-none/debug/ferris rmt/music.obx
	cat rmt/music.obx target/mos-unknown-none/debug/ferris > ferris.xex

rmt/music.obx:
	make -C rmt

create_ferris: tools/create_ferris.c
	${CLANG} tools/create_ferris.c -o create_ferris

src/ferris.dat: create_ferris
	./create_ferris > src/ferris.dat

target/mos-unknown-none/debug/ferris: src/ferris.dat src/*.rs Cargo.toml
	cargo +mos build

run: target/mos-unknown-none/debug/ferris
	atari800 -run target/mos-unknown-none/debug/ferris

clean:
	make -C rmt clean
	cargo +mos clean
	rm -f create_ferris src/ferris.dat
