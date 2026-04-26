fn main() {
    println!("cargo::rerun-if-changed=fonts/icons.toml");
    iced_lucide::build("fonts/icons.toml").expect("Build icon module");

    #[cfg(windows)]
    {
        embed_resource::compile("assets/windows/mannager.rc", embed_resource::NONE);
        windows_exe_info::versioninfo::link_cargo_env();
    }
}
