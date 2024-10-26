use crate::{
    config::{ENV_IF_SEND_EMAIL, ENV_VALID_FIELDS},
    email::check_send_email,
};
use anyhow::Result;
use dotenv;
use std::path::PathBuf;

#[allow(dead_code)]
pub fn load_env(env_path: &PathBuf) -> Result<()> {
    eprintln!("加载环境变量中");
    // 读取环境变量文件
    dotenv::from_path(env_path)?;
    eprintln!("加载环境变量成功");
    Ok(())
}

#[allow(dead_code)]
pub fn valid_env() -> Result<()> {
    // 验证必填字段
    for field in ENV_VALID_FIELDS {
        match dotenv::var(field) {
            Ok(value) => {
                println!("{}: {}", field, value);
                if value.trim().is_empty() {
                    return Err(anyhow::anyhow!("字段 {} 不能为空 ", field));
                }
            }
            Err(_) => {
                return Err(anyhow::anyhow!("请添加 {} 字段 ", field));
            }
        }
    }

    if check_send_email() {
        // 验证邮箱
        for field in ENV_IF_SEND_EMAIL {
            match dotenv::var(field) {
                Ok(value) => {
                    if value.trim().is_empty() {
                        return Err(anyhow::anyhow!("字段 {} 不能为空 ", field));
                    }
                }
                Err(_) => {
                    return Err(anyhow::anyhow!("请添加 {} 字段 ", field));
                }
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn env_file_exist(env_path: &PathBuf) -> bool {
    // 判断环境变量文件是否存在
    if !env_path.exists() {
        return false;
    }
    true
}

// 测试模块
#[cfg(test)]
pub mod tests {
    use super::{env_file_exist, load_env};
    use std::path::PathBuf;
    #[test]
    fn test_env_file_not_exist() {
        let env_path = dirs::home_dir().unwrap().join(".env");
        assert_eq!(env_file_exist(&env_path), false);
    }
    #[test]
    fn test_env_file_exist() {
        let env_path = PathBuf::new().join(".env");
        assert_eq!(env_file_exist(&env_path), true);
    }
    #[test]
    fn test_load_env_err() {
        let env_path = dirs::home_dir().unwrap().join(".env");
        let res = load_env(&env_path);
        assert_eq!(res.is_ok(), false);
    }
    #[test]
    fn test_load_env_ok() {
        let env_path = PathBuf::new().join(".env");
        let res = load_env(&env_path);
        assert_eq!(res.is_ok(), true);
    }
}
