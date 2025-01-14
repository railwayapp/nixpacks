use super::Provider;
use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::Environment,
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::Result;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write as _;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Staticfile {
    pub root: Option<String>,
    pub directory: Option<String>,
    pub gzip: Option<String>,
    pub status_code: Option<HashMap<u32, String>>,
}

pub struct StaticfileProvider {}

impl Provider for StaticfileProvider {
    fn name(&self) -> &'static str {
        "staticfile"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Staticfile")
            || app.includes_directory("public")
            || app.includes_directory("index")
            || app.includes_directory("dist")
            || app.includes_file("index.html"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let mut setup = Phase::setup(Some(vec![Pkg::new("nginx")]));
        setup.add_cmd("mkdir /etc/nginx/ /var/log/nginx/ /var/cache/nginx/");

        // shell command to edit 0.0.0.0:80 to $PORT
        let shell_cmd = "[[ -z \"${PORT}\" ]] && echo \"Environment variable PORT not found. Using PORT 80\" || sed -i \"s/0.0.0.0:80/$PORT/g\"";
        let start = StartPhase::new(format!(
            "{shell_cmd} {conf_location} && nginx -c {conf_location}",
            shell_cmd = shell_cmd,
            conf_location = app.asset_path("nginx.conf"),
        ));

        let static_assets = StaticfileProvider::get_static_assets(app, env)?;

        let mut plan = BuildPlan::new(&vec![setup], Some(start));
        plan.add_static_assets(static_assets);

        Ok(Some(plan))
    }
}

impl StaticfileProvider {
    pub fn get_root(app: &App, env: &Environment, staticfile_root: String) -> String {
        let mut root = String::new();
        if let Some(staticfile_root) = env.get_config_variable("STATICFILE_ROOT") {
            root = staticfile_root;
        } else if !staticfile_root.is_empty() {
            root = staticfile_root;
        } else if app.includes_directory("public") {
            root = "public".to_string();
        } else if app.includes_directory("dist") {
            root = "dist".to_string();
        } else if app.includes_directory("index") {
            root = "index".to_string();
        }

        root
    }

    fn get_static_assets(app: &App, env: &Environment) -> Result<StaticAssets> {
        let mut assets = StaticAssets::new();

        let mut mime_types = "include /nix/store/*-user-environment/conf/mime.types;".to_string();
        if app.includes_file("mime.types") {
            assets.insert("mime.types".to_string(), app.read_file("mime.types")?);
            mime_types = "include\tmime.types;".to_string();
        }

        let mut auth_basic = String::new();
        if app.includes_file("Staticfile.auth") {
            assets.insert(".htpasswd".to_string(), app.read_file("Staticfile.auth")?);
            auth_basic = format!(
                "auth_basic\t\"Password Required\";\nauth_basic_user_file\t{};",
                app.asset_path(".htpasswd")
            );
        }

        let staticfile: Staticfile = app.read_yaml("Staticfile").unwrap_or_default();
        let root = StaticfileProvider::get_root(app, env, staticfile.root.unwrap_or_default());
        let gzip = staticfile.gzip.unwrap_or_else(|| "on".to_string());
        let directory = staticfile.directory.unwrap_or_else(|| "off".to_string());
        let status_code = staticfile.status_code.unwrap_or_default();
        let mut error_page = String::new();
        for (key, value) in status_code {
            writeln!(error_page, "\terror_page {key} {value};")?;
        }

        let nginx_conf = formatdoc! {"
        daemon off;
        error_log /dev/stdout info;
        worker_processes  auto;
        events {{
            worker_connections  1024;
        }}
        
        http {{
            {mime_types}
            access_log /dev/stdout;
            default_type  application/octet-stream;
            sendfile       on;
            keepalive_timeout  60;
            types_hash_max_size 4096;
            server {{
                listen    0.0.0.0:80;
                gzip  	  {gzip};
                root	  /app/{root};
                location / {{
                    {auth_basic}
                    autoindex {directory};
                }}
        {error_page}
            }}
        }}
        ", 
        mime_types = mime_types,
        gzip = gzip,
        root = root,
        auth_basic = auth_basic,
        directory = directory,
        error_page = error_page
        };
        assets.insert("nginx.conf".to_string(), nginx_conf);

        Ok(assets)
    }
}
