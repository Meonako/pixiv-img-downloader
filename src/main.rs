use std::{fs::create_dir, fs::read_to_string, fs::File, io::Write, time::Instant};

use colored::*;
use reqwest::header;

const OUTPUT_FOLDER: &str = "outputs";
const IMG_CDN_PREFIX: &str = "https://i.pximg.net/img-original/img/";
const REFERER: (&str, &str) = ("referer", "https://www.pixiv.net/");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    control::set_virtual_terminal(true).unwrap();

    let client = build_client();

    create_output_folder();

    println!(
        "Prefix URL Path with \"{}\" will read from a file (e.g. {})",
        "file:".cyan(),
        "file:url.txt".bright_blue()
    );

    let mut path = Vec::new();

    loop_get_input(&mut path);

    if path.len() <= 0 {
        return Err(
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No URL to download"))
                as Box<dyn std::error::Error>,
        );
    }

    println!("Downloading {} file(s)", path.len().to_string().green());

    let mut tasks = Vec::with_capacity(path.len());

    for img_url in path {
        tasks.push(tokio::spawn(download_image(client.clone(), img_url)));
    }

    let now = Instant::now();

    for handle in tasks {
        handle.await.unwrap();
    }

    println!("Finished in: {} second(s)", now.elapsed().as_secs_f32());

    Ok(())
}

fn loop_get_input(path: &mut Vec<String>) {
    loop {
        print!(
            "Image URL Path (\"{}\"/\"{}\" to start download): ",
            "continue".green(),
            "download".green()
        );
        std::io::stdout().flush().unwrap();
        let mut user_input = String::new();
        std::io::stdin()
            .read_line(&mut user_input)
            .expect("able to read input");

        match user_input.to_lowercase().trim() {
            "continue" | "download" => break,
            x if x.is_empty() || path.contains(&x.to_owned()) => continue,
            x if x.starts_with("file:") => match read_to_string(x.replace("file:", "")) {
                Ok(res) => {
                    let mut list: Vec<String> = res.split("\r\n").map(|x| x.to_owned()).collect();
                    let unique: Vec<String> = list
                        .drain(..)
                        .collect::<std::collections::HashSet<_>>()
                        .drain()
                        .collect();
                    println!("Added: {} ", unique.len().to_string().green());
                    path.extend(unique);
                },
                Err(_) => println!("{}", "File not found!".red()),
            },
            x => path.push(x.to_owned()),
        }
    }
}

async fn download_image(client: reqwest::Client, url: String) {
    let filename = url.split("/").last().expect("filename to be present in URL");
    let output_path = format!("{OUTPUT_FOLDER}/{filename}");

    let url = if !url.contains(IMG_CDN_PREFIX) {
        format!("{IMG_CDN_PREFIX}{url}")
    } else {
        url
    };

    let res = client.get(url).send().await.expect("Cannot get url");
    println!(
        "{}: {}",
        match res.status().as_str() {
            s if s == "200" => "200 OK".green(),
            s if s == "403" => "403 FORBIDDEN".red(),
            s if s == "404" => "404 NOT FOUND".red(),
            s => s.white(),
        },
        output_path,
    );

    let body = res.bytes().await.expect("Cannot convert to bytes");

    let mut file = File::create(output_path).expect("to create file");
    file.write_all(&body).expect("to write file");
}

fn build_client() -> reqwest::Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(REFERER.0, header::HeaderValue::from_static(REFERER.1));

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("able to build reqwest client")
}

fn create_output_folder() {
    match create_dir(OUTPUT_FOLDER) {
        Ok(_) => println!("Folder \"{}\" Created", OUTPUT_FOLDER.cyan()),
        Err(err) => match err.kind() {
            std::io::ErrorKind::AlreadyExists => {
                println!("Folder \"{}\" Already Exists.", OUTPUT_FOLDER.cyan())
            }
            _ => panic!("{}", err),
        },
    }
}