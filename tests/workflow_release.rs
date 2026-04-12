use std::fs;

#[test]
fn release_publish_job_checks_out_repository() {
    let workflow = fs::read_to_string(".github/workflows/release.yml").unwrap();

    let publish_section = workflow
        .split("\n  publish:\n")
        .nth(1)
        .expect("release.yml should contain publish job");

    assert!(
        publish_section.contains("uses: actions/checkout@v4"),
        "publish job must checkout repository before gh release commands"
    );
}
