fn main() {
    #[cfg(windows)]
    {
        embed_resource::compile("assets/windows/mannager.rc", embed_resource::NONE);
        windows_exe_info::versioninfo::link_cargo_env();
    }
}
