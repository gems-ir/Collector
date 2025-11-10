#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("app.manifest");
    res.compile().unwrap();
}

#[!cfg(windows)]
fn main() {}