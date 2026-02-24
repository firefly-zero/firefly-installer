#[link(wasm_import_module = "wifi")]
unsafe extern "C" {
    pub(crate) fn scan(ptr: u32, len: u32) -> u32;
    pub(crate) fn connect(ssid_ptr: u32, ssid_len: u32, pass_ptr: u32, pass_len: u32);
    pub(crate) fn disconnect();
}
