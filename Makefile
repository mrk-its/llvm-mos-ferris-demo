CLANG = clang
TARGET = mos-atari8-none
TARGET_DIR = target/$(TARGET)/release

ferris.xex: $(TARGET_DIR)/ferris rmt/music.obx
	cat rmt/music.obx $(TARGET_DIR)/ferris > ferris.xex

rmt/music.obx:
	make -C rmt

$(TARGET_DIR)/ferris: src/*.rs Cargo.toml
	cargo +mos build --release -vv

clean:
	make -C rmt clean
	cargo +mos clean
