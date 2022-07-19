use anyhow::Result;
use nixpacks::providers::node::NODE_OVERLAY;
use nixpacks::{
    generate_build_plan,
    nixpacks::{
        nix::pkg::Pkg,
        plan::{generator::GeneratePlanOptions, BuildPlan},
    },
};
use std::env::consts::ARCH;

fn simple_gen_plan(path: &str) -> BuildPlan {
    generate_build_plan(path, Vec::new(), &GeneratePlanOptions::default()).unwrap()
}

#[test]
fn test_node() -> Result<()> {
    let plan = simple_gen_plan("./examples/node");
    assert_eq!(
        plan.install.clone().unwrap().cmds,
        Some(vec!["npm ci".to_string()])
    );
    assert_eq!(
        plan.install.unwrap().cache_directories,
        Some(vec!["/root/.npm".to_string()])
    );
    assert_eq!(plan.build.unwrap().cmds, None);
    assert_eq!(plan.start.unwrap().cmd, Some("npm run start".to_string()));

    Ok(())
}

#[test]
fn test_node_no_lockfile() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-no-lockfile-canvas");
    assert_eq!(plan.install.unwrap().cmds, Some(vec!["npm i".to_string()]));
    assert_eq!(plan.build.unwrap().cmds, None);
    assert_eq!(plan.start.unwrap().cmd, Some("npm run start".to_string()));
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![
            Pkg::new("nodejs"),
            Pkg::new("npm-8_x").from_overlay(NODE_OVERLAY)
        ]
    );

    Ok(())
}

#[test]
fn test_node_npm_old_lockfile() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-npm-old-lockfile");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![
            Pkg::new("nodejs"),
            Pkg::new("npm-6_x").from_overlay(NODE_OVERLAY)
        ]
    );

    Ok(())
}

#[test]
fn test_npm() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-npm");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["npm run build".to_string()])
    );
    assert_eq!(plan.start.unwrap().cmd, Some("npm run start".to_string()));
    assert_eq!(
        plan.variables.clone().unwrap().get("NODE_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        plan.variables.unwrap().get("NPM_CONFIG_PRODUCTION"),
        Some(&"false".to_string())
    );

    Ok(())
}

#[test]
fn test_node_no_scripts() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-no-scripts");
    assert_eq!(plan.build.unwrap().cmds, None);
    assert_eq!(plan.start.unwrap().cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_node_custom_version() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-custom-version");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![
            Pkg::new("nodejs-18_x"),
            Pkg::new("npm-8_x").from_overlay(NODE_OVERLAY)
        ]
    );

    Ok(())
}

#[test]
fn test_node_monorepo() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-monorepo");
    assert_eq!(
        plan.install.clone().unwrap().cmds,
        Some(vec!["yarn install --frozen-lockfile".to_string()])
    );
    assert_eq!(
        plan.install.unwrap().cache_directories,
        Some(vec!["/usr/local/share/.cache/yarn/v6".to_string()])
    );
    assert_eq!(plan.build.unwrap().cmds, None);

    Ok(())
}

#[test]
fn test_yarn() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-yarn");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["yarn install --frozen-lockfile".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["yarn run build".to_string()])
    );
    assert_eq!(plan.start.unwrap().cmd, Some("yarn run start".to_string()));
    assert_eq!(
        plan.variables.clone().unwrap().get("NODE_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        plan.variables.unwrap().get("NPM_CONFIG_PRODUCTION"),
        Some(&"false".to_string())
    );

    Ok(())
}

#[test]
fn test_yarn_berry() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-yarn-berry");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec![
            "yarn set version berry && yarn install --immutable --check-cache".to_string()
        ])
    );
    Ok(())
}

#[test]
fn test_yarn_custom_version() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-yarn-custom-node-version");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![
            Pkg::new("nodejs-14_x"),
            Pkg::new("yarn-1_x").from_overlay(NODE_OVERLAY)
        ]
    );

    Ok(())
}

