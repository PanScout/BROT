use clap::Parser;
use std::path::PathBuf;


use BVM::*;


#[derive(Parser, Debug)]
struct Args
{
    #[arg(required = true, short)]
    input: PathBuf,
    //#[arg(short)]
    //output: PathBuf,
}


fn path_into_vector(input: &String) -> Result<Vec<String>, String>
{
    const HEX_LENGTH: usize = 12;
    let mut ret: Vec<String> = Vec::new();
    let string_bin =
        std::fs::read_to_string(input).unwrap_or_else(|input| panic!("Could not open input file: {input}!"));
    for line_string in string_bin.lines()
    {
        assert!(
            string_bin[0..=1].to_string() != r"\\",
            "Comment comes before instructions. Did you edit the binary file?"
        );

        if string_bin.starts_with(r"\\")
        {
            return Err("Comment comes before instructions. Did you edit the binary file?".to_string());
        }

        if string_bin.starts_with(r"//")
        {
            return Err("No foward Slashes!".to_string());
        }

        let trun_string =
            line_string.strip_suffix(r"\\").unwrap_or(line_string).trim().to_ascii_uppercase().to_string();
        assert!(trun_string.len() == HEX_LENGTH, "Final hex string is of improper length");
        ret.push(trun_string);
    }
    Ok(ret)
}

fn main()
{
    let args = Args::parse();

    let Ok(string_vec) = path_into_vector(&args.input.to_str().expect("No file provided!").to_string())
    else
    {
        panic!("Could not file into vector!");
    };

    let mut vm: BvmState = BvmState::default();
    match execute_code(&mut vm, &string_vec, true)
    {
        Ok(_) => println!("PROGRAM FINISHED!"),
        Err(e) => print!("Could not finish program due to {e}"),
    }
}
