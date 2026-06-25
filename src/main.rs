mod cli;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: pita decode <input.tex|romfs_dir> [output.dds|output_dir]");
        eprintln!("       pita decode --help");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "decode" => {
            let input = &args[2];
            let output = args.get(3).map(|s| s.as_str());
            let ok = cli::decode(input, output);
            if !ok { std::process::exit(1); }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Usage: pita decode <input.tex|romfs_dir> [output.dds|output_dir]");
            std::process::exit(1);
        }
    }
}
