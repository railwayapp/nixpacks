use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use crate::Pkg;
use anyhow::Result;
use colored::Colorize;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};

use super::NIX_PACKS_VERSION;

pub mod generator;

const FIRST_COLUMN_WIDTH: usize = 10;
const MIN_BOX_WIDTH: usize = 20;
const MAX_BOX_WIDTH: usize = 80;

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
        let title_str = format!(" Nixpacks v{} ", NIX_PACKS_VERSION);
        let title_width = console::measure_text_width(title_str.as_str());

        let setup_phase = self.setup.clone().unwrap_or_default();
        let install_phase = self.install.clone().unwrap_or_default();
        let build_phase = self.build.clone().unwrap_or_default();
        let start_phase = self.start.clone().unwrap_or_default();

        let pkg_list = [
            setup_phase
                .pkgs
                .iter()
                .map(Pkg::to_pretty_string)
                .collect::<Vec<_>>(),
            setup_phase.apt_pkgs.unwrap_or_default(),
        ]
        .concat()
        .join(", ");

        let install_cmds = install_phase.clone().cmds.unwrap_or_default();
        let build_cmds = build_phase.clone().cmds.unwrap_or_default();
        let start_cmd = start_phase.clone().cmd.unwrap_or_default();

        let max_right_content = [
            vec![pkg_list.clone()],
            install_cmds,
            build_cmds,
            vec![start_cmd],
        ]
        .concat()
        .iter()
        .map(String::len)
        .max()
        .unwrap_or(0);

        let edge = format!("{} ", box_drawing::double::VERTICAL);
        let edge_width = console::measure_text_width(edge.as_str());

        let middle_padding = format!(" {} ", box_drawing::light::VERTICAL)
            .cyan()
            .dimmed()
            .to_string();
        let middle_padding_width = console::measure_text_width(middle_padding.as_str());

        let box_width = std::cmp::min(
            MAX_BOX_WIDTH,
            std::cmp::max(
                MIN_BOX_WIDTH,
                (edge_width * 2) + FIRST_COLUMN_WIDTH + middle_padding_width + max_right_content,
            ),
        );

        let second_column_width =
            box_width - (edge_width * 2) - FIRST_COLUMN_WIDTH - middle_padding_width;

        let packages_row = print_row(
            "Packages",
            &pkg_list,
            &edge,
            &middle_padding,
            second_column_width,
            true,
        );
        let install_row = print_row(
            "Install",
            &install_phase.cmds.unwrap_or_default().join("\n"),
            &edge,
            &middle_padding,
            second_column_width,
            false,
        );
        let build_row = print_row(
            "Build",
            &build_phase.cmds.unwrap_or_default().join("\n"),
            &edge,
            &middle_padding,
            second_column_width,
            false,
        );
        let start_row = print_row(
            "Start",
            &start_phase.cmd.unwrap_or_default(),
            &edge,
            &middle_padding,
            second_column_width,
            false,
        );

        let title_side_padding = ((box_width as f64) - (title_width as f64) - 2.0) / 2.0;

        let top_box = format!(
            "{}{}{}{}{}",
            box_drawing::double::DOWN_RIGHT.cyan().dimmed(),
            str::repeat(
                box_drawing::double::HORIZONTAL,
                title_side_padding.ceil() as usize
            )
            .cyan()
            .dimmed(),
            title_str.magenta().bold(),
            str::repeat(
                box_drawing::double::HORIZONTAL,
                title_side_padding.floor() as usize
            )
            .cyan()
            .dimmed(),
            box_drawing::double::DOWN_LEFT.cyan().dimmed(),
        );

        let bottom_box = format!(
            "{}{}{}",
            box_drawing::double::UP_RIGHT.cyan().dimmed(),
            str::repeat(box_drawing::double::HORIZONTAL, box_width - 2)
                .cyan()
                .dimmed(),
            box_drawing::double::UP_LEFT.cyan().dimmed()
        );

        let hor_sep = format!(
            "{}{}{}",
            box_drawing::double::VERTICAL.cyan().dimmed(),
            str::repeat(box_drawing::light::HORIZONTAL, box_width - 2)
                .cyan()
                .dimmed(),
            box_drawing::double::VERTICAL.cyan().dimmed()
        );

        formatdoc! {"

          {}
          {}
          {}
          {}
          {}
          {}
          {}
          {}
          {}
          ",
          top_box,
          packages_row,
          hor_sep,
          install_row,
          hor_sep,
          build_row,
          hor_sep,
          start_row,
          bottom_box
        }
    }
}

fn print_row(
    title: &str,
    content: &str,
    left_edge: &str,
    middle: &str,
    second_column_width: usize,
    indent_second_line: bool,
) -> String {
    let mut textwrap_opts = textwrap::Options::new(second_column_width);
    textwrap_opts.break_words = true;
    if indent_second_line {
        textwrap_opts.subsequent_indent = " ";
    }

    let right_edge = left_edge.chars().rev().collect::<String>();

    let list_lines = textwrap::wrap(content, textwrap_opts);
    let mut output = format!(
        "{}{}{}{}{}",
        left_edge.cyan().dimmed(),
        console::pad_str(title, FIRST_COLUMN_WIDTH, console::Alignment::Left, None),
        middle,
        console::pad_str(
            &list_lines[0],
            second_column_width,
            console::Alignment::Left,
            None
        )
        .white(),
        right_edge.cyan().dimmed()
    );

    for line in list_lines.iter().skip(1) {
        output = format!(
            "{}\n{}{}{}{}{}",
            output,
            left_edge.cyan().dimmed(),
            console::pad_str("", FIRST_COLUMN_WIDTH, console::Alignment::Left, None),
            middle,
            console::pad_str(line, second_column_width, console::Alignment::Left, None).white(),
            right_edge.cyan().dimmed()
        );
    }

    output
}
