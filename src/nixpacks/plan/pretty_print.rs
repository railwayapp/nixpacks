use super::{phase::Phase, BuildPlan};
use crate::nixpacks::{nix::pkg::Pkg, NIX_PACKS_VERSION};
use anyhow::Result;
use colored::Colorize;
use indoc::formatdoc;
use std::fmt::Write;

const FIRST_COLUMN_MIN_WIDTH: usize = 10;
const MIN_BOX_WIDTH: usize = 20;
const MAX_BOX_WIDTH: usize = 80;

impl BuildPlan {
    pub fn get_build_string(&self) -> Result<String> {
        let title_str = format!(" Nixpacks v{} ", NIX_PACKS_VERSION);
        let title_width = console::measure_text_width(title_str.as_str());

        let phase_contents = self
            .get_sorted_phases()?
            .iter()
            .map(|phase| (phase.name.clone(), self.get_phase_content(phase).unwrap()))
            .collect::<Vec<_>>();

        let start_contents = self
            .start_phase
            .clone()
            .unwrap_or_default()
            .cmd
            .unwrap_or_default();

        let max_right_content = phase_contents
            .iter()
            .flat_map(|(_, content)| {
                content
                    .split('\n')
                    .map(console::measure_text_width)
                    .collect::<Vec<_>>()
            })
            .max()
            .unwrap_or(0);
        let max_right_content = std::cmp::max(
            max_right_content,
            console::measure_text_width(start_contents.as_str()),
        );

        let first_column_width = std::cmp::max(
            FIRST_COLUMN_MIN_WIDTH,
            phase_contents
                .iter()
                .map(|(name, _)| console::measure_text_width(name))
                .max()
                .unwrap_or(0),
        );

        let edge = format!("{} ", box_drawing::double::VERTICAL);
        let edge_width = console::measure_text_width(edge.as_str());

        let middle_padding = format!(" {} ", box_drawing::light::VERTICAL);
        let middle_padding_width = console::measure_text_width(middle_padding.as_str());
        let middle_padding = middle_padding.cyan().dimmed().to_string();

        let box_width = std::cmp::min(
            MAX_BOX_WIDTH,
            std::cmp::max(
                MIN_BOX_WIDTH,
                (edge_width * 2) + first_column_width + middle_padding_width + max_right_content,
            ),
        );

        let second_column_width =
            box_width - (edge_width * 2) - first_column_width - middle_padding_width;

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
                    uppercase_first_letter(name.as_str()).as_str(),
                    content.as_str(),
                    edge.as_str(),
                    middle_padding.as_str(),
                    first_column_width,
                    second_column_width,
                    false,
                )
            })
            .collect::<Vec<_>>()
            .join(format!("\n{}\n", hor_sep).as_str());

        let start_row = print_row(
            "Start",
            start_contents.as_str(),
            edge.as_str(),
            middle_padding.as_str(),
            first_column_width,
            second_column_width,
            false,
        );

        Ok(formatdoc! {"

          {}
          {}
          {}
          {}
          {}
          ",
          top_box,
          phase_rows,
          hor_sep,
          start_row,
          bottom_box
        })
    }

    fn get_phase_content(&self, phase: &Phase) -> Result<String> {
        let mut c = String::new();

        let nix_pkgs = phase.nix_pkgs.clone().unwrap_or_default();
        let apt_pkgs = phase.apt_pkgs.clone().unwrap_or_default();
        let cmds = phase.cmds.clone().unwrap_or_default();
        let pkgs = [
            nix_pkgs
                .iter()
                .map(Pkg::to_pretty_string)
                .collect::<Vec<_>>(),
            apt_pkgs,
        ]
        .concat();

        let show_label = !pkgs.is_empty() && !cmds.is_empty();

        if !pkgs.is_empty() {
            write!(
                c,
                "{}{}",
                if show_label { "pkgs: " } else { "" },
                pkgs.join(", "),
            )?;
        }

        if !c.is_empty() && !cmds.is_empty() {
            c += "\n";
        }

        if !cmds.is_empty() {
            write!(
                c,
                "{}{}",
                if show_label { "cmds: " } else { "" },
                cmds.join("\n")
            )?;
        }

        Ok(c)
    }
}

fn print_row(
    title: &str,
    content: &str,
    left_edge: &str,
    middle: &str,
    first_column_width: usize,
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
        console::pad_str(title, first_column_width, console::Alignment::Left, None),
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
            "{}\n{}{}{}{}{}",
            output,
            left_edge.cyan().dimmed(),
            console::pad_str("", first_column_width, console::Alignment::Left, None),
            middle,
            console::pad_str(line, second_column_width, console::Alignment::Left, None),
            right_edge.cyan().dimmed()
        );
    }

    output
}

fn uppercase_first_letter(s: &str) -> String {
    s[0..1].to_uppercase() + &s[1..]
}
