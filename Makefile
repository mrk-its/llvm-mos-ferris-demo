CLANG = clang
TARGET = mos-a800xl-none
TARGET_DIR = target/$(TARGET)/release

ferris.xex: $(TARGET_DIR)/ferris rmt/music.obx
	cat rmt/music.obx $(TARGET_DIR)/ferris > ferris.xex

rmt/music.obx:
	make -C rmt

create_ferris: tools/create_ferris.c
	${CLANG} tools/create_ferris.c -o create_ferris

src/ferris.dat: create_ferris
	./create_ferris > src/ferris.dat

$(TARGET_DIR)/ferris: src/ferris.dat src/*.rs Cargo.toml
	cargo +mos build --release -vv

run: $(TARGET_DIR)/ferris
	atari800 -run $(TARGET_DIR)/ferris

clean:
	make -C rmt clean
	cargo +mos clean
	rm -f create_ferris src/ferris.dat
