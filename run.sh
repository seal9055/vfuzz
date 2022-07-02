echo "Launching VFUZZ"

qemu-system-x86_64 \
    -drive format=raw,file=vfuzz.boot \
    -serial stdio \
     # -s -S
