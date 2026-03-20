// ─── ANSI colour constants ────────────────────────────────────────────────────
const RESET:  &str = "\x1b[0m";
const BOLD:   &str = "\x1b[1m";
const CYAN:   &str = "\x1b[36m";
const GREEN:  &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA:&str = "\x1b[35m";

/// Prints a styled startup banner to stdout.
///
/// Called once inside the Poise `setup` callback after Discord confirms login,
/// so `bot_name` and `guild_count` are guaranteed to be real values.
pub fn print_banner(bot_name: &str, guild_count: usize) {
    println!();
    println!("{CYAN}{BOLD}╔══════════════════════════════════════╗{RESET}");
    println!("{CYAN}{BOLD}║        AniList Discord Bot           ║{RESET}");
    println!("{CYAN}{BOLD}╚══════════════════════════════════════╝{RESET}");
    println!();
    println!("  {GREEN}{BOLD}✔  Logged in{RESET}       {YELLOW}{bot_name}{RESET}");
    println!("  {GREEN}{BOLD}✔  Guilds{RESET}          {MAGENTA}{guild_count}{RESET}");
    println!("  {GREEN}{BOLD}✔  Presence{RESET}        rotating (10 statuses, 30s interval)");
    println!("  {GREEN}{BOLD}✔  Commands{RESET}        /anime  /manga  /profile");
    println!();
    println!("{CYAN}──────────────────────────────────────────{RESET}");
    println!();
}