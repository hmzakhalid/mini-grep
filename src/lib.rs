use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();
        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let results: Vec<String> = if fs::metadata(&config.file_path)?.is_dir() {
        let mut results = Vec::new();
        for entry in fs::read_dir(&config.file_path)? {
            let path = entry?.path();
            results.append(&mut search_in_file(path, &config)?);
        }
        results
    } else {
        search_in_file(Path::new(&config.file_path), &config)?
    };
    println!("Found {} results", results.len());
    println!("\n");
    print_results(results, &config);

    Ok(())
}

fn search_in_file(path: impl AsRef<Path>, config: &Config) -> Result<Vec<String>, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;

    let file_results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    Ok(file_results.iter().map(|s| s.to_string()).collect())
}

pub fn print_results(results: Vec<String>, config: &Config) {
    for line in results {
        let mut highlighted_line = String::new();
        let mut last_index = 0;
        for (index, _) in line.match_indices(&config.query) {
            highlighted_line.push_str(&line[last_index..index]);
            highlighted_line.push_str(&format!("\x1b[38;5;196m{}\x1b[0m", &config.query));
            last_index = index + config.query.len();
        }
        highlighted_line.push_str(&line[last_index..]);

        println!("{}", highlighted_line);
    }
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results: Vec<&str> = Vec::new();

    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }

    results
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    let mut results: Vec<&str> = Vec::new();

    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }

    results
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }
}
