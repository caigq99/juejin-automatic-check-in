use chrono::TimeZone;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, process};
use tokio::time::{sleep, Duration};
use tokio_cron_scheduler::{Job, JobBuilder, JobScheduler};

const ENV_VALID_FIELDS: [&str; 4] = ["COOKIE", "UUID", "AID", "SEND_EMAIL"];
const ENV_NOT_VALID_FIELDS: [&str; 5] = [
    "SMTP_USER",
    "SMTP_PASS",
    "SMTP_SERVER",
    "SMTP_PORT",
    "EMAIL_RECIPIENT",
];

const CHECK_IN_URL: &str = "https://api.juejin.cn/growth_api/v1/check_in";
const ROOT_URL: &str = "https://juejin.cn";

#[derive(Debug, Serialize, Deserialize)]
struct ResponseData {
    err_msg: String,
    data: Option<()>,
    err_no: u32,
}

#[tokio::main]
async fn main() {
    // 定义环境变量文件路径
    // let env_path = dirs::home_dir().unwrap().join(".env");
    let env_path = PathBuf::new().join(".env");
    // 判断环境变量文件是否存在
    if !env_file_exist(&env_path) {
        return;
    }

    let _ = load_env(&env_path);

    // 初始化调度器
    let schedule = JobScheduler::new().await.unwrap();

    // 定时任务1： 每天 21：45 执行
    let check_in_job = Job::new_async("0 56 21 * * * *", |_uuid, _lock| {
        Box::pin(async {
            eprintln!(
                "开始自动签到： {}",
                chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
            );

            // chrono::TimeZone::from_offset(chrono::Utc::offset_from_local_datetime(&self, local));
            if let Err(e) = auto_check_in().await {
                eprintln!("自动签到失败: {:?}, 请自行手动签到", e);
            } else {
                eprintln!(
                    "自动签到成功： {}",
                    chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
                );
            }
        })
    })
    .unwrap();
    // 定时任务2： 每隔一个月发送邮件提醒更换session
    // let send_email_job = Job::new_async("1/10 * * * * *", |_uuid, _lock| {
    //     Box::pin(async move {
    //         eprintln!(
    //             "开始自动发送邮件： {}",
    //             chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
    //         );
    //     })
    // })
    // .unwrap();

    // 添加定时任务
    schedule.add(check_in_job).await.unwrap();
    // schedule.add(send_email_job).await.unwrap();

    // 启动调度器
    schedule.start().await.unwrap();

    // 保持程序运行
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
#[allow(dead_code)]
fn env_file_exist(env_path: &PathBuf) -> bool {
    let mut path = env_path.clone();
    // 判断环境变量文件是否存在
    if !path.exists() {
        // 提示用户创建环境变量文件， 将需要的配置信息字段打印出来
        path.pop();
        println!("请在以下路径创建.env文件： {}", path.display());
        println!("以下字段为必须： ");
        for field in ENV_VALID_FIELDS {
            println!("{}", field);
        }
        println!("以下字段非必须： （如果SEND_EMAIL为1, 则以下字段为必须）");
        for field in ENV_NOT_VALID_FIELDS {
            println!("{}", field);
        }
        return false;
    }
    true
}

#[allow(dead_code)]
fn load_env(env_path: &PathBuf) {
    // 读取环境变量文件
    let _ = dotenv::from_path(env_path);
    eprintln!("加载环境变量中");
    for field in ENV_VALID_FIELDS {
        match dotenv::var(field) {
            Ok(value) => {
                if value.trim().is_empty() {
                    eprintln!("字段 {} 不能为空 ", field);
                    process::exit(1);
                } else {
                    if field == "SEND_EMAIL" && value == "1" {
                        for field in ENV_NOT_VALID_FIELDS {
                            match dotenv::var(field) {
                                Ok(value) => {
                                    if value.trim().is_empty() {
                                        eprintln!("字段 {} 不能为空 ", field);
                                        process::exit(1);
                                    }
                                }
                                Err(_) => {
                                    eprintln!("请添加 {} 字段 ", field);
                                    process::exit(1);
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {
                eprintln!("请添加 {} 字段 ", field);
                process::exit(1);
            }
        }
    }
    eprintln!("加载环境变量成功");
}

#[allow(dead_code)]
async fn auto_check_in() -> reqwest::Result<()> {
    let client = Client::new();
    // 自定义请求头
    let mut headers = header::HeaderMap::new();

    let mut cookie = dotenv::var("COOKIE").unwrap();
    cookie.insert_str(0, "sessionid=");

    headers.insert(header::COOKIE, cookie.parse().unwrap());
    headers.insert(header::REFERER, ROOT_URL.parse().unwrap());

    // 自定义请求参数
    let mut params = HashMap::new();
    params.insert("uuid", dotenv::var("UUID").unwrap());
    params.insert("aid", dotenv::var("AID").unwrap());
    let response = client
        .post(CHECK_IN_URL)
        .headers(headers)
        .query(&params)
        .send()
        .await?;

    // 打印响应状态码
    let status = response.status();
    // 打印响应内容
    let body: ResponseData = response.json().await?;
    eprintln!("请求状态: {:#?}, 响应内容: {:?}", status, body);
    Ok(())
}

#[allow(dead_code)]
fn auto_send_email() -> reqwest::Result<()> {
    Ok(())
}
