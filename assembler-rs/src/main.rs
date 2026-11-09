use std::{path::PathBuf, process::exit};

use clap::Parser;

use basm::*;

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
fn main()
{
    //Gonaherra argument parsing
    let args = Args::parse();
    //Clean Input file and store inot a vector of lines and original line numbers

    let file_as_string = match std::fs::read_to_string(&args.input)
    {
        Ok(s) => s,
        Err(e) =>
        {
            eprint!("Could not read file! Error {}", e);
            exit(-1);
        }
    };
    let instruction_vector = match clean_input(&file_as_string)
    {
        Ok(r) => r,
        Err(e) =>
        {
            println!("{e}");
            std::process::exit(-1);
        }
    };
    //Assemble into a vector of the binary encodings
    let ret_vector: Vec<String> = match assemble_all(&instruction_vector)
    {
        Ok(r) => r,
        Err(e) =>
        {
            for (s, line) in e
            {
                println!("ERROR: {s} on line {line}",);
            }
            std::process::exit(-1)
        }
    };

    //Encode them
    match write_binary(&ret_vector, &args.output)
    {
        Ok(_) =>
        {
            println!("Success! File Written.");
            std::process::exit(1)
        }
        Err(e) =>
        {
            println!("Could not write output file due to: {e} ");
            std::process::exit(-1);
        }
    }
}
