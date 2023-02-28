echo "Launching VFUZZ"

qemu-system-x86_64 \
    -drive format=raw,file=vfuzz.boot \
    -serial stdio \
    -smp cores=4,threads=1,sockets=1
     # -s -S
