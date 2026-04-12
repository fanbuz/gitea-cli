use std::fs;

#[test]
fn release_workflow_builds_macos_amd64_target() {
    let workflow = fs::read_to_string(".github/workflows/release.yml").unwrap();

    assert!(
        workflow.contains("target: x86_64-apple-darwin"),
        "release workflow should include a macOS amd64 target entry"
    );
    assert!(
        workflow.contains("asset_name: gitea-cli-macos-amd64.tar.gz"),
        "release workflow should package a dedicated macOS amd64 archive"
    );
}

#[test]
fn release_workflow_builds_from_matrix_target_directory() {
    let workflow = fs::read_to_string(".github/workflows/release.yml").unwrap();

    assert!(
        workflow.contains("cargo build --locked --release --target ${{ matrix.target }}"),
        "release workflow should build the explicit matrix target"
    );
    assert!(
        workflow.contains("target/${{ matrix.target }}/release"),
        "release workflow should package binaries from the explicit matrix target directory"
    );
}

#[test]
fn build_workflow_builds_macos_amd64_target() {
    let workflow = fs::read_to_string(".github/workflows/build.yml").unwrap();

    assert!(
        workflow.contains("target: x86_64-apple-darwin"),
        "build workflow should include a macOS amd64 target entry"
    );
    assert!(
        workflow.contains("asset_name: gitea-cli-macos-amd64.tar.gz"),
        "build workflow should package a dedicated macOS amd64 archive"
    );
}
