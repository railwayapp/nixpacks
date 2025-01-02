use crate::nixpacks::{app::App, nix::pkg::Pkg, plan::phase::Phase};

pub mod vite;

pub struct SpaProvider {}

impl SpaProvider {
    pub fn is_spa(app: &App) -> bool {
        // other ones will be implemented here
        vite::ViteSpaProvider::is_vite(app)
    }

    pub fn caddy_phase(app: &App) -> Phase {
        let mut caddy = Phase::new("caddy");
        caddy.set_nix_archive(String::from("ba913eda2df8eb72147259189d55932012df6301")); // caddy 2.0.4
        caddy.add_nix_pkgs(&[Pkg::new("caddy")]);
        caddy.add_cmd(format!(
            "caddy fmt --overwrite {}",
            app.asset_path("Caddyfile")
        ));
        caddy
    }

    pub fn get_output_directory(app: &App) -> String {
        // other ones will be implemented here
        vite::ViteSpaProvider::get_output_directory(app)
    }
}
