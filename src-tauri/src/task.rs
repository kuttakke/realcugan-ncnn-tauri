use async_process::windows::CommandExt;
use async_process::{Command, Stdio};
use futures_lite::{io::BufReader, prelude::*};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::io::Error;
use std::path::Path;
// use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tauri::Manager;
use tokio::fs::{remove_file, write};
use tokio::sync::Semaphore;
#[cfg(target_os = "windows")]
use winapi::um::winbase::CREATE_NO_WINDOW;

#[derive(Clone, Serialize, Deserialize)]
pub struct RealcuganResult {
    pub id: String,
    pub status: bool,
    pub file: String,
    pub output: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RealcuganStatus {
    pub id: String,
    pub percent: f32,
    pub file: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RealcuganTaskArgs {
    pub file_id: String,
    pub input_file: String,
    pub input_file_data: Option<Vec<u8>>,
    pub num_magnification: u32,
    pub num_noise_level: i32,
    pub is_tta: bool,
    pub format_type: String,
}

pub async fn run_task(
    app: tauri::AppHandle,
    args: &RealcuganTaskArgs,
    real_path_str: &str,
) -> Result<(), Error> {
    let input_file_path = format!("{}temp/{}", real_path_str, args.input_file);
    match &args.input_file_data {
        Some(data) => {
            write(input_file_path.clone(), data).await?;
        }
        None => {}
    }

    let path = Path::new(&input_file_path);
    if !path.exists() {
        error!("File not found: {}", args.input_file);

        let payload = RealcuganResult {
            id: args.file_id.clone(),
            status: false,
            file: args.input_file.clone(),
            output: String::from("File not found"),
        };
        app.emit_all("run-res", &payload).unwrap();
        return Ok(());
    }

    let mut file_name = String::from(args.input_file.clone());
    if let Some(pos) = file_name.rfind('.') {
        file_name.truncate(pos);
    };
    let out_file = format!(
        "{}-{}x-{}n.{}",
        &file_name, args.num_magnification, args.num_noise_level, args.format_type
    );
    let tta = if args.is_tta { "-x" } else { "" };
    let out_file_path = format!("{}result/{}", real_path_str, &out_file);

    let result = Command::new("./resources/realcugan-ncnn-vulkan.exe")
        .args([
            "-i",
            &input_file_path,
            "-o",
            &out_file_path,
            "-s",
            format!("{}", args.num_magnification).as_str(),
            "-n",
            format!("{}", args.num_noise_level).as_str(),
            "-f",
            &args.format_type[..],
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
            let payload = RealcuganResult {
                id: args.file_id.clone(),
                status: false,
                file: args.input_file.clone(),
                output: e.to_string(),
            };
            app.emit_all("run-res", &payload).unwrap();
            return Ok(());
        }
    };

    let stdout = child.stderr.take().unwrap(); // 它的所有输出都在stderr
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    let mut output = String::new();
    while let Some(line) = lines.next().await {
        if let Ok(line) = line {
            // info!("{}", &line);
            output.push_str(&line);
            if line.ends_with("%") {
                if let Ok(number) = &line[..line.len() - 1].parse::<f32>() {
                    let playload = RealcuganStatus {
                        id: args.file_id.clone(),
                        percent: *number,
                        file: args.input_file.clone(),
                    };
                    app.emit_all("run-status", &playload).unwrap();
                }
            }
        }
    }
    info!("\n{}", output);
    let res = remove_file(&input_file_path).await;
    info!("remove {} file : {:?}", args.input_file, res);

    let payload = RealcuganResult {
        id: args.file_id.clone(),
        status: true,
        file: args.input_file.clone(),
        output: out_file,
    };
    app.emit_all("run-res", &payload).unwrap();
    return Ok(());
}

fn time_log(base: u64, info: &str) {
    // base 是由js的`new Date().getTime()`生成的
    let start_time = SystemTime::now();
    // 将 JavaScript 时间戳转换为 SystemTime
    let base_time = SystemTime::UNIX_EPOCH + Duration::from_millis(base);
    // 计算时间差
    let elapsed = start_time.duration_since(base_time).unwrap_or_else(|e| {
        error!("SystemTime went backwards: {e:?}");
        Duration::new(0, 0) // For example, default to 0 if error occurs
    });
    let elapsed_ms = elapsed.as_millis();

    info!("Time {}: {} milliseconds", info, elapsed_ms);
}

#[tauri::command(async)]
pub async fn run_all(
    app: tauri::AppHandle,
    args_list: Vec<RealcuganTaskArgs>,
    max_tasks: u32,
    base_time: u64,
) -> Result<(), RealcuganResult> {
    time_log(base_time, "start");
    // info!("run_all: {:?}", args_list);
    let real_path = app.path_resolver().resolve_resource(".");
    let real_path_str = match real_path {
        None => {
            error!("real_path: None");
            return Err(RealcuganResult {
                id: "1".to_string(),
                status: false,
                file: "1".to_string(),
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

    let semaphore = Arc::new(Semaphore::new(max_tasks as usize));
    let mut handles = vec![];
    for args in args_list {
        let app_handle = app.app_handle();
        let real_path_str_copy = real_path_str.clone();
        let semaphore = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            time_log(base_time, &format!("task {:?} run", &args.input_file));
            run_task(app_handle, &args, &real_path_str_copy)
                .await
                .unwrap();
            time_log(base_time, &format!("task {:?} over", &args.input_file));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
    time_log(base_time, "task over");
    return Ok(());
}
