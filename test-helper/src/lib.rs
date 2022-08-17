use proc_macro::TokenStream;
use quote::{format_ident, quote};
use walkdir::{DirEntry, WalkDir};

const IGNORE: &[&str] = &[
    "rust-custom-version",
    "rust-rocket",
    "haskell-stack",
    "zig-gyro",
    "rust-ring",
    "rust-openssl",
    "rust-custom-toolchain",
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
                    ".version" => "[version]",
                    ".setup.archive" => "[archive]",
                    ".setup.baseImage" => "[baseImage]",
                });
            }
        }

        fn simple_gen_plan(path: &str) -> ::nixpacks::nixpacks::plan::BuildPlan {
            if let Ok(raw_env) = ::std::fs::read_to_string(format!("{}/test.env", path)) {
                let env = ::dotenv_parser::parse_dotenv(&raw_env).unwrap();
                let opts = ::nixpacks::nixpacks::plan::generator::GeneratePlanOptions {
                    pin_pkgs: env.get("PIN_PKGS").is_some(),
                    custom_start_cmd: env.get("CUSTOM_START_CMD").map(|cmd| cmd.to_string()),
                    custom_pkgs: env
                        .get("CUSTOM_PKGS")
                        .map(|pkgs| pkgs.split(',')
                        .map(|pkg| ::nixpacks::nixpacks::nix::pkg::Pkg::new(pkg)).collect())
                        .unwrap_or_default(),
                    ..::nixpacks::nixpacks::plan::generator::GeneratePlanOptions::default()
                };

                return ::nixpacks::generate_build_plan(
                    path,
                    env.get("ENVS").map(|envs| envs.split(", ").collect()).unwrap_or_default(),
                    &opts
                ).unwrap();
            }

            ::nixpacks::generate_build_plan(
                path,
                ::std::vec::Vec::new(),
                &::nixpacks::nixpacks::plan::generator::GeneratePlanOptions::default()
            ).unwrap()
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
