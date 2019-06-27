use insta::assert_display_snapshot_matches;

#[cfg(test)]
fn transpile_from_file(test_filename: &str) -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_case_path: std::path::PathBuf = [&manifest_dir, "tests", "assets", test_filename]
        .iter()
        .collect();
    let test_case_file = std::fs::File::open(test_case_path).unwrap();
    let test_case_buf = std::io::BufReader::new(test_case_file);
    let lines = indentexlib::preprocessing::read_and_trim_lines(test_case_buf).unwrap();

    let default_options = indentexlib::TranspileOptions {
        flatten_output: false,
        prepend_do_not_edit_notice: true,
    };

    indentexlib::transpile(lines, &default_options)
}

#[test]
fn transpile_corner_cases() {
    assert_display_snapshot_matches!(
        "corner_cases",
        transpile_from_file("corner_cases.inden.tex")
    );
}

#[test]
fn transpile_envs() {
    assert_display_snapshot_matches!("envs", transpile_from_file("envs.inden.tex"));
}

#[test]
fn transpile_large_corpus() {
    assert_display_snapshot_matches!(
        "large_corpus",
        transpile_from_file("large_corpus.inden.tex")
    );
}

#[test]
fn transpile_list_like() {
    assert_display_snapshot_matches!("list_like", transpile_from_file("list_like.inden.tex"));
}

#[test]
fn transpile_mixed_tabs() {
    assert_display_snapshot_matches!("mixed_tabs", transpile_from_file("mixed_tabs.inden.tex"));
}

#[test]
fn transpile_single_line_cmds() {
    assert_display_snapshot_matches!(
        "single_line_cmds",
        transpile_from_file("single_line_cmds.inden.tex")
    );
}
