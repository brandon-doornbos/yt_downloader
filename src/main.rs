use std::{
    error::Error,
    fs::File,
    io::{stdin, Read, Write},
    process::{Child, Command, Stdio},
    thread, time,
};

const CONFIG_PATH: &str = "config.txt";

fn main() -> std::io::Result<()> {
    let mut config_file = File::open(CONFIG_PATH);
    let mut save_path = String::new();

    match config_file {
        Ok(mut file) => {
            file.read_to_string(&mut save_path)?;

            if save_path.is_empty() {
                println!("Please enter the folder where you want your playlists to be saved:");
                stdin().read_line(&mut save_path)?;

                file.write_all(save_path.as_bytes())?;
            }
        }
        Err(_) => {
            config_file = File::create(CONFIG_PATH);

            println!("Please enter the folder where you want your playlists to be saved:");
            stdin().read_line(&mut save_path)?;

            config_file?.write_all(save_path.trim().as_bytes())?;
        }
    }

    loop {
        if let Err(error) = get_playlist(&save_path) {
            println!("Error occured, try again! ({})", error);
        }
    }
}

fn get_playlist(save_path: &str) -> Result<(), Box<dyn Error>> {
    let mut url = String::new();
    println!("Please enter the url of the playlist or song you want to download:");
    stdin().read_line(&mut url).unwrap_or(0);

    println!("Getting tracks...");

    let playlist_title_stdout = String::from_utf8(
        Command::new("yt-dlp")
            .arg("--flat-playlist")
            .arg("--print")
            .arg("%(playlist_title)s")
            .arg("--yes-playlist")
            .arg("--playlist-items")
            .arg("1")
            .arg("--no-cache-dir")
            .arg("--")
            .arg(&url)
            .output()?
            .stdout,
    )?;
    let mut playlist_title = playlist_title_stdout.trim();

    if playlist_title != "NA" {
        println!("Found playlist: {}", playlist_title);
    } else {
        playlist_title = "";
    }

    let playlist_ids_stdout = String::from_utf8(
        Command::new("yt-dlp")
            .arg("--flat-playlist")
            .arg("--print")
            .arg("%(id)s")
            .arg("--yes-playlist")
            .arg("--no-cache-dir")
            .arg("--")
            .arg(&url)
            .output()?
            .stdout,
    )?;

    let playlist_ids: Vec<&str> = playlist_ids_stdout.split_whitespace().collect();
    println!("Found {} tracks", playlist_ids.len());

    let thread_count: usize = num_cpus::get();

    let mut children = vec![];
    let mut index = 0;

    for _i in 0..std::cmp::min(thread_count, playlist_ids.len()) {
        children.push(spawn_process(
            save_path,
            playlist_ids[index],
            playlist_title,
        ));
        index += 1;
        println!("Downloading track {}", index);
    }

    while index < playlist_ids.len() {
        for child in &mut children {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    *child = spawn_process(save_path, playlist_ids[index], playlist_title);
                    index += 1;
                    println!("Downloading track {}", index);
                }
                Ok(None) => {}
                Err(error) => println!("Error attempting to wait: {}", error),
            }
        }
        thread::sleep(time::Duration::from_millis(100));
    }

    for mut child in children {
        child.wait().expect("Command was not running!");
    }

    println!("Done!");
    Ok(())
}

fn spawn_process(save_path: &str, id: &str, playlist_title: &str) -> Child {
    let mut path = save_path.to_owned();
    if !playlist_title.is_empty() {
        path += "/";
        path += playlist_title;
    }
    path += "/%(title)s.%(ext)s";

    Command::new("yt-dlp")
        .arg("-o")
        .arg(path)
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg("0")
        .arg("--format")
        .arg("bestaudio/best")
        .arg("-i")
        .arg("--no-cache-dir")
        .arg("--embed-metadata")
        .arg("--")
        .arg(id)
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to execute command")
}
