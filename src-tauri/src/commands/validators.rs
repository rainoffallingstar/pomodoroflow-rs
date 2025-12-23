//! 输入验证器
//!
//! 用于验证命令输入参数的有效性

use super::CommandError;

/// 验证Todo标题
pub fn validate_todo_title(title: &str) -> Result<(), CommandError> {
    if title.trim().is_empty() {
        return Err(CommandError::Validation("标题不能为空".to_string()));
    }
    if title.len() > 200 {
        return Err(CommandError::Validation("标题过长（最多200个字符）".to_string()));
    }
    Ok(())
}

/// 验证Todo描述
pub fn validate_todo_description(description: &str) -> Result<(), CommandError> {
    if description.len() > 1000 {
        return Err(CommandError::Validation("描述过长（最多1000个字符）".to_string()));
    }
    Ok(())
}

/// 验证ID格式
pub fn validate_id(id: &str) -> Result<(), CommandError> {
    if id.trim().is_empty() {
        return Err(CommandError::Validation("ID不能为空".to_string()));
    }
    Ok(())
}

/// 验证用户名
pub fn validate_username(username: &str) -> Result<(), CommandError> {
    if username.trim().is_empty() {
        return Err(CommandError::Validation("用户名不能为空".to_string()));
    }
    if username.len() < 3 {
        return Err(CommandError::Validation("用户名至少3个字符".to_string()));
    }
    if username.len() > 50 {
        return Err(CommandError::Validation("用户名过长（最多50个字符）".to_string()));
    }
    Ok(())
}

/// 验证GitHub Token
pub fn validate_github_token(token: &str) -> Result<(), CommandError> {
    if token.trim().is_empty() {
        return Err(CommandError::Validation("GitHub Token不能为空".to_string()));
    }
    if !token.starts_with("ghp_") && !token.starts_with("github_pat_") {
        return Err(CommandError::Validation("无效的GitHub Token格式".to_string()));
    }
    Ok(())
}

/// 验证URL
pub fn validate_url(url: &str) -> Result<(), CommandError> {
    if url.trim().is_empty() {
        return Err(CommandError::Validation("URL不能为空".to_string()));
    }
    // 简单验证URL格式
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(CommandError::Validation("URL必须以http://或https://开头".to_string()));
    }
    Ok(())
}

/// 验证主题名称
pub fn validate_theme(theme: &str) -> Result<(), CommandError> {
    if theme.trim().is_empty() {
        return Err(CommandError::Validation("主题不能为空".to_string()));
    }
    if !["light", "dark", "auto"].contains(&theme) {
        return Err(CommandError::Validation("主题必须是 light、dark 或 auto".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_todo_title() {
        assert!(validate_todo_title("Valid Title").is_ok());
        assert!(validate_todo_title("").is_err());
        assert!(validate_todo_title(" ").is_err());

        let long_title = "a".repeat(201);
        assert!(validate_todo_title(&long_title).is_err());
    }

    #[test]
    fn test_validate_username() {
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("us").is_err()); // 太短
        assert!(validate_username("").is_err());

        let long_username = "u".repeat(51);
        assert!(validate_username(&long_username).is_err());
    }

    #[test]
    fn test_validate_github_token() {
        assert!(validate_github_token("ghp_xxxxxxxxxxxx").is_ok());
        assert!(validate_github_token("github_pat_xxxxxxxxxxxx").is_ok());
        assert!(validate_github_token("invalid_token").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://github.com").is_ok());
        assert!(validate_url("invalid_url").is_err());
        assert!(validate_url("").is_err());
    }

    #[test]
    fn test_validate_theme() {
        assert!(validate_theme("light").is_ok());
        assert!(validate_theme("dark").is_ok());
        assert!(validate_theme("auto").is_ok());
        assert!(validate_theme("invalid").is_err());
    }
}
