use serde::{Deserialize, Serialize};

use super::{
    environment::EnvironmentVariables,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildPlan {
    pub version: Option<String>,
    pub setup: Option<SetupPhase>,
    pub install: Option<InstallPhase>,
    pub build: Option<BuildPhase>,
    pub start: Option<StartPhase>,
    pub variables: Option<EnvironmentVariables>,
}
