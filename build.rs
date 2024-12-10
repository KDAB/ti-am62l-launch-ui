#[cfg(not(target_arch = "aarch64"))]
fn main() {
    slint_build::compile("ui/appwindow.slint").expect("Slint build failed");
}

#[cfg(target_arch = "aarch64")]
fn main() {
    slint_build::compile_with_config(
        "ui/appwindow.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles),
            // .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer),
    )
    .expect("Slint build failed");
}
