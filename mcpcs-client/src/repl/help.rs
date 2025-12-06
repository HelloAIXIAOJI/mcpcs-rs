use colored::Colorize;

pub fn print_banner() {
    println!("{}", "mcpcs-client REPL".cyan().bold());
    println!("{}", "Commands:".yellow());
    println!("  {}              - Reload configuration from ~/.mcpcsrs/mcps/*.json", "/reload".green());
    println!("  {}            - List connected MCP servers", "/list mcp".green());
    println!("  {}           - List available tools from all servers", "/list tool".green());
    println!("  {} {} - Call a tool with JSON arguments (use server/tool for conflicts)", "/call".green(), "<tool> <json>".dimmed());
    println!("  {} {}    - Show detailed info about a tool", "/info tool".green(), "<name>".dimmed());
    println!("  {} {}    - Create a new empty MCP configuration file", "/newconfig".green(), "<name>".dimmed());
    println!("  {}                - Exit the REPL", "/exit".green());
    println!();
}
