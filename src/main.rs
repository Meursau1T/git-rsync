use std::process::Command;
use std::fs;
use structopt::StructOpt;
use itertools::Itertools;


#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "l", long = "local", default_value = "")]
    local_path: String,

    #[structopt(short = "r", long = "remote", default_value = "")]
    remote_path: String,

    #[structopt(short = "u", long = "user", default_value = "")]
    user_ip: String,

    #[structopt(short = "p", long = "rsync_param", default_value = "")]
    rsync_param: String,
}

fn main() {
    let res = get_git_edit();
    let async_res  = async_by_log(res.clone());
    println!("File list: {:?}", &async_res);
    for file in async_res {
        call_rsync(file);
    }
}

/**
 * 获取Git的编辑记录
 */
fn get_git_edit() -> Vec<String> {
    let raw_gst = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .expect("Gst err");
    let gst = String::from_utf8(raw_gst.stdout).expect("From utf8 err");
    let gst_in_line: Vec<String> = gst
        .split("\n")
        .map(|line| line.split(" ").last().expect("Get last err"))
        .filter(|line| line.len() > 0)
        .map(|line| line.to_string())
        .collect();
    gst_in_line
}

/**
 * 根据历史编辑记录/tmp/rsync-git，找到上次编辑过的文件，和本次编辑的文件取合集，作为同步目标
 * 记录本次编辑的文件列表，以备下次使用
 * 此逻辑用于解决编辑的文件被恢复后下次编辑时不同步到云端的问题
 */
fn async_by_log(curr_edit: Vec<String>) -> Vec<String> {
    let log_path = "/tmp/rsync-git";
    if let Err(_) = fs::metadata(&log_path) {
        let _ = fs::OpenOptions::new().write(true)
                                      .create_new(true)
                                      .open(&log_path);
    }
    let log_file = fs::read_to_string(&log_path).expect("Read to string err");
    let last_edit: Vec<_> = log_file.split("\n").filter(|line| line.len() > 0).map(|line| line.to_string()).collect();
    fs::write(&log_path, &curr_edit.join("\n")).expect("Write new log err");
    let mut both_edit = vec![];
    both_edit.extend(last_edit);
    both_edit.extend(curr_edit);
    both_edit.into_iter().unique().collect()
}

/**
 * 获取文件对应的目录
 */
fn get_dir(path: String) -> String {
    let res = &path[..path.rfind('/').expect("Rfind err")];
    res.to_string()
}

/**
 * 执行Rsync命令同步本地文件到云端
 */
fn call_rsync(path: String) {
    let opt = Cli::from_args();
    let Cli { local_path, remote_path, user_ip, rsync_param } = &opt;
    let orig_path = format!("{}/{}", local_path, &path);
    let tgt_path = get_dir(format!("{}:{}/{}", &user_ip, &remote_path, &path));
    let cmd = format!("-av{}", rsync_param);
    let raw_rsync = Command::new("rsync")
        .arg(&cmd)
        .arg(&orig_path)
        .arg(&tgt_path)
        .output()
        .expect(format!("Rsync err {}", path).as_str());
    if rsync_param == "n" {
        let rsync = String::from_utf8(raw_rsync.stdout).expect("From utf8 err");
        let rsync_err = String::from_utf8(raw_rsync.stderr).expect("From utf8 err");
        println!("Rsync input:\n{}", format!("rsync {} {} {}", &cmd, &orig_path, &tgt_path));
        println!("Rsync result:\n{}", rsync);
        println!("\x1b[93mError:\x1b[0m\n{}", rsync_err);
    }
}
