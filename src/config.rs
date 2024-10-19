pub const ENV_VALID_FIELDS: [&str; 3] = ["COOKIE", "UUID", "AID"];
pub const ENV_NOT_VALID_FIELDS: [&str; 3] =
    ["SEND_EMAIL", "CHECK_IN_CRON_STR", "SEND_EMAIL_CRON_STR"];

pub const CHECK_IN_URL: &str = "https://api.juejin.cn/growth_api/v1/check_in";
pub const ROOT_URL: &str = "https://juejin.cn";

// 默认签到时间  每天早上8点
pub const DEFAULT_CHECK_IN_CRON_STR: &str = "* * 8 * * *";

// 默认发送邮件提醒时间  每两个月 一号
pub const DEFAULT_SEND_EMAIL_CRON_STR: &str = "* * * 1 1/2 *";
