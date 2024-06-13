pub fn ensure_bindings_exists_in_tmpdir(next_bindings_dir: std::path::PathBuf) {
    let tmp_bindings_dir = std::env::temp_dir().join("bindings");

    if tmp_bindings_dir.exists() {
        std::fs::remove_dir_all(&tmp_bindings_dir)
            .expect("byth was unable to remove tmp_bindings_dir");
    }

    std::fs::create_dir_all(&tmp_bindings_dir)
        .expect("Unable to create tmp_bindings_dir.");


    fs_extra::dir::copy(&next_bindings_dir, &tmp_bindings_dir, &fs_extra::dir::CopyOptions::new().content_only(true))
        .expect("byth was unable to copy next_bindings_dir");
}