#[test]
fn test_pnpm() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-pnpm");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![
            Pkg::new("nodejs"),
            Pkg::new("pnpm-6_x").from_overlay(NODE_OVERLAY)
        ]
    );
    assert_eq!(
        plan.install.clone().unwrap().cmds,
        Some(vec!["pnpm i --frozen-lockfile".to_string()])
    );
    assert_eq!(
        plan.install.unwrap().cache_directories,
        Some(vec!["/root/.cache/pnpm".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["pnpm run build".to_string()])
    );
    assert_eq!(plan.start.unwrap().cmd, Some("pnpm run start".to_string()));
    assert_eq!(
        plan.variables.clone().unwrap().get("NODE_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        plan.variables.unwrap().get("NPM_CONFIG_PRODUCTION"),
        Some(&"false".to_string())
    );

    Ok(())
}

#[test]
fn test_bun() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-bun");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![Pkg::new("bun").from_overlay(NODE_OVERLAY)]
    );
    assert_eq!(
        plan.install.clone().unwrap().cmds,
        Some(vec!["bun i --no-save".to_string()])
    );
    assert_eq!(
        plan.install.unwrap().cache_directories,
        Some(vec!["/root/.bun".to_string()])
    );
    assert_eq!(plan.start.unwrap().cmd, Some("bun run start".to_string()));
    assert_eq!(
        plan.variables.clone().unwrap().get("NODE_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        plan.variables.unwrap().get("NPM_CONFIG_PRODUCTION"),
        Some(&"false".to_string())
    );

    Ok(())
}

#[test]
fn test_bun_no_start() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-bun-no-start");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![Pkg::new("bun").from_overlay(NODE_OVERLAY)]
    );
    assert_eq!(
        plan.install.clone().unwrap().cmds,
        Some(vec!["bun i --no-save".to_string()])
    );
    assert_eq!(
        plan.install.unwrap().cache_directories,
        Some(vec!["/root/.bun".to_string()])
    );
    assert_eq!(plan.start.unwrap().cmd, Some("bun index.ts".to_string()));
    assert_eq!(
        plan.variables.clone().unwrap().get("NODE_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        plan.variables.unwrap().get("NPM_CONFIG_PRODUCTION"),
        Some(&"false".to_string())
    );

    Ok(())
}

#[test]
fn test_bun_web_server() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-bun-no-start");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![Pkg::new("bun").from_overlay(NODE_OVERLAY)]
    );
    assert_eq!(
        plan.install.clone().unwrap().cmds,
        Some(vec!["bun i --no-save".to_string()])
    );
    assert_eq!(
        plan.install.unwrap().cache_directories,
        Some(vec!["/root/.bun".to_string()])
    );
    assert_eq!(plan.start.unwrap().cmd, Some("bun index.ts".to_string()));
    assert_eq!(
        plan.variables.clone().unwrap().get("NODE_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        plan.variables.unwrap().get("NPM_CONFIG_PRODUCTION"),
        Some(&"false".to_string())
    );

    Ok(())
}

#[test]
fn test_pnpm_v7() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-pnpm-v7");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![
            Pkg::new("nodejs"),
            Pkg::new("pnpm-7_x").from_overlay(NODE_OVERLAY)
        ]
    );

    Ok(())
}

#[test]
fn test_pnpm_custom_version() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-pnpm-custom-node-version");
    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![
            Pkg::new("nodejs-14_x"),
            Pkg::new("pnpm-6_x").from_overlay(NODE_OVERLAY)
        ]
    );

    Ok(())
}

#[test]
fn test_go() -> Result<()> {
    let plan = simple_gen_plan("./examples/go");
    assert_eq!(
        plan.build.clone().unwrap().cmds,
        Some(vec!["go build -o out main.go".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cache_directories,
        Some(vec!["/root/.cache/go-build".to_string()])
    );
    assert_eq!(plan.start.clone().unwrap().cmd, Some("./out".to_string()));
    assert!(plan.start.unwrap().run_image.is_some());

    Ok(())
}

#[test]
fn test_go_cgo_enabled() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/go",
        vec!["CGO_ENABLED=1"],
        &GeneratePlanOptions::default(),
    )?;
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["go build -o out main.go".to_string()])
    );
    assert_eq!(plan.start.clone().unwrap().cmd, Some("./out".to_string()));
    assert!(plan.start.unwrap().run_image.is_none());

    Ok(())
}

#[test]
fn test_go_mod() -> Result<()> {
    let plan = simple_gen_plan("./examples/go-mod");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["go build -o out".to_string()])
    );
    assert_eq!(plan.start.unwrap().cmd, Some("./out".to_string()));

    Ok(())
}

#[test]
fn test_go_custom_version() -> Result<()> {
    let plan = simple_gen_plan("./examples/go-custom-version");
    assert_eq!(plan.setup.unwrap().pkgs, vec![Pkg::new("go_1_18")]);

    Ok(())
}

