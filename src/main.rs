use std::{
    error::Error,
    io::stdin,
    process::{Child, Command, Stdio},
    thread, time,
};

fn main() {
    Command::new("yt-dlp")
        .arg("-U")
        .arg("-q")
        .spawn()
        .expect("Unable to update yt-dlp, do you have it installed?");

    let folder = rfd::FileDialog::new()
        .set_title("Select save location")
        .pick_folder()
        .unwrap();
    let save_path = folder.to_str().unwrap();

    loop {
        if let Err(error) = get_playlist(save_path) {
            println!("Error occured, try again! ({})", error);
        }
    }
}

fn get_playlist(save_path: &str) -> Result<(), Box<dyn Error>> {
    let mut url = String::new();
    println!("Please enter the url of the playlist or song you want to download:");
    stdin().read_line(&mut url).unwrap_or(0);

    println!("Getting tracks...");
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
        children.push(spawn_process(save_path, playlist_ids[index]));
        index += 1;
        println!("Downloading track {}", index);
    }

    while index < playlist_ids.len() {
        for child in &mut children {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    *child = spawn_process(save_path, playlist_ids[index]);
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

fn spawn_process(save_path: &str, id: &str) -> Child {
    Command::new("yt-dlp")
        .arg("-o")
        .arg(save_path.to_owned() + "/%(title)s.%(ext)s")
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
