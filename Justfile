doc:
    cargo +nightly rustdoc --all-features --open -- --cfg docsrs

test:
    cargo +nightly test --all-features

emit-llvm example:
    rm -rf ./out
    cargo +nightly rustc --example {{example}} --release --all-features --target-dir ./out -- --emit=llvm-ir

emit-asm example:
    rm -rf ./out
    cargo +nightly rustc --example {{example}} --release --all-features --target-dir ./out -- --emit=asm
