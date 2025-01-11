use crate::nixpacks::{app::App, environment::Environment, nix::pkg::Pkg, plan::phase::Phase};

pub mod vite;

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
            caddy.set_nix_archive(String::from("ba913eda2df8eb72147259189d55932012df6301")); // caddy 2.0.4
            caddy.add_nix_pkgs(&[Pkg::new("caddy")]);
            caddy.add_cmd(format!(
                "caddy fmt --overwrite {}",
                app.asset_path("Caddyfile")
            ));
            return Some(caddy);
        }
        None
    }

    pub fn get_output_directory(app: &App) -> String {
        // other ones will be implemented here
        vite::ViteSpaProvider::get_output_directory(app)
    }
}
