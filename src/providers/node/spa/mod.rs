use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::Environment,
    nix::pkg::Pkg,
    plan::phase::Phase,
};

pub mod vite;

const NIX_ARCHIVE: &str = "ba913eda2df8eb72147259189d55932012df6301";

pub struct SpaProvider {}

impl SpaProvider {
    pub fn is_spa(app: &App) -> bool {
        // other ones will be implemented here
        vite::ViteSpaProvider::is_vite(app)
    }

    pub fn caddy_phase(app: &App, env: &Environment) -> Option<Phase> {
        if let Some(s) = env.get_config_variable("SPA_CADDY") {
            if s.to_lowercase() == "false" || s.to_lowercase() == "0" {
                return None;
            }
        }
        if Self::is_spa(app)
            && (vite::ViteSpaProvider::caddy_allowlist(app)
                || env.get_config_variable("SPA_OUT_DIR").is_some())
        {
            let mut caddy = Phase::new("caddy");
            caddy.set_nix_archive(String::from(NIX_ARCHIVE)); // caddy 2.0.4
            caddy.add_nix_pkgs(&[Pkg::new("caddy")]);
            caddy.add_cmd(format!(
                "caddy fmt --overwrite {}",
                app.asset_path("Caddyfile")
            ));
            caddy.depends_on_phase("setup");
            return Some(caddy);
        }
        None
    }

    pub fn static_assets() -> StaticAssets {
        static_asset_list! {
            "Caddyfile" => include_str!("Caddyfile")
        }
    }

    pub fn get_output_directory(app: &App) -> String {
        // other ones will be implemented here
        vite::ViteSpaProvider::get_output_directory(app)
    }

    pub fn start_command(app: &App, env: &Environment) -> Option<String> {
        if Self::caddy_phase(app, env).is_some() {
            Some(format!(
                "exec caddy run --config {} --adapter caddyfile 2>&1",
                app.asset_path("Caddyfile")
            ))
        } else {
            None
        }
    }
}
