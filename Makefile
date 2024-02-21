CLANG = clang
TARGET = mos-atari8-dos
TARGET_DIR = target/$(TARGET)/release
DEPS_DIR = $(TARGET_DIR)/deps/

all: ferris.xex

ferris.xex: $(TARGET_DIR)/ferris rmt/music.obx ferris.elf
	cat rmt/music.obx $(TARGET_DIR)/ferris > ferris.xex

rmt/music.obx:
	make -C rmt

$(TARGET_DIR)/ferris: src/*.rs Cargo.toml
	cargo +mos build --release

ferris.elf: $(TARGET_DIR)/ferris
	cp $(DEPS_DIR)$$(ls -t $(DEPS_DIR) | grep 'ferris-.*elf' | head -n 1) ferris.elf

clean:
	make -C rmt clean
	cargo +mos clean
	rm -f ferris.xex ferris.elf
