fn main() {
    prost_build::compile_protos(&["proto/snakes.proto"], &["proto"]).unwrap_or_else(|err| {
        panic!("error compiling .proto: {err}\nMaybe protobuf compiler is not installed?");
    });
}
