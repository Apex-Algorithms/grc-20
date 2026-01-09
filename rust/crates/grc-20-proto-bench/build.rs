fn main() {
    prost_build::compile_protos(&["src/grc20.proto"], &["src/"])
        .expect("Failed to compile protos");
}
