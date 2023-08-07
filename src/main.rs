use std::process::Stdio;
use std::os::unix::process::CommandExt;
use std::fs;
use std::env;
use std::io::{stdout, Write};
extern crate pam;
extern crate ini;
use ini::Ini;
use std::process::Command;
fn main() {
    // get  desktops
    let desktops = fs::read_dir("/usr/share/wayland-sessions").unwrap();

    // define variables for later use, also ^
    let mut username = String::new();
    let mut choice = String::new();
    let mut _count = 0;
    let mut desktops_path = Vec::new();

    for desktop in desktops {
        desktops_path.push(desktop.unwrap().path());
    }
    let env_path = env!("PATH");
    let env_xdg_dir = env!("XDG_RUNTIME_DIR");
    println!("Availible desktops: ");
    for (i, desk) in desktops_path.iter().enumerate() {
        println!("{desk} [{i}]", desk = desk.display().to_string().replace("/usr/share/wayland-sessions/", "").replace(".desktop", ""));
    }

    // adding newline for looks
    println!();
    // save username
    print!("Username: ");
    stdout().flush().unwrap();
    let _b1 = std::io::stdin().read_line(&mut username).unwrap();

    // save password, but securly
    let password = rpassword::prompt_password("Password: ").unwrap();

    // save desktop choice
    print!("Desktop: ");
    stdout().flush().unwrap();
    let _b2 = std::io::stdin().read_line(&mut choice).unwrap();
    stdout().flush().unwrap();

    // parse selected .desktop file
    let index: usize = choice.trim().parse::<usize>().unwrap();
    let path: String = desktops_path[index].display().to_string();
    println!("you chose: {}", path);
    let data = Ini::load_from_file_noescape(path).unwrap();
    let section = data.section(Some("Desktop Entry")).unwrap();
    let exec = section.get("Exec").unwrap();
    let exec_split = exec.split_whitespace();
    let name = section.get("Name").unwrap();

    // PAM shit, attempting to log user in
    let mut auth = pam::Authenticator::with_password("tdm").unwrap();
    auth.get_handler().set_credentials(username.trim(), password);
    if auth.authenticate().is_ok() && auth.open_session().is_ok() {
        println!("Successfully opened a session!");
    }
    else {
        println!("Authentication failed =/");
    }
    auth.close_on_drop = false;


    // launching selected DE/WM
    // setting path for some reason
    println!("Now launching {}", name);
    let exec_args: Vec<_> = exec_split.collect();
    let _error = Command::new(exec_args[0])
        .args(&exec_args[1..])
        .env("PATH", env_path)
        .env("XDG_RUNTIME_DIR", env_xdg_dir)
        .env("XDG_DATA_DIRS", "/usr/share")
        .env("XDG_CONFIG_DIRS", "/etc/xdg")
        .uid(1000)
        .gid(1000)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .exec();

}
