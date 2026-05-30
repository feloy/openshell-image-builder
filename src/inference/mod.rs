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

mod anthropic;
mod vertexai;

#[cfg(test)]
pub use anthropic::AnthropicInference;
#[cfg(test)]
pub use vertexai::VertexAiInference;

use clap::ValueEnum;

pub trait Inference {
    /// Returns a network policy YAML fragment scoped to the given agent binary.
    fn policy_yaml(&self, agent_binary: &str) -> String;
}

#[derive(Clone, ValueEnum)]
pub enum InferenceKind {
    Anthropic,
    #[value(name = "vertexai")]
    VertexAi,
}

pub fn from_kind(kind: InferenceKind) -> Box<dyn Inference> {
    match kind {
        InferenceKind::Anthropic => Box::new(anthropic::AnthropicInference),
        InferenceKind::VertexAi => Box::new(vertexai::VertexAiInference),
    }
}
