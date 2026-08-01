fn main() {
    // The `qt-style` feature (used by the separate `clipforge-kde` package)
    // compiles std-widgets against Slint's `qt` style, so they render via
    // the user's actual KDE QStyle instead of Slint's default look. This is
    // baked in at compile time and hard-links Qt6 — there's no runtime
    // toggle between the two, see DESIGN docs / plan for why.
    if std::env::var_os("CARGO_FEATURE_QT_STYLE").is_some() {
        let config = slint_build::CompilerConfiguration::new().with_style("qt".into());
        slint_build::compile_with_config("ui/app.slint", config)
            .expect("failed to compile Slint UI with qt style");
    } else {
        slint_build::compile("ui/app.slint").expect("failed to compile Slint UI");
    }

    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("../../packaging/windows/icon.ico")
            .compile()
            .expect("failed to embed Windows exe icon");
    }
}
