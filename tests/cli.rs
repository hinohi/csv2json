use std::env;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

static DATA_0: &'static str = r#"a,b,c
1,2,3
4,5,6
"#;
static JSON_0_H: &'static str = r#"{"a":"a","b":"b","c":"c"}"#;
static JSON_0_B: &'static str = r#"{"a":"1","b":"2","c":"3"}
{"a":"4","b":"5","c":"6"}
"#;
static ARR_0_H: &'static str = r#"["a","b","c"]"#;
static ARR_0_B: &'static str = r#"["1","2","3"]
["4","5","6"]
"#;

static DATA_1: &'static str = r#"a,b,a
A,B,C
"#;
static JSON_1_H: &'static str = r#"{"a":"a","b":"b","a":"a"}"#;
static JSON_1_B: &'static str = r#"{"a":"A","b":"B","a":"C"}
"#;
static ARR_1_H: &'static str = r#"["a","b","a"]"#;
static ARR_1_B: &'static str = r#"["A","B","C"]
"#;

static DATA_2: &'static str = "date\tevent
1995/01/17\tEarthquake
2000/01/01\tMillennium
2021/07/23\tOlympics
";
static JSON_2_H: &'static str = r#"{"date":"date","event":"event"}"#;
static JSON_2_B: &'static str = r#"{"date":"1995/01/17","event":"Earthquake"}
{"date":"2000/01/01","event":"Millennium"}
{"date":"2021/07/23","event":"Olympics"}
"#;
static ARR_2_B: &'static str = r#"["1995/01/17","Earthquake"]
["2000/01/01","Millennium"]
["2021/07/23","Olympics"]
"#;

fn exe_root() -> PathBuf {
    env::current_exe()
        .expect("No executable binary path")
        .parent()
        .expect("No executable's directory")
        .to_path_buf()
}

fn workdir(name: &str) -> PathBuf {
    let mut root = exe_root();
    if root.ends_with("deps") {
        root.pop();
    }
    root.push("test-wd");
    root.push(name);
    std::fs::create_dir_all(&root).expect("Failed to create dir");
    root
}

fn write_file(wd: &Path, name: &str, data: &str) -> PathBuf {
    let path = wd.join(name);
    std::fs::write(&path, data).expect("Failed to write data");
    path
}

fn spawn_cmd<I, S>(args: I) -> Child
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut root = exe_root();
    root.push("csv2json");
    Command::new(root)
        .args(args)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn")
}

fn write_stdin(child: &mut Child, data: &'static str) {
    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(data.as_bytes())
            .expect("Failed to write to stdout")
    });
}

fn read_stdout(child: Child) -> String {
    let output = child.wait_with_output().expect("Failed to read stdout");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// ```sh
/// echo "CSV" | csv2json
/// ```
#[test]
fn test_stdin_false_none() {
    let mut child = spawn_cmd(&[] as &[String]);
    write_stdin(&mut child, DATA_0);
    assert_eq!(read_stdout(child), JSON_0_B);
}

/// ```sh
/// echo "TSV" | csv2json -d \t -H no
/// ```
#[test]
fn test_stdin_false_no() {
    let mut child = spawn_cmd(&["-d", "\\t", "-H", "no"]);
    write_stdin(&mut child, DATA_2);
    assert_eq!(read_stdout(child), JSON_2_B);
}

/// ```sh
/// echo "CSV" | csv2json - -H ff
/// ```
#[test]
fn test_stdin_false_ff() {
    let mut child = spawn_cmd(&["-", "-H", "ff"]);
    write_stdin(&mut child, DATA_1);
    assert_eq!(read_stdout(child), format!("{}\n{}", JSON_1_H, JSON_1_B));
}

/// ```sh
/// echo "CSV" | csv2json -a -H always
/// ```
#[test]
fn test_stdin_true_always() {
    let mut child = spawn_cmd(&["-a", "-H", "always"]);
    write_stdin(&mut child, DATA_0);
    assert_eq!(read_stdout(child), format!("{}\n{}", ARR_0_H, ARR_0_B));
}

/// ```sh
/// csv2json a.csv
/// ```
#[test]
fn test_single_false_none() {
    let wd = workdir("test_single_false_none");
    let a = write_file(&wd, "a.csv", DATA_0);
    let child = spawn_cmd(&[a]);
    assert_eq!(read_stdout(child), JSON_0_B);
}

/// ```sh
/// csv2json a.tsv -H always
/// ```
#[test]
fn test_single_false_always() {
    let wd = workdir("test_single_false_always");
    let a = write_file(&wd, "a.tsv", DATA_2);
    let child = spawn_cmd(&[
        a.as_os_str(),
        &OsString::from("-H"),
        &OsString::from("always"),
    ]);
    assert_eq!(read_stdout(child), format!("{}\n{}", JSON_2_H, JSON_2_B));
}

/// ```sh
/// csv2json a.csv b.csv c.tsv -H always
/// ```
#[test]
fn test_many_false_always() {
    let wd = workdir("test_many_false_always");
    let a = write_file(&wd, "a.csv", DATA_0);
    let b = write_file(&wd, "b.csv", DATA_1);
    let c = write_file(&wd, "c.tsv", DATA_2);
    let child = spawn_cmd(&[
        &OsString::from("-H"),
        &OsString::from("always"),
        a.as_os_str(),
        b.as_os_str(),
        c.as_os_str(),
    ]);
    assert_eq!(
        read_stdout(child),
        format!(
            "{}\n{}{}\n{}{}\n{}",
            JSON_0_H, JSON_0_B, JSON_1_H, JSON_1_B, JSON_2_H, JSON_2_B
        )
    );
}

/// ```sh
/// cat a.csv | csv2json -H ff --array b.csv - c.tsv
/// ```
#[test]
fn test_many_true_ff() {
    let wd = workdir("test_many_true_ff");
    let b = write_file(&wd, "b.csv", DATA_1);
    let c = write_file(&wd, "c.tsv", DATA_2);
    let mut child = spawn_cmd(&[
        &OsString::from("-H"),
        &OsString::from("ff"),
        &OsString::from("--array"),
        b.as_os_str(),
        &OsString::from("-"),
        c.as_os_str(),
    ]);
    write_stdin(&mut child, DATA_0);
    assert_eq!(
        read_stdout(child),
        format!("{}\n{}{}{}", ARR_1_H, ARR_1_B, ARR_0_B, ARR_2_B)
    );
}
