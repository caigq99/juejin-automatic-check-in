use anyhow::Result;
use lettre::{message::MessageBuilder, SmtpTransport, Transport};

#[allow(dead_code)]
pub async fn auto_send_email() -> Result<()> {
    //  STMP 服务配置
    let smtp_server = "smtp.qq.com";
    let stmp_port = 465;
    let qq_email_address = dotenv::var("QQ_EMAIL_ADDRESS")?;
    let qq_email_password = dotenv::var("QQ_EMAIL_PASSWORD")?;

    // 邮件内容
    let email = MessageBuilder::new()
        .from(qq_email_address.parse()?)
        .to(qq_email_address.parse()?)
        .subject("掘金自动签到提醒")
        .body(format!(
            "请尽快更新cookie: {}",
            chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S")
        ))?;

    let transport = SmtpTransport::relay(smtp_server)?
        .credentials((qq_email_address, qq_email_password).into())
        .port(stmp_port)
        .build();

    transport.send(&email)?;

    Ok(())
}

// 检查是否需要发送邮件
#[allow(dead_code)]
pub fn check_send_email() -> bool {
    dotenv::var("SEND_EMAIL").unwrap_or("0".to_string()) == "1".to_string()
}
