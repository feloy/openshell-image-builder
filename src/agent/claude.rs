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

use super::Agent;

pub struct ClaudeAgent;

impl Agent for ClaudeAgent {
    fn install(&self) -> String {
        "RUN curl -fsSL https://claude.ai/install.sh | bash\nENV PATH=/sandbox/.local/bin:$PATH"
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_is_nonempty() {
        assert!(!ClaudeAgent.install().is_empty());
    }

    #[test]
    fn install_contains_claude_installer() {
        assert!(
            ClaudeAgent
                .install()
                .contains("https://claude.ai/install.sh")
        );
    }

    #[test]
    fn install_adds_local_bin_to_path() {
        assert!(
            ClaudeAgent
                .install()
                .contains("ENV PATH=/sandbox/.local/bin:$PATH")
        );
    }
}
