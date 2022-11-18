use std::{
    fs::File,
    io::{stdin, Read, Write},
    process::{Child, Command},
    str,
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

            config_file.ok().unwrap().write_all(save_path.as_bytes())?;
        }
    }

    let mut url = String::new();
    println!("Please enter the url of the playlist you want to download:");
    stdin().read_line(&mut url)?;

    let playlist_count_stdout = Command::new("yt-dlp")
        .arg("--flat-playlist")
        .arg("--print")
        .arg("%(playlist_count)j")
        .arg("--playlist_items")
        .arg("1")
        .arg("--yes-playlist")
        .arg("--no-cache-dir")
        .arg(&url)
        .output()
        .expect("Failed to execute command")
        .stdout;

    let playlist_count: usize = str::from_utf8(&playlist_count_stdout)
        .ok()
        .unwrap()
        .trim()
        .parse::<usize>()
        .unwrap();

    let mut thread_count: usize = num_cpus::get();
    if playlist_count < thread_count {
        thread_count = playlist_count;
    }
    let tracks_per_thread = playlist_count / thread_count;

    save_path = save_path.trim().to_string();

    let mut children = vec![];

    children.push(spawn_process(&save_path, &url, 1, 2 * tracks_per_thread));
    for i in 2..thread_count {
        let start = i * tracks_per_thread;
        children.push(spawn_process(
            &save_path,
            &url,
            start + 1,
            start + tracks_per_thread,
        ));
    }
    children.push(spawn_process(
        &save_path,
        &url,
        thread_count * tracks_per_thread + 1,
        playlist_count,
    ));

    for i in 0..children.len() {
        children[i].wait().expect("Command wasn't running");
    }

    Ok(())
}

fn spawn_process(save_path: &str, url: &str, playlist_start: usize, playlist_end: usize) -> Child {
    Command::new("yt-dlp")
        .arg("-o")
        .arg(save_path.to_owned() + "/%(playlist_title)s/%(title)s.%(ext)s")
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg("0")
        .arg("--format")
        .arg("bestaudio/best")
        .arg("-i")
        .arg("--yes-playlist")
        .arg("--no-cache-dir")
        .arg("--embed-metadata")
        .arg("--playlist-items")
        .arg(playlist_start.to_string() + "-" + &playlist_end.to_string())
        .arg(url)
        .spawn()
        .expect("Failed to execute command")
}
