fn main() -> ::ipwis_kernel_builder::BuildResult {
    let src = "../../target/wasm32-wasi/release/ipwis_kernel_api.wasm";
    let dst = "output.wasm";

    ::ipwis_kernel_builder::try_build_wasi(src, dst)
}
