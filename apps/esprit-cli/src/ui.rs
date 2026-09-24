use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::time::Instant;

pub fn logo() -> String {
    format!(
        r#"
  {}
  {}
  {}
  {}
  {}
"#,
        "███████╗███████╗██████╗ ██████╗ ██╗████████╗".cyan().bold(),
        "██╔════╝██╔════╝██╔══██╗██╔══██╗██║╚══██╔══╝".cyan(),
        "█████╗  ███████╗██████╔╝██████╔╝██║   ██║   "
            .magenta()
            .bold(),
        "██╔══╝  ╚════██║██╔═══╝ ██╔══██╗██║   ██║   ".magenta(),
        "███████╗███████║██║     ██║  ██║██║   ██║   ".cyan().bold(),
    )
}

pub fn banner() {
    println!("{}", logo());
    println!(
        "  {} {} {} {}",
        format!(" v{} ", env!("CARGO_PKG_VERSION"))
            .black()
            .on_cyan()
            .bold(),
        " AIR-GAPPED ".black().on_green().bold(),
        " APPLE SILICON METAL ".black().on_magenta().bold(),
        " LOCAL-FIRST ".black().on_yellow().bold(),
    );
    println!("  {}\n", "─".repeat(58).dimmed());
}

pub fn card(title: &str, content: &[String]) {
    let width: usize = 64;
    let title_fmt = format!(" {} ", title);
    let rem = width.saturating_sub(title_fmt.len() + 4);
    println!(
        "  ╭─{}─{}╮",
        title_fmt.cyan().bold(),
        "─".repeat(rem).dimmed()
    );
    for line in content {
        let clean_len = strip_ansi_escapes(line).len();
        let pad = width.saturating_sub(clean_len + 4);
        println!("  │  {} {}│", line, " ".repeat(pad));
    }
    println!("  ╰{}╯", "─".repeat(width.saturating_sub(2)).dimmed());
}

pub fn panel_header(title: &str, tag: Option<&str>) {
    let tag_str = tag.map(|t| format!(" [{t}]")).unwrap_or_default();
    println!(
        "\n  {} {}{}",
        "◆".cyan().bold(),
        title.bold(),
        tag_str.dimmed()
    );
    println!("  {}", "─".repeat(56).dimmed());
}

pub fn section(title: &str) {
    println!("\n  {} {}", "▸".magenta().bold(), title.bold());
}

pub fn divider() {
    println!("  {}", "─".repeat(56).dimmed());
}

pub fn kv(key: &str, val: &str) {
    let key_pad = format!("{:<22}", key);
    println!("  {} {}", key_pad.dimmed(), val.bold());
}

pub fn kv_dot(key: &str, val: &str) {
    let total: usize = 32;
    let dots = ".".repeat(total.saturating_sub(key.len() + 2));
    println!("  {} {} {}", key.dimmed(), dots.dimmed(), val.bold());
}

pub fn ok(msg: &str) {
    println!("  {} {}", "✓".green().bold(), msg);
}

pub fn fail(msg: &str) {
    println!("  {} {}", "✗".red().bold(), msg);
}

pub fn warn(msg: &str) {
    println!("  {} {}", "⚠".yellow().bold(), msg);
}

pub fn info(msg: &str) {
    println!("  {} {}", "ℹ".cyan().bold(), msg);
}

pub fn step(idx: usize, total: usize, msg: &str) {
    println!(
        "  {} {}",
        format!("[{}/{}]", idx, total).magenta().bold(),
        msg
    );
}

pub fn update_badge(current: &str, latest: &str, summary: Option<&str>) {
    println!();
    println!("  ╭────────────────────────────────────────────────────────╮");
    println!(
        "  │  {} {} → {}  │",
        "⚡ UPDATE AVAILABLE:".yellow().bold(),
        current.dimmed(),
        latest.cyan().bold()
    );
    if let Some(s) = summary {
        let s_trunc = if s.len() > 46 { &s[..46] } else { s };
        println!("  │  {} {:<48}│", "•".dimmed(), s_trunc.dimmed());
    }
    println!(
        "  │  {} {}              │",
        "Run:".dimmed(),
        "esprit update".bold().green()
    );
    println!("  ╰────────────────────────────────────────────────────────╯\n");
}

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(70));
    pb
}

#[allow(dead_code)]
pub fn bar(len: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "  {msg:.bold}  [{bar:38.cyan/black}]  {pos}/{len}  {elapsed}",
        )
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    pb.set_message(msg.to_string());
    pb
}

pub fn elapsed(start: Instant) -> String {
    let ms = start.elapsed().as_millis();
    if ms < 1000 {
        format!("{ms}ms")
    } else {
        format!("{:.1}s", start.elapsed().as_secs_f64())
    }
}

pub fn meter_bar(used: u64, total: u64, width: usize) -> String {
    if total == 0 {
        return "─".repeat(width);
    }
    let ratio = (used as f64 / total as f64).clamp(0.0, 1.0);
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let filled_str = "█".repeat(filled);
    let empty_str = "░".repeat(empty);

    if ratio > 0.85 {
        format!("{}{}", filled_str.red().bold(), empty_str.dimmed())
    } else if ratio > 0.65 {
        format!("{}{}", filled_str.yellow().bold(), empty_str.dimmed())
    } else {
        format!("{}{}", filled_str.cyan().bold(), empty_str.dimmed())
    }
}

pub fn alert_box(title: &str, lines: &[String]) {
    let width: usize = 64;
    let title_fmt = format!(" 🚨 {} ", title);
    let rem = width.saturating_sub(title_fmt.len() + 4);
    println!(
        "\n  ╭─{}─{}╮",
        title_fmt.red().bold(),
        "─".repeat(rem).red()
    );
    for line in lines {
        let clean_len = strip_ansi_escapes(line).len();
        let pad = width.saturating_sub(clean_len + 4);
        println!("  │  {} {}│", line, " ".repeat(pad));
    }
    println!("  ╰{}╯\n", "─".repeat(width.saturating_sub(2)).red());
}

fn strip_ansi_escapes(s: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1B' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}
