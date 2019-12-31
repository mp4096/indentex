use insta::assert_display_snapshot;

#[cfg(test)]
fn read_from_file(test_filename: &str) -> Vec<String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_case_path: std::path::PathBuf = [&manifest_dir, "tests", "assets", test_filename]
        .iter()
        .collect();
    let test_case_file = std::fs::File::open(test_case_path).unwrap();
    let test_case_buf = std::io::BufReader::new(test_case_file);
    let lines = indentexlib::preprocessing::read_and_trim_lines(test_case_buf);
    assert!(lines.is_ok());
    lines.unwrap()
}

#[cfg(test)]
fn transpile_from_file(test_filename: &str) -> String {
    let lines = read_from_file(test_filename);

    let default_options = indentexlib::TranspileOptions {
        prepend_do_not_edit_notice: true,
    };

    indentexlib::transpile(lines, &default_options)
}

#[test]
fn vanilla_latex_is_not_modified() {
    use indentexlib::preprocessing::read_and_trim_lines;
    use std::io::BufReader;

    let lines = read_from_file("large_latex_corpus.inden.tex");
    let expected_lines = lines.clone();
    let no_prepend = indentexlib::TranspileOptions {
        prepend_do_not_edit_notice: false,
    };
    let transpiled = indentexlib::transpile(lines, &no_prepend);
    let actual_lines = read_and_trim_lines(BufReader::new(transpiled.as_bytes())).unwrap();
    assert_eq!(actual_lines, expected_lines);
}

#[test]
fn transpile_corner_cases() {
    assert_display_snapshot!(
        "corner_cases",
        transpile_from_file("corner_cases.inden.tex")
    );
}

#[test]
fn transpile_envs() {
    assert_display_snapshot!("envs", transpile_from_file("envs.inden.tex"));
}

#[test]
fn transpile_large_corpus() {
    assert_display_snapshot!(
        "large_corpus",
        transpile_from_file("large_corpus.inden.tex")
    );
}

#[test]
fn transpile_list_like() {
    assert_display_snapshot!("list_like", transpile_from_file("list_like.inden.tex"));
}

#[test]
fn transpile_mixed_tabs() {
    assert_display_snapshot!("mixed_tabs", transpile_from_file("mixed_tabs.inden.tex"));
}

#[test]
fn transpile_single_line_cmds() {
    assert_display_snapshot!(
        "single_line_cmds",
        transpile_from_file("single_line_cmds.inden.tex")
    );
}
