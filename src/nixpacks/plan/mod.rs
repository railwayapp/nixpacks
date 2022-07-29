use self::{
    config::GeneratePlanConfig,
    legacy_phase::{LegacyBuildPhase, LegacyInstallPhase, LegacySetupPhase, LegacyStartPhase},
    phase::{Phase, StartPhase},
    topological_sort::topological_sort,
};
use super::{images::DEFAULT_BASE_IMAGE, NIX_PACKS_VERSION};
use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
};
use anyhow::Result;
use colored::Colorize;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};

pub mod config;
pub mod generator;
pub mod legacy_phase;
pub mod phase;
mod topological_sort;

const FIRST_COLUMN_WIDTH: usize = 10;
const MIN_BOX_WIDTH: usize = 20;
const MAX_BOX_WIDTH: usize = 80;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct LegacyBuildPlan {
    pub version: Option<String>,
    pub setup: Option<LegacySetupPhase>,
    pub install: Option<LegacyInstallPhase>,
    pub build: Option<LegacyBuildPhase>,
    pub start: Option<LegacyStartPhase>,
    pub variables: Option<EnvironmentVariables>,
    pub static_assets: Option<StaticAssets>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct BuildPlan {
    #[serde(rename = "nixpacksVersion")]
    nixpacks_version: Option<String>,

    #[serde(rename = "buildImage")]
    pub build_image: String,

    pub variables: Option<EnvironmentVariables>,

    #[serde(rename = "staticAssets")]
    pub static_assets: Option<StaticAssets>,

    pub phases: Vec<Phase>,

    #[serde(rename = "startPhase")]
    pub start_phase: Option<StartPhase>,
}

impl BuildPlan {
    pub fn new(phases: Vec<Phase>, start_phase: Option<StartPhase>) -> Self {
        Self {
            nixpacks_version: Some(NIX_PACKS_VERSION.to_string()),
            phases,
            start_phase,
            build_image: DEFAULT_BASE_IMAGE.to_string(),
            ..Default::default()
        }
    }

    pub fn add_phase(&mut self, phase: Phase) {
        self.phases.push(phase);
    }

    pub fn add_start_phase(&mut self, start_phase: StartPhase) {
        self.start_phase = Some(start_phase);
    }

    pub fn set_variables(&mut self, variables: EnvironmentVariables) {
        self.variables = Some(variables);
    }

    pub fn get_sorted_phases(&self) -> Result<Vec<Phase>> {
        topological_sort(self.phases.clone())
    }
}

pub trait PlanGenerator {
    fn generate_plan(&mut self, app: &App, environment: &Environment) -> Result<BuildPlan>;
}

impl BuildPlan {
    pub fn apply_config(&mut self, config: &GeneratePlanConfig) {
        let setup = self.get_phase("setup").unwrap_or_default();

        // if let Some(setup) = self.get_phase("setup").as_mut() {
        //     setup.add_nix_pkgs(config.custom_pkgs.clone());
        // }
    }

    pub fn get_phase(&self, name: &str) -> Option<&Phase> {
        self.phases.iter().find(|phase| phase.name == name)
    }

    pub fn get_build_string(&self) -> Result<String> {
        let title_str = format!(" Nixpacks v{} ", NIX_PACKS_VERSION);
        let title_width = console::measure_text_width(title_str.as_str());

        let phase_contents = self
            .get_sorted_phases()?
            .iter()
            .map(|phase| (phase.name.clone(), self.get_phase_content(phase)))
            .collect::<Vec<_>>();

        let start_contents = self
            .start_phase
            .clone()
            .unwrap_or_default()
            .cmd
            .unwrap_or_default();

        let max_right_content = phase_contents
            .iter()
            .map(|(_, content)| {
                content
                    .split('\n')
                    .map(|l| console::measure_text_width(l))
                    .collect::<Vec<_>>()
            })
            .flatten()
            .max()
            .unwrap_or(0);
        let max_right_content = std::cmp::max(
            max_right_content,
            console::measure_text_width(start_contents.as_str()),
        );

        let edge = format!("{} ", box_drawing::double::VERTICAL);
        let edge_width = console::measure_text_width(edge.as_str());

        let middle_padding = format!(" {} ", box_drawing::light::VERTICAL).to_string();
        let middle_padding_width = console::measure_text_width(middle_padding.as_str());
        let middle_padding = middle_padding.cyan().dimmed().to_string();

        let box_width = std::cmp::min(
            MAX_BOX_WIDTH,
            std::cmp::max(
                MIN_BOX_WIDTH,
                (edge_width * 2) + FIRST_COLUMN_WIDTH + middle_padding_width + max_right_content,
            ),
        );

        let second_column_width =
            box_width - (edge_width * 2) - FIRST_COLUMN_WIDTH - middle_padding_width;

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

        let phase_rows = phase_contents
            .into_iter()
            .map(|(name, content)| {
                print_row(
                    uppercase_first_letter(name).as_str(),
                    content,
                    edge.clone(),
                    middle_padding.clone(),
                    second_column_width,
                    false,
                )
            })
            .collect::<Vec<_>>()
            .join(format!("\n{hor_sep}\n").as_str());

        let start_row = print_row(
            "Start",
            start_contents,
            edge.clone(),
            middle_padding.clone(),
            second_column_width,
            false,
        );

        Ok(formatdoc! {"

          {top_box}
          {phase_rows}
          {hor_sep}
          {start_row}
          {bottom_box}
          ",
        })
    }

    fn get_phase_content(&self, phase: &Phase) -> String {
        let mut c = String::new();

        let nix_pkgs = phase.nix_pkgs.clone().unwrap_or_default();
        let apt_pkgs = phase.apt_pkgs.clone().unwrap_or_default();
        let cmds = phase.cmds.clone().unwrap_or_default();
        let pkgs = [
            nix_pkgs
                .iter()
                .map(|pkg| pkg.to_pretty_string())
                .collect::<Vec<_>>(),
            apt_pkgs,
        ]
        .concat();

        let show_label = !pkgs.is_empty() && !cmds.is_empty();

        if !pkgs.is_empty() {
            c += &format!(
                "{}{}",
                if show_label { "pkgs: " } else { "" },
                pkgs.join(", ")
            );
        }

        if c != "" && !cmds.is_empty() {
            c += "\n";
        }

        if !cmds.is_empty() {
            c += &format!(
                "{}{}",
                if show_label { "cmds: " } else { "" },
                cmds.join("\n")
            );
        }

        c
    }
}

fn print_row(
    title: &str,
    content: String,
    left_edge: String,
    middle: String,
    second_column_width: usize,
    indent_second_line: bool,
) -> String {
    let mut textwrap_opts = textwrap::Options::new(second_column_width);
    textwrap_opts.break_words = true;
    if indent_second_line {
        textwrap_opts.subsequent_indent = " ";
    }

    let right_edge = left_edge.chars().rev().collect::<String>();

    let list_lines = textwrap::wrap(content.as_str(), textwrap_opts);
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
        ),
        right_edge.cyan().dimmed()
    );

    for line in list_lines.iter().skip(1) {
        output = format!(
            "{output}\n{}{}{}{}{}",
            left_edge.cyan().dimmed(),
            console::pad_str("", FIRST_COLUMN_WIDTH, console::Alignment::Left, None),
            middle,
            console::pad_str(line, second_column_width, console::Alignment::Left, None),
            right_edge.cyan().dimmed()
        );
    }

    output
}

fn uppercase_first_letter(s: String) -> String {
    s[0..1].to_uppercase() + &s[1..]
}

impl From<LegacyBuildPlan> for BuildPlan {
    fn from(legacy_plan: LegacyBuildPlan) -> Self {
        let phases: Vec<Phase> = vec![
            legacy_plan.setup.unwrap_or_default().into(),
            legacy_plan.install.unwrap_or_default().into(),
            legacy_plan.build.unwrap_or_default().into(),
        ];

        let start: StartPhase = legacy_plan.start.unwrap_or_default().into();

        let mut plan = BuildPlan::new(phases, Some(start));
        plan.static_assets = legacy_plan.static_assets;
        plan.variables = legacy_plan.variables;

        plan
    }
}
