use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use colored::Colorize;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};

use super::NIX_PACKS_VERSION;

pub mod generator;

pub const BOX_WIDTH: usize = 80;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildPlan {
    pub version: Option<String>,
    pub setup: Option<SetupPhase>,
    pub install: Option<InstallPhase>,
    pub build: Option<BuildPhase>,
    pub start: Option<StartPhase>,
    pub variables: Option<EnvironmentVariables>,
    pub static_assets: Option<StaticAssets>,
}

pub trait PlanGenerator {
    fn generate_plan(&mut self, app: &App, environment: &Environment) -> Result<BuildPlan>;
}

impl BuildPlan {
    pub fn get_build_string(&self) -> String {
        let title_str = format!("Nixpacks v{}", NIX_PACKS_VERSION);
        let title_width = console::measure_text_width(title_str.as_str()) + 2;

        let top_box = format!(
            "{}{} {} {}{}",
            box_drawing::double::DOWN_RIGHT.cyan().dimmed(),
            str::repeat(
                box_drawing::double::HORIZONTAL,
                (BOX_WIDTH - title_width) / 2
            )
            .cyan()
            .dimmed(),
            title_str.magenta().bold(),
            str::repeat(
                box_drawing::double::HORIZONTAL,
                (BOX_WIDTH - title_width) / 2
            )
            .cyan()
            .dimmed(),
            box_drawing::double::DOWN_LEFT.cyan().dimmed(),
        );

        let bottom_box = format!(
            "{}{}{}",
            box_drawing::double::UP_RIGHT.cyan().dimmed(),
            str::repeat(box_drawing::double::HORIZONTAL, BOX_WIDTH - 1)
                .cyan()
                .dimmed(),
            box_drawing::double::UP_LEFT.cyan().dimmed()
        );

        let hor_sep = format!(
            "{}{}{}",
            box_drawing::double::VERTICAL.cyan().dimmed(),
            str::repeat(box_drawing::light::HORIZONTAL, BOX_WIDTH - 1)
                .cyan()
                .dimmed(),
            box_drawing::double::VERTICAL.cyan().dimmed()
        );

        let setup_phase = self.setup.clone().unwrap_or_default();
        let install_phase = self.install.clone().unwrap_or_default();
        let build_phase = self.build.clone().unwrap_or_default();
        let start_phase = self.start.clone().unwrap_or_default();

        let pkg_list = [
            setup_phase
                .pkgs
                .iter()
                .map(|pkg| pkg.to_pretty_string())
                .collect::<Vec<_>>(),
            setup_phase.apt_pkgs.unwrap_or_default(),
        ]
        .concat()
        .join(", ");

        let packages_row = print_row("Packages", pkg_list);
        let install_row = print_row("Install", install_phase.cmds.unwrap_or_default().join("\n"));
        let build_row = print_row("Build", build_phase.cmds.unwrap_or_default().join("\n"));
        let start_row = print_row("Start", start_phase.cmd.unwrap_or_default());

        return formatdoc! {"

          {top_box}
          {packages_row}
          {hor_sep}
          {install_row}
          {hor_sep}
          {build_row}
          {hor_sep}
          {start_row}
          {bottom_box}
          ",
        };
    }
}

fn print_row(title: &str, content: String) -> String {
    let first_column_width = 10;

    let middle_padding = format!(" {} ", box_drawing::light::VERTICAL)
        .cyan()
        .dimmed()
        .to_string();
    let middle_padding_width = console::measure_text_width(middle_padding.as_str());
    let second_column_width = BOX_WIDTH - first_column_width - middle_padding_width - 2;

    let mut textwrap_opts = textwrap::Options::new(second_column_width);
    textwrap_opts.break_words = true;
    textwrap_opts.subsequent_indent = "  ";

    let list_lines = textwrap::wrap(content.as_str(), textwrap_opts);
    let mut output = format!(
        "{} {}{}{} {}",
        box_drawing::double::VERTICAL.cyan().dimmed(),
        console::pad_str(
            title,
            first_column_width - 1,
            console::Alignment::Left,
            None
        ),
        middle_padding,
        console::pad_str(
            &list_lines[0],
            second_column_width,
            console::Alignment::Left,
            None
        )
        .white(),
        box_drawing::double::VERTICAL.cyan().dimmed()
    );

    for line in list_lines.iter().skip(1) {
        output = format!(
            "{output}\n{}{}{}{}{}",
            box_drawing::double::VERTICAL.cyan().dimmed(),
            console::pad_str("", first_column_width, console::Alignment::Left, None),
            middle_padding,
            console::pad_str(
                line,
                second_column_width + 1,
                console::Alignment::Left,
                None
            )
            .white(),
            box_drawing::double::VERTICAL.cyan().dimmed()
        );
    }

    output
}
