mod env;
use anyhow::Result;
use chrono_tz::Asia::Shanghai;
use env::{env_file_exist, load_env, valid_env};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tokio_cron_scheduler::{Job, JobScheduler};

pub const ENV_VALID_FIELDS: [&str; 4] = ["COOKIE", "UUID", "AID", "SEND_EMAIL"];
pub const ENV_NOT_VALID_FIELDS: [&str; 5] = [
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
    data: Option<SuccessData>,
    err_no: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SuccessData {
    incr_point: u32,
    sum_point: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 定义环境变量文件路径
    let env_path = dirs::home_dir().unwrap().join(".env");

    // 判断环境变量文件是否存在
    if !env_file_exist(&env_path) {
        return Err(anyhow::anyhow!(
            "环境变量文件不存在,请在以下路径创建.env文件： {}",
            env_path.display()
        ));
    }

    load_env(&env_path)?;

    valid_env()?;

    // 初始化调度器
    let schedule = JobScheduler::new().await?;

    // 定时任务1： 每天 早上八点 执行自动签到
    let check_in_job = Job::new_async_tz("* * 8 * * * ", Shanghai, |_uuid, _lock| {
        Box::pin(async {
            eprintln!(
                "开始自动签到： {}",
                chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            if let Err(e) = auto_check_in().await {
                eprintln!("自动签到失败: {:?}, 请自行手动签到", e);
            } else {
                eprintln!(
                    "自动签到成功： {}",
                    chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
                );
            }
        })
    })?;
    // 定时任务2： 每隔一个月发送邮件提醒更换session
    // let send_email_job = Job::new_async_tz("1/10 * * * * *", Shanghai,|_uuid, _lock| {
    //     Box::pin(async move {
    //         eprintln!(
    //             "开始自动发送邮件： {}",
    //             chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
    //         );
    //     })
    // })
    // .unwrap();

    // 添加定时任务
    schedule.add(check_in_job).await?;
    // schedule.add(send_email_job).await.unwrap();

    // 启动调度器
    schedule.start().await?;

    // 保持程序运行
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}

#[allow(dead_code)]
async fn auto_check_in() -> Result<()> {
    let client = Client::new();
    // 自定义请求头
    let mut headers = header::HeaderMap::new();

    let mut cookie = dotenv::var("COOKIE")?;
    cookie.insert_str(0, "sessionid=");

    headers.insert(header::COOKIE, cookie.parse()?);
    headers.insert(header::REFERER, ROOT_URL.parse()?);

    // 自定义请求参数
    let mut params = HashMap::new();
    params.insert("uuid", dotenv::var("UUID")?);
    params.insert("aid", dotenv::var("AID")?);
    params.insert("spider", 0.to_string());
    let response = client
        .post(CHECK_IN_URL)
        .headers(headers)
        .query(&params)
        .send()
        .await?;

    // 打印响应状态码
    let status = response.status();

    let body: ResponseData = response.json().await?;
    eprintln!("请求状态: {:#?}, 响应内容: {:?}", status, body);

    Ok(())
}

#[allow(dead_code)]
fn auto_send_email() -> Result<()> {
    Ok(())
}
