use std::fs;

#[test]
fn release_publish_job_checks_out_repository() {
    let workflow = fs::read_to_string(".github/workflows/release.yml").unwrap();
    let publish_section = extract_publish_section(&workflow);

    assert!(
        publish_section.contains("uses: actions/checkout@v4"),
        "publish job must checkout repository before gh release commands"
    );
}

#[test]
fn release_publish_job_detection_supports_crlf() {
    let workflow = "\
name: Release Binaries\r\n\
jobs:\r\n\
  build:\r\n\
    runs-on: ubuntu-latest\r\n\
  publish:\r\n\
    steps:\r\n\
      - name: Checkout release ref\r\n\
        uses: actions/checkout@v4\r\n";

    let publish_section = extract_publish_section(workflow);

    assert!(
        publish_section.contains("uses: actions/checkout@v4"),
        "publish job parsing should work with CRLF newlines"
    );
}

fn extract_publish_section(workflow: &str) -> String {
    let normalized = workflow.replace("\r\n", "\n");
    let lines = normalized.lines().collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| line.trim() == "publish:")
        .expect("release.yml should contain publish job")
        + 1;

    lines[start..].join("\n")
}
