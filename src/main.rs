mod config;
mod email;
mod env;

use anyhow::Result;
use chrono_tz::Asia::Shanghai;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tokio_cron_scheduler::{Job, JobScheduler};

use config::{CHECK_IN_URL, DEFAULT_CHECK_IN_CRON_STR, DEFAULT_SEND_EMAIL_CRON_STR, ROOT_URL};
use email::{auto_send_email, check_send_email};
use env::{env_file_exist, load_env, valid_env};

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
    welcome();
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
    let check_in_job = Job::new_async_tz(get_check_in_cron_str(), Shanghai, |_uuid, _lock| {
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

                let _ = auto_send_email(
                    "掘金自动签到成功提醒",
                    format!(
                        "自动签到成功: {}",
                        chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
                    ),
                )
                .await;
            }
        })
    })?;
    if check_send_email() {
        // 定时任务2： 每隔一个月发送邮件提醒更换session
        let send_email_job =
            Job::new_async_tz(get_send_email_cron_str(), Shanghai, |_uuid, _lock| {
                Box::pin(async {
                    eprintln!(
                        "开始发送邮件提醒： {}",
                        chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
                    );

                    if let Err(e) = auto_send_email(
                        "掘金签到脚本更新session提醒",
                        format!(
                            "请尽快更新cookie: {}",
                            chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
                        ),
                    )
                    .await
                    {
                        eprintln!("发送邮件提醒失败: {:?}", e);
                    } else {
                        eprintln!(
                            "发送邮件提醒成功： {}",
                            chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
                        );
                    }
                })
            })?;
        schedule.add(send_email_job).await?;
    }

    // 添加定时任务
    schedule.add(check_in_job).await?;

    // 启动调度器
    schedule.start().await?;
    eprintln!("\n定时任务已启动，如需退出按Ctrl+C退出程序\n");
    // 保持程序运行
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}

#[allow(dead_code)]
fn welcome() {
    eprintln!("欢迎使用掘金自动签到脚本 \n");
    eprintln!("关于配置字段描述：");
    // 字段描述
    let fields_description: HashMap<&str, &str> = HashMap::from([
        (
            "COOKIE",
            "掘金账号Cookie, 请自行获取, 对应掘金 sessinid（必填）",
        ),
        ("UUID", "掘金账号uuid（必填）"),
        ("AID", "掘金账号aid（必填）"),
        ("SEND_EMAIL", "是否发送邮件提醒更新cookie 0-否 1-是（可选）"),
        ("CHECK_IN_CRON_STR", "自定义签到时间 cron 表达式（可选）"),
        (
            "SEND_EMAIL_CRON_STR",
            "自定义发送邮件提醒时间 cron 表达式（可选）",
        ),
        (
            "QQ_EMAIL_ADDRESS",
            "QQ邮箱地址（可选,如果send_email为1,必填）",
        ),
        (
            "QQ_EMAIL_PASSWORD",
            "QQ邮箱密码（可选, 如果send_email为1，必填）",
        ),
    ]);
    for (field, description) in fields_description {
        eprintln!("{}: {}", field, description);
    }
    eprintln!(
        "\n cron 表达式说明(默认可以省略year):
           sec   min   hour   day of month   month   day of week   year
            *     *     *      *              *       *             *\n"
    );
}

#[allow(dead_code)]
fn get_check_in_cron_str() -> String {
    dotenv::var("CHECK_IN_CRON_STR").unwrap_or(DEFAULT_CHECK_IN_CRON_STR.to_string())
}

#[allow(dead_code)]
fn get_send_email_cron_str() -> String {
    dotenv::var("SEND_EMAIL_CRON_STR").unwrap_or(DEFAULT_SEND_EMAIL_CRON_STR.to_string())
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
