use proc_macro::TokenStream;
use quote::{format_ident, quote};
use walkdir::{DirEntry, WalkDir};

const IGNORE: &[&str] = &[
    "custom-plan-path",
    "rust-custom-version",
    "rust-rocket",
    "haskell-stack",
    "zig-gyro",
    "rust-ring",
    "rust-openssl",
    "rust-custom-toolchain",
    "rust-cargo-workspaces",
    "rust-cargo-workspaces-glob",
    "rust-multiple-bins",
    "ruby-no-version",
];

fn get_examples() -> Vec<String> {
    let mut current_dir = std::env::current_dir().unwrap();

    current_dir.push("examples");

    let walker = WalkDir::new(&current_dir).max_depth(1);

    walker
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
        .map(DirEntry::into_path)
        .filter(|path| path.is_dir())
        .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
        .filter(|path| !IGNORE.contains(&path.as_str()))
        .collect()
}

#[proc_macro]
pub fn generate_plan_tests(_tokens: TokenStream) -> TokenStream {
    let mut examples = get_examples();
    let mut tests = Vec::with_capacity(examples.len());

    // First element is always "examples"
    examples.remove(0);

    tests.push(quote! {
        macro_rules! assert_plan_snapshot {
            ($plan:expr) => {
                ::insta::assert_json_snapshot!($plan, {
                    ".buildImage" => "[build_image]",
                    ".phases.*.nixpkgsArchive" => "[archive]",
                });
            }
        }

        fn simple_gen_plan(path: &str) -> ::nixpacks::nixpacks::plan::BuildPlan {
            if let Ok(raw_env) = ::std::fs::read_to_string(format!("{}/test.env", path)) {
                let env = ::dotenv_parser::parse_dotenv(&raw_env).unwrap();
                let plan = ::nixpacks::nixpacks::plan::BuildPlan {
                    phases: Some(::std::collections::BTreeMap::from([(
                        "setup".to_string(),
                        ::nixpacks::nixpacks::plan::phase::Phase {
                            nix_pkgs: env.get("CUSTOM_PKGS").map(|pkgs| {
                                pkgs.split(',')
                                    .map(|pkg| pkg.to_string())
                                    .collect::<Vec<_>>()
                            }),
                            ..Default::default()
                        },
                    )])),
                    start_phase: Some(::nixpacks::nixpacks::plan::phase::StartPhase {
                        cmd: env.get("CUSTOM_START_CMD").map(|cmd| cmd.to_string()),
                        ..Default::default()
                    }),
                    ..::nixpacks::nixpacks::plan::BuildPlan::default()
                };
                let opts = ::nixpacks::nixpacks::plan::generator::GeneratePlanOptions {
                    plan: Some(plan),
                    ..Default::default()
                };

                return ::nixpacks::generate_build_plan(
                    path,
                    env.get("ENVS")
                        .map(|envs| envs.split(", ").collect())
                        .unwrap_or_default(),
                    &opts,
                )
                .unwrap();
            }

            ::nixpacks::generate_build_plan(
                path,
                ::std::vec::Vec::new(),
                &::nixpacks::nixpacks::plan::generator::GeneratePlanOptions::default(),
            )
            .unwrap()
        }
    });

    for example in examples {
        let test_name = format_ident!("{}", example.replace('-', "_"));
        let test = quote! {
            #[test]
            fn #test_name() {
                let plan = simple_gen_plan(&format!("./examples/{}", #example));
                assert_plan_snapshot!(plan);
            }
        };

        tests.push(test);
    }

    tests
        .into_iter()
        .collect::<proc_macro2::TokenStream>()
        .into()
}
