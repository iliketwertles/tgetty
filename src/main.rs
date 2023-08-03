use std::fs;
use std::io::{stdout, Write};
extern crate pam;
extern crate ini;
use ini::Ini;
use std::process::Command;
use users::get_user_by_name;
use std::os::unix::process::CommandExt;

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
    let mut exec_split = exec.split_whitespace();
    let name = section.get("Name").unwrap();

    // PAM shit
    let mut auth = pam::Authenticator::with_password("tdm").unwrap();
    auth.get_handler().set_credentials(&username, password);
    if auth.authenticate().is_ok() && auth.open_session().is_ok() {
        println!("Successfully opened a session!");
    }
    else {
        println!("Authentication failed =/");
    }

    // launching 
    let user = get_user_by_name(&username).unwrap();
    println!("Now launching {}", name); 
    let _error = Command::new(exec_split.next().unwrap())
        .args(exec_split.next())
        .uid(user.uid())
        .gid(user.primary_group_id())
        .exec();

}