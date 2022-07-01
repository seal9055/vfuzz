build:
	@echo "Compiling stage2 and flattening (Rust portion of bootloader)"
	-@ cd bootloader; cargo build --release
	-@ cp ./bootloader/target/bootloader_config/release/bootloader ./stage2.bin
	-@ cargo run --release
	@echo "Assembling stage1"
	-@ nasm -f bin -o stage1.bin bootloader/src/stage1.asm
	@echo "Assembling stage0"
	-@ nasm -f bin -o vfuzz.boot bootloader/src/stage0.asm
	@echo "Removing intermediate files"
	-@ rm stage1.bin
	-@ rm stage2.bin

clean:
	-@rm vfuzz.boot 2>/dev/null || true
	-@rm -r bootloader/target 2>/dev/null || true
	-@rm stage1.bin 2>/dev/null || true
	-@rm stage2.bin 2>/dev/null || true
	-@rm flattened_stage2.bin 2>/dev/null || true
	-@rm -r target
	@echo "Clean Successful"
