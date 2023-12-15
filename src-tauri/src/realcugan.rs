use async_process::windows::CommandExt;
use async_process::{Command, Stdio};
use futures_lite::{io::BufReader, prelude::*};
use log::{error, info};
use std::fs;
use std::path::Path;
use tauri::Manager;
#[cfg(target_os = "windows")]
use winapi::um::winbase::CREATE_NO_WINDOW;

#[derive(Clone, serde::Serialize)]
pub struct RealcuganResult {
    pub id: String,
    pub status: bool,
    pub file: String,
    pub output: String,
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct RealcuganStatus {
    pub id: String,
    pub percent: f32,
    pub file: String,
}

#[tauri::command]
pub async fn run_realcugan<'a>(
    app: tauri::AppHandle,
    file_id: &'a str,
    input_file: &'a str,
    num_magnification: u32,
    num_noise_level: i32,
    is_tta: bool,
    format_type: &'a str,
) -> Result<RealcuganResult, RealcuganResult> {
    let real_path = app.path_resolver().resolve_resource(".");
    let real_path_str = match real_path {
        None => {
            error!("real_path: None");
            return Err(RealcuganResult {
                id: file_id.to_string(),
                status: false,
                file: input_file.to_string(),
                output: String::from("real_path: None"),
            });
        }
        Some(path) => {
            let path_str = path
                .to_str()
                .unwrap()
                .replace("\\\\?\\", "")
                .replace("\\", "/");
            info!("real_path: {}", path_str);
            path_str
        }
    };
    let input_file_path = format!("{}temp/{}", real_path_str.as_str(), input_file);
    info!("input_file_path: {}", &input_file_path);
    let path = Path::new(&input_file_path);
    if !path.exists() {
        error!("File not found: {}", input_file);
        return Err(RealcuganResult {
            id: file_id.to_string(),
            status: false,
            file: input_file.to_string(),
            output: String::from("File not found"),
        });
    }

    let mut file_name = String::from(input_file);
    if let Some(pos) = file_name.rfind('.') {
        file_name.truncate(pos);
    };
    let out_file = format!(
        "{}-{}x-{}n.{}",
        &file_name, num_magnification, num_noise_level, format_type
    );
    let tta = if is_tta { "-x" } else { "" };
    let out_file_path = format!("{}result/{}", real_path_str.as_str(), &out_file);

    let result = Command::new("./resources/realcugan-ncnn-vulkan.exe")
        .args([
            "-i",
            &input_file_path,
            "-o",
            &out_file_path,
            "-s",
            format!("{}", num_magnification).as_str(),
            "-n",
            format!("{}", num_noise_level).as_str(),
            "-f",
            format_type,
            tta,
        ])
        .stdout(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .stderr(Stdio::piped())
        .spawn();
    // #[cfg(target_os = "windows")]
    // command.creation_flags(CREATE_NO_WINDOW);

    let mut child = match result {
        Ok(child) => child,
        Err(e) => {
            error!("failed to execute child: {}", e);
            return Err(RealcuganResult {
                id: file_id.to_string(),
                status: false,
                file: input_file.to_string(),
                output: e.to_string(),
            });
        }
    };

    let stdout = child.stderr.take().unwrap(); // 它的所有输出都在stderr
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Some(line) = lines.next().await {
        if let Ok(line) = line {
            info!("{}", &line);
            if line.ends_with("%") {
                if let Ok(number) = &line[..line.len() - 1].parse::<f32>() {
                    let playload = RealcuganStatus {
                        id: file_id.to_string(),
                        percent: *number,
                        file: input_file.to_string(),
                    };
                    app.emit_all("run-status", &playload).unwrap();
                }
            }
        }
    }

    let res = fs::remove_file(&input_file_path);
    info!("remove file : {:?}", res);

    return Ok(RealcuganResult {
        id: file_id.to_string(),
        status: true,
        file: input_file.to_string(),
        output: out_file,
    });
}
