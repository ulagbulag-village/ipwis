#[link(wasm_import_module = "ipwis_modules_stream")]
extern "C" {
    pub fn reader_next(error: u32, id: u64, buf: u32) -> u32;
    pub fn writer_next(error: u32, id: u64, buf: u32) -> u32;
    pub fn writer_flush(error: u32, id: u64);
    pub fn writer_shutdown(error: u32, id: u64);
}
