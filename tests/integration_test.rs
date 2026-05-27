// Copyright (C) 2026 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use std::io::Write;
use std::process::{Command, Output};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Image build helpers
// ---------------------------------------------------------------------------

fn fedora_config_file() -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        f,
        "[openshell_image_builder.base_image]\nimage = \"fedora\"\ntag = \"latest\""
    )
    .unwrap();
    f
}

fn build_image(tag: &str, extra_args: &[&str]) -> String {
    let binary = env!("CARGO_BIN_EXE_openshell-image-builder");
    let status = Command::new(binary)
        .args(extra_args)
        .arg(tag)
        .status()
        .expect("binary should run");
    assert!(status.success(), "image build failed for tag {tag}");
    tag.to_string()
}

fn run_in_image(image: &str, cmd: &str) -> Output {
    Command::new("podman")
        .args(["run", "--rm", image, "-c", cmd])
        .output()
        .expect("podman run should execute")
}

// ---------------------------------------------------------------------------
// One OnceLock per image variant — each image is built at most once
// ---------------------------------------------------------------------------

static UBUNTU_IMAGE: OnceLock<String> = OnceLock::new();
static UBUNTU_CLAUDE_IMAGE: OnceLock<String> = OnceLock::new();
static FEDORA_IMAGE: OnceLock<String> = OnceLock::new();
static FEDORA_CLAUDE_IMAGE: OnceLock<String> = OnceLock::new();

fn ubuntu_image() -> &'static str {
    UBUNTU_IMAGE.get_or_init(|| build_image("openshell-test-ubuntu:integration", &[]))
}

fn ubuntu_claude_image() -> &'static str {
    UBUNTU_CLAUDE_IMAGE.get_or_init(|| {
        build_image(
            "openshell-test-ubuntu-claude:integration",
            &["--agent", "claude"],
        )
    })
}

fn fedora_image() -> &'static str {
    FEDORA_IMAGE.get_or_init(|| {
        let config = fedora_config_file();
        build_image(
            "openshell-test-fedora:integration",
            &["--config", config.path().to_str().unwrap()],
        )
    })
}

fn fedora_claude_image() -> &'static str {
    FEDORA_CLAUDE_IMAGE.get_or_init(|| {
        let config = fedora_config_file();
        build_image(
            "openshell-test-fedora-claude:integration",
            &[
                "--config",
                config.path().to_str().unwrap(),
                "--agent",
                "claude",
            ],
        )
    })
}

// ---------------------------------------------------------------------------
// Shared assertion helpers
// ---------------------------------------------------------------------------

fn check_users_and_groups(image: &str) {
    for user in ["sandbox", "supervisor"] {
        let out = run_in_image(image, &format!("id {user}"));
        assert!(out.status.success(), "{user} user not found in image");
    }

    for group in ["sandbox", "supervisor"] {
        let out = run_in_image(image, &format!("getent group {group}"));
        assert!(out.status.success(), "{group} group not found in image");
    }

    let out = run_in_image(image, "whoami");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "sandbox",
        "default image user is not sandbox"
    );

    let out = run_in_image(image, "echo $HOME");
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "/sandbox",
        "sandbox home directory is not /sandbox"
    );
}

fn check_packages(image: &str) {
    for pkg in ["curl", "ip", "ping", "traceroute"] {
        let out = run_in_image(image, &format!("which {pkg}"));
        assert!(out.status.success(), "{pkg} not found in image");
    }
}

fn check_bash_entrypoint(image: &str) {
    let out = Command::new("podman")
        .args(["inspect", "--format", "{{json .Config.Entrypoint}}", image])
        .output()
        .expect("podman inspect should execute");
    assert!(out.status.success(), "podman inspect failed");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("/bin/bash"),
        "expected /bin/bash entrypoint, got: {stdout}"
    );
}

fn check_claude_in_path(image: &str, expected: bool) {
    let out = run_in_image(image, "which claude");
    if expected {
        assert!(out.status.success(), "claude not found in PATH");
    } else {
        assert!(!out.status.success(), "claude should not be in PATH");
    }
}

// ---------------------------------------------------------------------------
// Matrix: base_image × agent — one test module per variant
// ---------------------------------------------------------------------------

macro_rules! image_tests {
    ($mod_name:ident, $image_fn:ident, has_claude: $has_claude:literal) => {
        mod $mod_name {
            use super::*;

            #[test]
            #[ignore]
            fn users_and_groups_exist() {
                check_users_and_groups($image_fn());
            }

            #[test]
            #[ignore]
            fn packages_installed() {
                check_packages($image_fn());
            }

            #[test]
            #[ignore]
            fn bash_entrypoint() {
                check_bash_entrypoint($image_fn());
            }

            #[test]
            #[ignore]
            fn claude_in_path() {
                check_claude_in_path($image_fn(), $has_claude);
            }
        }
    };
}

image_tests!(ubuntu,        ubuntu_image,        has_claude: false);
image_tests!(ubuntu_claude, ubuntu_claude_image, has_claude: true);
image_tests!(fedora,        fedora_image,        has_claude: false);
image_tests!(fedora_claude, fedora_claude_image, has_claude: true);

// ---------------------------------------------------------------------------
// Cleanup — runs when the test process exits, after all tests complete
// ---------------------------------------------------------------------------

#[ctor::dtor]
fn cleanup_images() {
    for tag in [
        "openshell-test-ubuntu:integration",
        "openshell-test-ubuntu-claude:integration",
        "openshell-test-fedora:integration",
        "openshell-test-fedora-claude:integration",
    ] {
        Command::new("podman")
            .args(["rmi", "--force", tag])
            .status()
            .ok();
    }
}
