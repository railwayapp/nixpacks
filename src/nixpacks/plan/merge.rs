use super::{
    phase::{Phase, StartPhase},
    BuildPlan,
};

pub trait Mergeable {
    fn merge(c1: &Self, c2: &Self) -> Self;
}

impl Mergeable for BuildPlan {
    fn merge(c1: &BuildPlan, c2: &BuildPlan) -> BuildPlan {
        let mut new_plan = c1.clone();
        let plan2 = c2.clone();

        new_plan.providers = fill_auto_in_vec(new_plan.providers.clone(), plan2.providers.clone());

        new_plan.static_assets = match (new_plan.static_assets, plan2.static_assets) {
            (None, assets) => assets,
            (assets, None) => assets,
            (Some(assets1), Some(assets2)) => {
                let mut assets = assets1.clone();
                assets.extend(assets2.clone());
                Some(assets)
            }
        };

        new_plan.variables = match (new_plan.variables, plan2.variables) {
            (None, vars) => vars,
            (vars, None) => vars,
            (Some(vars1), Some(vars2)) => {
                let mut vars = vars1.clone();
                vars.extend(vars2.clone());
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
            (None, s) => s,
            (s, None) => s,
            (Some(s1), Some(s2)) => Some(StartPhase::merge(&s1, &s2)),
        };

        new_plan.resolve_phase_names();
        new_plan
    }
}

impl Mergeable for Phase {
    fn merge(c1: &Phase, c2: &Phase) -> Phase {
        let mut phase = c1.clone();
        let c2 = c2.clone();
        phase.nixpacks_archive = c2
            .nixpacks_archive
            .or_else(|| phase.nixpacks_archive.clone());

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

/// Fills in the `"..."`'s or `"@auto"`'s in `replacer` with the values from the `original`
///
/// ```
/// let arr = fill_auto_in_vec(
///   Some(vec!["a", "b", "c"]),
///   Some(vec!["x", "...", "z"])
/// );
/// assert_eq!(Some(vec!["x", "...", "a", "b", "c", "z"]), arr);
/// ```
fn fill_auto_in_vec(
    original: Option<Vec<String>>,
    replacer: Option<Vec<String>>,
) -> Option<Vec<String>> {
    if let Some(replacer) = replacer {
        let original = original.unwrap_or_default();
        let modified = replacer
            .into_iter()
            .flat_map(|x| {
                let v = x.clone();
                if v == "@auto".to_string() || v == "...".to_string() {
                    let mut fill = vec![v.clone()];
                    fill.append(&mut original.clone());
                    fill
                } else {
                    vec![x]
                }
            })
            .collect::<Vec<_>>();

        Some(modified)
    } else {
        original
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn vso(v: Vec<&str>) -> Option<Vec<String>> {
        Some(v.into_iter().map(|x| x.to_string()).collect())
    }

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

    #[test]
    fn test_fill_auto_in_vec() {
        assert_eq!(
            vec!["x", "...", "z"],
            fill_auto_in_vec(None, vso(vec!["x", "...", "z"])).unwrap()
        );
        assert_eq!(
            vec!["a", "b", "c"],
            fill_auto_in_vec(vso(vec!["a", "b", "c"]), None).unwrap()
        );
        assert_eq!(
            vec!["x", "...", "a", "b", "c", "z"],
            fill_auto_in_vec(vso(vec!["a", "b", "c"]), vso(vec!["x", "...", "z"])).unwrap()
        );
    }
}