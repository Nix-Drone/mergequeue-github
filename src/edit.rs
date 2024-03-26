use rand::Rng;
use std::fs::OpenOptions;
use std::io::{BufRead, Write};
use std::io::{BufReader, BufWriter};
use walkdir::WalkDir;

pub fn change_file(filenames: &[String]) -> std::io::Result<()> {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..filenames.len());
    move_random_line(&filenames[idx])
}

pub fn move_random_line(filename: &str) -> std::io::Result<()> {
    // Read the file into a vector of lines
    let file = std::fs::File::open(&filename)?;
    let reader = BufReader::new(file);
    let mut lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    // Choose a random line
    let mut rng = rand::thread_rng();
    let line_index = rng.gen_range(0..lines.len());

    // Remove the line from the vector
    let line = lines.remove(line_index);

    // Choose another random line
    let other_line_index = rng.gen_range(0..lines.len());

    // Insert the line at the new position
    lines.insert(other_line_index, line);

    // Write the lines back to the file
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&filename)?;
    let mut writer = BufWriter::new(file);
    for line in lines {
        writeln!(writer, "{}", line)?;
    }

    Ok(())
}
