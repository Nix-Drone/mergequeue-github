use rand;

fn move_random_word(filename: &str) -> std::io::Result<()> {
    // Read the file into a vector of lines
    let file = std::fs::File::open(&filename)?;
    let reader = BufReader::new(file);
    let mut lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    // Choose a random line
    let mut rng = rand::thread_rng();
    let line_index = rng.gen_range(0..lines.len());

    // Choose a random word from the line
    let words: Vec<&str> = lines[line_index].split_whitespace().collect();
    if words.is_empty() {
        return Ok(());
    }
    let word_index = rng.gen_range(0..words.len());
    let word = words[word_index].to_string();

    // Remove the word from the line
    lines[line_index] = lines[line_index].replace(&word, "");

    // Choose another random line
    let other_line_index = rng.gen_range(0..lines.len());

    // Add the word to the other line
    lines[other_line_index] = format!("{} {}", lines[other_line_index], word);

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
