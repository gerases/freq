use clap::{Arg, ArgMatches, Command};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufRead;
use std::io::{self};
use std::sync::mpsc;
use std::thread;
use colored::Colorize;

fn count_words(filename: &str, result: &mut HashMap<String, usize>, stop_words: &HashSet<String>) {
    let fh = File::open(filename).expect(format!("Couldn't open {}", filename).as_str());
    let reader = io::BufReader::new(fh);
    for line in reader.lines() {
        let line = line.expect("Can't read line");
        line.split_whitespace()
            .map(|word| word.to_lowercase())
            .filter(|word| !stop_words.contains(word))
            .for_each(|word| {
                *result.entry(word).or_insert(0) += 1;
            })
    }
}

fn display(result: &mut HashMap<String, usize>, top: usize) {
    let mut res_vec: Vec<(String, usize)> = result.iter().map(|(k, &v)| (k.clone(), v)).collect();
    res_vec.sort_by(|a, b| b.1.cmp(&a.1));
    for (word, count) in res_vec.iter().take(top) {
        println!("{:<10} {count}", format!("'{}'", word));
    }
}

fn parse_args() -> ArgMatches {
    let matches = Command::new("Word Frequency Analyzer")
        .version("1.0")
        .about("Prints top N frequent words given a set of files")
        .arg(
            Arg::new("top")
                .short('n')
                .long("top-num")
                .value_parser(clap::value_parser!(usize))
                .default_value("10")
                .help("Number of top words to include"),
        )
        .arg(
            Arg::new("include-stop-words")
                .short('i')
                .long("include-stop-words")
                .help("Include words like 'the', 'a', etc")
                .action(clap::ArgAction::SetTrue)
                .default_value("false"),
        )
        .arg(
            Arg::new("files")
                .help("A list of files to process")
                .required(true)
                .num_args(1..)
                .value_name("FILES"), // This is how it will be referred to in the help message
        )
        .get_matches();

    return matches;
}

fn main() {
    struct FileResult {
        filename: String,
        word_counts: HashMap<String, usize>,
    }

    let matches = parse_args();

    let mut stop_words: HashSet<String> = [
        "the", "and", "a", "to", "of", "in", "it", "is", "that", "we", "with", "on", "had", "as",
        "but", "for", "not", "from", "at", "was",
    ]
    .iter()
    .map(|stop| stop.to_string())
    .collect();

    let files: Vec<String> = matches
        .get_many::<String>("files")
        .unwrap()
        .cloned()
        .collect(); // Clone the files
    if let Some(true) = matches.get_one::<bool>("include-stop-words") {
        stop_words = HashSet::new();
    }
    let top_num: &usize = matches.get_one("top").unwrap();

    let (tx, rx) = mpsc::channel::<FileResult>();

    for file in files {
        let tx = tx.clone();
        let stop_words = stop_words.clone();
        thread::spawn(move || {
            let mut result: HashMap<String, usize> = HashMap::new();
            count_words(file.as_str(), &mut result, &stop_words);
            tx.send(FileResult {
                filename: file,
                word_counts: result,
            })
            .unwrap();
        });
    }

    drop(tx);

    for mut result in rx {
        println!("==> {}", result.filename.green());
        display(&mut result.word_counts, *top_num);
    }
}
