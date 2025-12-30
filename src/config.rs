use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub cloudflare: CloudflareConfig,
    pub dns_records: Vec<DnsRecordConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CloudflareConfig {
    #[serde(rename = "auth_type")]
    pub auth_type: String,  // 临时使用 String，稍后转换
    #[serde(rename = "auth_email")]
    pub auth_email: Option<String>,
    #[serde(rename = "auth_key")]
    pub auth_key: Option<String>,
    #[serde(rename = "api_token")]
    pub api_token: Option<String>,
    #[serde(rename = "zone_name")]
    pub zone_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DnsRecordConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub ttl: u32,
    pub proxied: bool,
    #[serde(rename = "ip_version")]
    pub ip_version: String,  // 临时使用 String，稍后转换
}

// 定义辅助函数来转换字符串到枚举
impl CloudflareConfig {
    pub fn get_auth_type(&self) -> Result<AuthType, &'static str> {
        match self.auth_type.as_str() {
            "token" => Ok(AuthType::Token),
            "emailkey" => Ok(AuthType::EmailKey),
            _ => Err("Invalid auth type"),
        }
    }
}

impl DnsRecordConfig {
    pub fn get_ip_version(&self) -> Result<IpVersion, &'static str> {
        match self.ip_version.as_str() {
            "v4" => Ok(IpVersion::V4),
            "v6" => Ok(IpVersion::V6),
            _ => Err("Invalid IP version"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthType {
    EmailKey,
    Token,
}

#[derive(Debug, Clone)]
pub enum IpVersion {
    V4,
    V6,
}