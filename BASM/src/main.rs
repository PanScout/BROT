use std::process::ExitCode;
use std::{path::PathBuf, process::exit};

use clap::Parser;

use basm::clean_input;
use basm::assemble_all;

fn write_binary(instructions: &[String], output_file: &PathBuf) -> Result<(), String>
{
    std::fs::write(output_file, instructions.join("\n")).map_err(|e| format!("{e}"))
}

#[derive(Parser, Debug)]
struct Args
{
    #[arg(required = true)]
    input: PathBuf,
    #[arg(short)]
    output: PathBuf,
}

fn run(input: &PathBuf, output: &PathBuf) -> Result<(), String>
{
    let file_as_string = std::fs::read_to_string(input).map_err(|e| e.to_string())?;
    let instruction_vector = clean_input(&file_as_string)?;
    let ret_vector: Vec<String> = match assemble_all(&instruction_vector)
    {
        Ok(r) => r,
        Err(e) => 
        {
            for (error, line) in e 
            {
                eprintln!("Error one line {line:^5}: {error}")
            }

            return Err("Could not assemble".to_string());
        }
    };
    write_binary(&ret_vector,output)?;
    Ok(())
}
fn main()
{
    //Gonaherra argument parsing
    let args = Args::parse();
    //Clean Input file and store inot a vector of lines and original line numbers
    match run(&args.input, &args.output) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {eprintln!("ERROR: {e}"); ExitCode::FAILURE},
    };

}
