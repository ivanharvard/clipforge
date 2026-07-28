fn main() {
    slint_build::compile("ui/app.slint").expect("failed to compile Slint UI");

    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("../../packaging/windows/icon.ico")
            .compile()
            .expect("failed to embed Windows exe icon");
    }
}