#[test]
fn test_deno() -> Result<()> {
    let plan = simple_gen_plan("./examples/deno");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["deno cache src/index.ts".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("deno run --allow-all src/index.ts".to_string())
    );

    Ok(())
}

#[test]
fn test_deno_fresh() -> Result<()> {
    let plan = simple_gen_plan("./examples/deno-fresh");
    assert_eq!(plan.build.unwrap().cmds, None);
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("deno run -A dev.ts".to_string())
    );

    Ok(())
}

#[test]
fn test_csharp_api() -> Result<()> {
    let plan = simple_gen_plan("./examples/csharp-api");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["dotnet restore".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            "dotnet publish --no-restore -c Release -o out".to_string()
        ])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("./out/csharp-api".to_string())
    );

    Ok(())
}

#[test]
fn test_fsharp_api() -> Result<()> {
    let plan = simple_gen_plan("./examples/fsharp-api");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["dotnet restore".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            "dotnet publish --no-restore -c Release -o out".to_string()
        ])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("./out/fsharp-api".to_string())
    );

    Ok(())
}

#[test]
fn test_csharp_cli() -> Result<()> {
    let plan = simple_gen_plan("./examples/csharp-cli");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["dotnet restore".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            "dotnet publish --no-restore -c Release -o out".to_string()
        ])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("./out/csharp-cli".to_string())
    );

    Ok(())
}

#[test]
fn test_procfile() -> Result<()> {
    let plan = simple_gen_plan("./examples/procfile");
    assert_eq!(plan.start.unwrap().cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_custom_pkgs() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/shell-hello",
        Vec::new(),
        &GeneratePlanOptions {
            custom_start_cmd: Some("./start.sh".to_string()),
            custom_pkgs: vec![Pkg::new("cowsay")],
            ..Default::default()
        },
    )?;
    assert_eq!(plan.setup.unwrap().pkgs, vec![Pkg::new("cowsay")]);
    assert_eq!(plan.start.unwrap().cmd, Some("./start.sh".to_string()));

    Ok(())
}

#[test]
fn test_pin_archive() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/shell-hello",
        Vec::new(),
        &GeneratePlanOptions {
            pin_pkgs: true,
            ..Default::default()
        },
    )?;
    assert!(plan.setup.unwrap().archive.is_some());

    Ok(())
}

#[test]
fn test_custom_rust_version() -> Result<()> {
    let plan = simple_gen_plan("./examples/rust-custom-version");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            format!("cargo build --release --target {}-unknown-linux-musl", ARCH),
            format!(
                "cp target/{}-unknown-linux-musl/release/rust-custom-version rust-custom-version",
                ARCH
            )
        ])
    );
    assert_eq!(
        plan.setup
            .unwrap()
            .pkgs
            .iter()
            .filter(|p| p.name.contains("1.56.0"))
            .count(),
        1
    );

    Ok(())
}

#[test]
fn test_rust_rocket() -> Result<()> {
    let plan = simple_gen_plan("./examples/rust-rocket");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            format!("cargo build --release --target {}-unknown-linux-musl", ARCH),
            format!(
                "cp target/{}-unknown-linux-musl/release/rocket rocket",
                ARCH
            )
        ])
    );
    assert!(plan.start.clone().unwrap().cmd.is_some());
    assert_eq!(
        plan.start.clone().unwrap().cmd.unwrap(),
        "./rocket".to_string()
    );
    assert!(plan.start.unwrap().run_image.is_some());

    Ok(())
}

#[test]
fn test_rust_rocket_no_musl() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/rust-rocket",
        vec!["NIXPACKS_NO_MUSL=1"],
        &GeneratePlanOptions::default(),
    )?;
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            "cargo build --release".to_string(),
            "cp target/release/rocket rocket".to_string()
        ])
    );
    assert!(plan
        .start
        .clone()
        .unwrap()
        .cmd
        .unwrap()
        .contains("./rocket"));
    assert!(plan.start.unwrap().run_image.is_none());

    Ok(())
}

#[test]
pub fn test_python() -> Result<()> {
    let plan = simple_gen_plan("./examples/python");
    assert_eq!(plan.build.unwrap().cmds, None);
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["python -m venv /opt/venv && . /opt/venv/bin/activate && pip install -r requirements.txt".to_string()])
    );
    assert_eq!(plan.start.unwrap().cmd, Some("python main.py".to_string()));

    Ok(())
}

