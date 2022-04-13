use serde::{Deserialize, Serialize};

use super::{
    environment::EnvironmentVariables,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildPlan {
    pub version: String,
    pub setup: SetupPhase,
    pub install: InstallPhase,
    pub build: BuildPhase,
    pub start: StartPhase,
    pub variables: EnvironmentVariables,
}
