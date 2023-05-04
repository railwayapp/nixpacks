use super::{
    phase::{Phase, StartPhase},
    utils::fill_auto_in_vec,
    BuildPlan,
};

/// Types that impl this trait can be pairwise combined.
pub trait Mergeable {
    fn merge(c1: &Self, c2: &Self) -> Self;
}

impl Mergeable for BuildPlan {
    /// Given two BuildPlans, produce a third BuildPlan containing the data of both.
    fn merge(c1: &BuildPlan, c2: &BuildPlan) -> BuildPlan {
        let mut new_plan = c1.clone();
        let plan2 = c2.clone();

        new_plan.providers = fill_auto_in_vec(new_plan.providers.clone(), plan2.providers.clone());
        new_plan.build_image = plan2.build_image.or(new_plan.build_image);

        new_plan.static_assets = match (new_plan.static_assets, plan2.static_assets) {
            (None, assets) | (assets, None) => assets,
            (Some(assets1), Some(assets2)) => {
                let mut assets = assets1;
                assets.extend(assets2);
                Some(assets)
            }
        };

        new_plan.variables = match (new_plan.variables, plan2.variables) {
            (None, vars) | (vars, None) => vars,
            (Some(vars1), Some(vars2)) => {
                let mut vars = vars1;
                vars.extend(vars2);
                Some(vars)
            }
        };

        if new_plan.phases.is_none() {
            new_plan.phases = plan2.phases;
        } else {
            for (name, c2_phase) in plan2.phases.clone().unwrap_or_default() {
                let phase = new_plan.remove_phase(&name);
                let phase = phase.unwrap_or_else(|| {
                    let mut phase = Phase::new(name.clone());
                    if name == "install" {
                        phase.depends_on_phase("setup");
                    } else if name == "build" {
                        phase.depends_on_phase("install");
                    };

                    phase
                });

                let merged_phase = Phase::merge(&phase, &c2_phase);
                new_plan.add_phase(merged_phase);
            }
        };

        new_plan.start_phase = match (new_plan.start_phase, plan2.start_phase) {
            (None, s) | (s, None) => s,
            (Some(s1), Some(s2)) => Some(StartPhase::merge(&s1, &s2)),
        };

        new_plan.resolve_phase_names();
        new_plan
    }
}

impl Mergeable for Phase {
    /// Given two Phases, produce a third Phase containing the data of both.
    fn merge(c1: &Phase, c2: &Phase) -> Phase {
        let mut phase = c1.clone();
        let c2 = c2.clone();
        phase.nixpkgs_archive = c2.nixpkgs_archive.or_else(|| phase.nixpkgs_archive.clone());

        phase.cmds = fill_auto_in_vec(phase.cmds.clone(), c2.cmds);
        phase.depends_on = fill_auto_in_vec(phase.depends_on.clone(), c2.depends_on);
        phase.nix_pkgs = fill_auto_in_vec(phase.nix_pkgs.clone(), c2.nix_pkgs);
        phase.nix_libs = fill_auto_in_vec(phase.nix_libs.clone(), c2.nix_libs);
        phase.apt_pkgs = fill_auto_in_vec(phase.apt_pkgs.clone(), c2.apt_pkgs);
        phase.nix_overlays = fill_auto_in_vec(phase.nix_overlays.clone(), c2.nix_overlays);
        phase.only_include_files =
            fill_auto_in_vec(phase.only_include_files.clone(), c2.only_include_files);
        phase.cache_directories =
            fill_auto_in_vec(phase.cache_directories.clone(), c2.cache_directories);
        phase.paths = fill_auto_in_vec(phase.paths.clone(), c2.paths);

        phase
    }
}

impl Mergeable for StartPhase {
    /// Given two StartPhases, produce a third StartPhase containing the data of both.
    fn merge(c1: &StartPhase, c2: &StartPhase) -> StartPhase {
        let mut start_phase = c1.clone();
        let c2 = c2.clone();
        start_phase.cmd = c2.cmd.or_else(|| start_phase.cmd.clone());
        start_phase.run_image = c2.run_image.or_else(|| start_phase.run_image.clone());
        start_phase.only_include_files = fill_auto_in_vec(
            start_phase.only_include_files.clone(),
            c2.only_include_files,
        );
        start_phase
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merge_plan() {
        let merged = BuildPlan::merge(
            &BuildPlan::from_toml(
                r#"
                [phases.setup]
                nixPkgs = ["cowsay"]
                "#,
            )
            .unwrap(),
            &BuildPlan::from_toml("").unwrap(),
        );

        assert_eq!(
            BuildPlan::from_toml(
                r#"
                [phases.setup]
                nixPkgs = ["cowsay"]
                "#,
            )
            .unwrap(),
            merged
        );

        let merged = BuildPlan::merge(
            &BuildPlan::from_toml(
                r#"
                [phases.setup]
                nixPkgs = ["nodejs", "yarn"]

                [phases.build]
                cmds = ["yarn run build"]

                [start]
                cmd = "yarn run start"
                "#,
            )
            .unwrap(),
            &BuildPlan::from_toml(
                r#"
                [phases.setup]
                nixPkgs = ["...", "cowsay"]

                [start]
                cmd = "yarn run client:start"
                "#,
            )
            .unwrap(),
        );

        assert_eq!(
            BuildPlan::from_toml(
                r#"
                [phases.setup]
                nixPkgs = ["...", "nodejs", "yarn", "cowsay"]

                [phases.build]
                cmds = ["yarn run build"]

                [start]
                cmd = "yarn run client:start"
                "#,
            )
            .unwrap(),
            merged
        );

        let merged = BuildPlan::merge(
            &BuildPlan::from_toml(
                r#"
                [phases.setup]
                nixPkgs = ["nodejs", "yarn"]
                "#,
            )
            .unwrap(),
            &BuildPlan::from_toml(
                r#"
                providers = []

                [phases.setup]
                nixPkgs = ["cowsay"]
                "#,
            )
            .unwrap(),
        );

        assert_eq!(
            BuildPlan::from_toml(
                r#"
                providers = []

                [phases.setup]
                nixPkgs = ["cowsay"]
                "#,
            )
            .unwrap(),
            merged
        );
    }
}