#[test]
pub fn test_python_poetry() -> Result<()> {
    let plan = simple_gen_plan("./examples/python-poetry");
    assert_eq!(plan.build.unwrap().cmds, None);
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["python -m venv /opt/venv && . /opt/venv/bin/activate && pip install poetry==$NIXPACKS_POETRY_VERSION && poetry install --no-dev --no-interaction --no-ansi".to_string()])
    );
    assert_eq!(plan.start.unwrap().cmd, Some("python main.py".to_string()));

    Ok(())
}

#[test]
fn test_node_main_file() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-main-file");
    assert_eq!(plan.build.unwrap().cmds, None);
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("node src/index.js".to_string())
    );

    Ok(())
}

#[test]
pub fn test_python_setuptools() -> Result<()> {
    let plan = simple_gen_plan("./examples/python-setuptools");
    assert_eq!(plan.install.unwrap().cmds, Some(vec!["python -m venv /opt/venv && . /opt/venv/bin/activate && pip install --upgrade build setuptools && pip install .".to_string()]));
    assert_eq!(plan.build.unwrap().cmds, None);
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("python -m nixpacks-setuptools".to_string())
    );

    Ok(())
}

#[test]
fn test_node_main_file_doesnt_exist() -> Result<()> {
    let plan = simple_gen_plan("./examples/node-main-file-not-exist");
    assert_eq!(plan.build.unwrap().cmds, None);
    assert_eq!(plan.start.unwrap().cmd, Some("node index.js".to_string()));

    Ok(())
}

#[test]
fn test_haskell_stack() -> Result<()> {
    let plan = simple_gen_plan("./examples/haskell-stack");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["stack setup".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["stack install".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("/root/.local/bin/haskell-stack-exe".to_string())
    );
    Ok(())
}

#[test]
fn test_crystal() -> Result<()> {
    let plan = simple_gen_plan("./examples/crystal");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["shards install".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["shards build --release".to_string()])
    );
    assert_eq!(plan.start.unwrap().cmd, Some("./bin/crystal".to_string()));
    Ok(())
}

#[test]
fn test_overriding_environment_variables() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/node-variables",
        vec!["NODE_ENV=test"],
        &GeneratePlanOptions::default(),
    )?;
    assert_eq!(
        plan.variables.unwrap().get("NODE_ENV"),
        Some(&"test".to_string())
    );

    Ok(())
}

#[test]
fn test_config_from_environment_variables() -> Result<()> {
    let plan = generate_build_plan(
        "./examples/shell-hello",
        vec![
            "NIXPACKS_PKGS=cowsay ripgrep",
            "NIXPACKS_INSTALL_CMD=install",
            "NIXPACKS_BUILD_CMD=build",
            "NIXPACKS_START_CMD=start",
            "NIXPACKS_RUN_IMAGE=alpine",
            "NIXPACKS_INSTALL_CACHE_DIRS=/tmp,foobar",
            "NIXPACKS_BUILD_CACHE_DIRS=/build,barbaz",
        ],
        &GeneratePlanOptions::default(),
    )?;

    assert_eq!(
        plan.setup.unwrap().pkgs,
        vec![Pkg::new("cowsay"), Pkg::new("ripgrep")]
    );

    assert_eq!(
        plan.install.clone().unwrap().cmds,
        Some(vec!["install".to_string()])
    );
    assert_eq!(
        plan.install.unwrap().cache_directories,
        Some(vec!["/tmp".to_string(), "foobar".to_string()])
    );

    assert_eq!(
        plan.build.clone().unwrap().cmds,
        Some(vec!["build".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cache_directories,
        Some(vec!["/build".to_string(), "barbaz".to_string()])
    );

    assert_eq!(plan.start.clone().unwrap().cmd, Some("start".to_string()));
    assert_eq!(plan.start.unwrap().run_image, Some("alpine".to_string()));

    Ok(())
}

#[test]
fn test_staticfile() -> Result<()> {
    let plan = simple_gen_plan("./examples/staticfile");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            "mkdir /etc/nginx/ /var/log/nginx/ /var/cache/nginx/".to_string()
        ])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("[[ -z \"${PORT}\" ]] && echo \"Environment variable PORT not found. Using PORT 80\" || sed -i \"s/0.0.0.0:80/$PORT/g\" /assets/nginx.conf && nginx -c /assets/nginx.conf".to_string())
    );
    Ok(())
}

