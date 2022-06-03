use super::Provider;
use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::Environment,
    nix::pkg::Pkg,
    phase::{BuildPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Staticfile {
    pub root: Option<String>,
    pub directory: Option<String>,
    pub gzip: Option<String>,
    pub status_code: Option<HashMap<u32, String>>,
}

pub struct StaticfileProvider {}

impl Provider for StaticfileProvider {
    fn name(&self) -> &str {
        "staticfile"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Staticfile"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        let pkg = Pkg::new("nginx");
        Ok(Some(SetupPhase::new(vec![pkg])))
    }

    fn static_assets(&self, app: &App, _env: &Environment) -> Result<Option<StaticAssets>> {
        let mut assets = StaticAssets::new();

        let mut mime_types = "".to_string();
        if app.includes_file("mime.types") {
            assets.insert("mime.types".to_string(), app.read_file("mime.types")?);
            mime_types = "include\tmime.types;".to_string();
        }

        let mut auth_basic = "".to_string();
        if app.includes_file("Staticfile.auth") {
            assets.insert(".htpasswd".to_string(), app.read_file("Staticfile.auth")?);
            auth_basic = format!(
                "auth_basic\t\"Password Required\";\nauth_basic_user_file\t{};",
                app.asset_path(".htpasswd")
            );
        }

        let staticfile: Staticfile = app.read_yaml("Staticfile").unwrap_or_default();
        let root = staticfile.root.unwrap_or_else(|| "/app".to_string());
        let gzip = staticfile.gzip.unwrap_or_else(|| "on".to_string());
        let directory = staticfile.directory.unwrap_or_else(|| "off".to_string());
        let status_code = staticfile.status_code.unwrap_or_default();
        let mut error_page = "".to_string();
        for (key, value) in status_code {
            error_page += &format!("\terror_page {} {};\n", key, value);
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
                root	  {root};
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
        Ok(Some(assets))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(Some(BuildPhase::new(
            "mkdir /etc/nginx/ /var/log/nginx/ /var/cache/nginx/".to_string(),
        )))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        // shell command to edit 0.0.0.0:80 to $PORT
        let shell_cmd = "[[ -z \"${PORT}\" ]] && echo \"Environment variable PORT not found. Using PORT 80\" || sed -i \"s/0.0.0.0:80/$PORT/g\"";
        Ok(Some(StartPhase::new(format!(
            "{shell_cmd} {conf_location} && nginx -c {conf_location}",
            shell_cmd = shell_cmd,
            conf_location = app.asset_path("nginx.conf"),
        ))))
    }
}
