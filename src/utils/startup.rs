// ─── ANSI colour constants ────────────────────────────────────────────────────
const RESET:   &str = "\x1b[0m";
const BOLD:    &str = "\x1b[1m";
const CYAN:    &str = "\x1b[36m";
const GREEN:   &str = "\x1b[32m";
const YELLOW:  &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";

/// Prints a styled startup banner to stdout.
///
/// `commands` is built from the live `FrameworkOptions` command list so the
/// output always reflects exactly what was registered — no manual update needed
/// when commands are added or removed.
pub fn print_banner(bot_name: &str, guild_count: usize, commands: &[String]) {
    let cmd_lines = format_commands(commands);

    println!();
    println!("{CYAN}{BOLD}╔══════════════════════════════════════╗{RESET}");
    println!("{CYAN}{BOLD}║        AniList Discord Bot           ║{RESET}");
    println!("{CYAN}{BOLD}╚══════════════════════════════════════╝{RESET}");
    println!();
    println!("  {GREEN}{BOLD}✔  Logged in{RESET}       {YELLOW}{bot_name}{RESET}");
    println!("  {GREEN}{BOLD}✔  Guilds{RESET}          {MAGENTA}{guild_count}{RESET}");
    println!("  {GREEN}{BOLD}✔  Presence{RESET}        rotating (10 statuses, 30s interval)");
    println!("  {GREEN}{BOLD}✔  Commands{RESET}        ({} registered)", commands.len());
    for line in &cmd_lines {
        println!("             {MAGENTA}{line}{RESET}");
    }
    println!();
    println!("{CYAN}──────────────────────────────────────────{RESET}");
    println!();
}

/// Groups command names into wrapped lines capped at 42 characters.
fn format_commands(commands: &[String]) -> Vec<String> {
    const MAX: usize = 42;
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for cmd in commands {
        let token = format!("/{cmd}");
        if !current.is_empty() && current.len() + 2 + token.len() > MAX {
            lines.push(current.clone());
            current.clear();
        }
        if !current.is_empty() {
            current.push_str("  ");
        }
        current.push_str(&token);
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}