#[test]
fn test_php_vanilla() -> Result<()> {
    let plan = simple_gen_plan("./examples/php-vanilla");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec![
            "mkdir -p /var/log/nginx && mkdir -p /var/cache/nginx".to_string()
        ])
    );
    assert_eq!(plan.build.unwrap().cmds, None);
    assert!(plan
        .start
        .unwrap()
        .cmd
        .unwrap()
        .contains("nginx -c /nginx.conf"));
    Ok(())
}

#[test]
fn test_php_laravel() -> Result<()> {
    let plan = simple_gen_plan("./examples/php-laravel");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec![
            "mkdir -p /var/log/nginx && mkdir -p /var/cache/nginx".to_string(),
            "composer install".to_string(),
            "npm i".to_string()
        ])
    );
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["npm run prod".to_string()])
    );
    assert!(plan
        .start
        .unwrap()
        .cmd
        .unwrap()
        .contains("nginx -c /nginx.conf"));
    Ok(())
}

#[test]
fn test_dart() -> Result<()> {
    let plan = simple_gen_plan("./examples/dart");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["dart pub get".to_string()])
    );
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["dart compile exe bin/console_simple.dart".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("./bin/console_simple.exe".to_string())
    );

    Ok(())
}

#[test]
fn test_swift() -> Result<()> {
    let plan = simple_gen_plan("./examples/swift");

    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            "CC=clang++ swift build -c release --static-swift-stdlib".to_string(),
            "cp ./.build/release/swift ./swift && rm -rf ./.build".to_string()
        ])
    );

    assert_eq!(plan.start.unwrap().cmd, Some("./swift".to_owned()));
    Ok(())
}

#[test]
fn test_swift_vapor() -> Result<()> {
    let plan = simple_gen_plan("./examples/swift-vapor");

    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec![
            "CC=clang++ swift build -c release --static-swift-stdlib".to_string(),
            "cp ./.build/release/Run ./Run && rm -rf ./.build".to_string()
        ])
    );

    assert_eq!(plan.start.unwrap().cmd, Some("./Run".to_owned()));

    Ok(())
}

#[test]
fn test_java_maven() -> Result<()> {
    let plan = simple_gen_plan("./examples/java-maven");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["mvn -DoutputFile=target/mvn-dependency-list.log -B -DskipTests clean dependency:list install".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("java -Dserver.port=$PORT $JAVA_OPTS -jar target/*jar".to_string())
    );
    Ok(())
}

#[test]
fn test_java_maven_wrapper() -> Result<()> {
    let plan = simple_gen_plan("./examples/java-maven-wrapper");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["./mvnw -DoutputFile=target/mvn-dependency-list.log -B -DskipTests clean dependency:list install".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("java -Dserver.port=$PORT $JAVA_OPTS -jar target/*jar".to_string())
    );
    Ok(())
}

#[test]
fn test_zig() -> Result<()> {
    let plan = simple_gen_plan("./examples/zig");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["zig build -Drelease-safe=true".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("./zig-out/bin/zig".to_string())
    );
    Ok(())
}

#[cfg(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "i386"))]
#[test]
fn test_zig_gyro() -> Result<()> {
    let plan = simple_gen_plan("./examples/zig-gyro");
    assert_eq!(
        plan.build.unwrap().cmds,
        Some(vec!["zig build -Drelease-safe=true".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("./zig-out/bin/zig-gyro".to_string())
    );
    assert!(plan
        .install
        .unwrap()
        .cmds
        .unwrap()
        .get(0)
        .unwrap()
        .contains("mkdir /gyro"));
    Ok(())
}

#[test]
fn test_ruby_rails() -> Result<()> {
    let plan = simple_gen_plan("./examples/ruby-rails-postgres");
    assert_eq!(
        plan.setup.unwrap().apt_pkgs,
        Some(vec!["procps".to_string(), "libpq-dev".to_string()])
    );
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["bundle install".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some(
            "rake db:migrate && bundle exec bin/rails server -b 0.0.0.0 -p ${PORT:-3000}"
                .to_string()
        )
    );
    Ok(())
}

#[test]
fn test_ruby_sinatra() -> Result<()> {
    let plan = simple_gen_plan("./examples/ruby-sinatra");
    assert_eq!(
        plan.install.unwrap().cmds,
        Some(vec!["bundle install".to_string()])
    );
    assert_eq!(
        plan.start.unwrap().cmd,
        Some("RACK_ENV=production bundle exec puma".to_string())
    );
    Ok(())
}
