LLVM_BIN_DIR="../llvm-build/bin/"

./target/debug/ironcc "$1"
${LLVM_BIN_DIR}clang -S -emit-llvm a.bc
${LLVM_BIN_DIR}clang a.bc
