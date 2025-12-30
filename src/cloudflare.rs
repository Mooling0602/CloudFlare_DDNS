use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Zone {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    pub content: String,
    pub r#type: String,
    pub ttl: u32,
    pub proxied: bool,
}

#[derive(Debug, Serialize)]
pub struct UpdateDnsRecordRequest {
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    pub proxied: bool,
}

/// 更新 DNS 记录时使用的参数结构体
#[derive(Debug, Clone)]
pub struct UpdateDnsRecordParams<'a> {
    pub zone_id: &'a str,
    pub record_id: &'a str,
    pub record_type: &'a str,
    pub name: &'a str,
    pub content: &'a str,
    pub ttl: u32,
    pub proxied: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub errors: Vec<ApiError>,
    pub messages: Vec<String>,
    pub result: T,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiError {
    pub code: u32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ListZonesResponse {
    pub success: bool,
    pub errors: Vec<ApiError>,
    pub messages: Vec<String>,
    pub result: Vec<Zone>,
    pub result_info: ResultInfo,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ListDnsRecordsResponse {
    pub success: bool,
    pub errors: Vec<ApiError>,
    pub messages: Vec<String>,
    pub result: Vec<DnsRecord>,
    pub result_info: ResultInfo,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ResultInfo {
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub count: u32,
    pub total_count: u32,
}

pub struct CloudflareClient {
    client: reqwest::Client,
    auth_email: String,
    auth_key: String,
}

impl CloudflareClient {
    pub fn new(auth_email: String, auth_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            auth_email,
            auth_key,
        }
    }

    /// 使用 Bearer Token 的 CloudflareClient
    pub fn new_with_token(token: String) -> Self {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                        .expect("Invalid token"),
                );
                headers
            })
            .build()
            .expect("Failed to build client");

        Self {
            client,
            auth_email: String::new(),
            auth_key: token,
        }
    }

    /// 获取 Zone ID
    pub async fn get_zone_id(&self, zone_name: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://api.cloudflare.com/client/v4/zones?name={}", zone_name);
        
        let response = if !self.auth_email.is_empty() {
            // 使用 Email + API Key 认证
            self.client
                .get(&url)
                .header("X-Auth-Email", &self.auth_email)
                .header("X-Auth-Key", &self.auth_key)
                .header("Content-Type", "application/json")
                .send()
                .await?
        } else {
            // 使用 API Token 认证
            self.client.get(&url).send().await?
        };

        let status = response.status();
        let _response_text = response.text().await?;
        
        // 检查响应状态码
        if !status.is_success() {
            return Err(format!("API 请求失败，状态码 {}。请检查您的 API 凭据。", status).into());
        }
        
        let zones_response: Result<ListZonesResponse, _> = serde_json::from_str(&_response_text);
        match zones_response {
            Ok(zones_response) => {
                if zones_response.success && !zones_response.result.is_empty() {
                    Ok(zones_response.result[0].id.clone())
                } else {
                    Err("无法获取区域 ID".to_string().into())
                }
            }
            Err(_) => {
                // 解析失败，可能是认证错误或无效的响应格式
                Err("API 认证失败或凭据无效。请检查您的 API 凭据。".to_string().into())
            }
        }
    }

    /// 获取 DNS 记录 ID
    pub async fn get_dns_record_id(&self, zone_id: &str, record_name: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}",
            zone_id, record_name
        );

        let response = if !self.auth_email.is_empty() {
            // 使用 Email + API Key 认证
            self.client
                .get(&url)
                .header("X-Auth-Email", &self.auth_email)
                .header("X-Auth-Key", &self.auth_key)
                .header("Content-Type", "application/json")
                .send()
                .await?
        } else {
            // 使用 API Token 认证
            self.client.get(&url).send().await?
        };

        let status = response.status();
        let response_text = response.text().await?;
        
        // 检查响应状态码
        if !status.is_success() {
            return Err(format!("API 请求失败，状态码 {}: {}。请检查您的 API 凭据。", status, response_text).into());
        }
        
        let dns_response: Result<ListDnsRecordsResponse, _> = serde_json::from_str(&response_text);
        match dns_response {
            Ok(dns_response) => {
                if dns_response.success && !dns_response.result.is_empty() {
                    Ok(dns_response.result[0].id.clone())
                } else {
                    Err(format!("无法获取 DNS 记录 ID: {:?}", dns_response.errors).into())
                }
            }
            Err(_) => {
                // 解析失败，可能是认证错误或无效的响应格式
                Err(format!("无法解析 API 响应。请检查您的 API 凭据。\n响应: {}", response_text).into())
            }
        }
    }

    /// 获取 DNS 记录详情
    pub async fn get_dns_record(&self, zone_id: &str, record_id: &str) -> Result<DnsRecord, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id
        );

        let response = if !self.auth_email.is_empty() {
            // 使用 Email + API Key 认证
            self.client
                .get(&url)
                .header("X-Auth-Email", &self.auth_email)
                .header("X-Auth-Key", &self.auth_key)
                .header("Content-Type", "application/json")
                .send()
                .await?
        } else {
            // 使用 API Token 认证
            self.client.get(&url).send().await?
        };

        let status = response.status();
        let response_text = response.text().await?;
        
        // 检查响应状态码
        if !status.is_success() {
            return Err(format!("API 请求失败，状态码 {}: {}。请检查您的 API 凭据。", status, response_text).into());
        }
        
        let response_data: Result<ApiResponse<DnsRecord>, _> = serde_json::from_str(&response_text);
        match response_data {
            Ok(response_data) => {
                if response_data.success {
                    Ok(response_data.result)
                } else {
                    Err(format!("无法获取 DNS 记录: {:?}", response_data.errors).into())
                }
            }
            Err(_) => {
                // 解析失败，可能是认证错误或无效的响应格式
                Err(format!("无法解析 API 响应。请检查您的 API 凭据。\n响应: {}", response_text).into())
            }
        }
    }

    /// 更新 DNS 记录
    pub async fn update_dns_record(
        &self,
        params: UpdateDnsRecordParams<'_>,
    ) -> Result<DnsRecord, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            params.zone_id, params.record_id
        );

        let update_request = UpdateDnsRecordRequest {
            record_type: params.record_type.to_string(),
            name: params.name.to_string(),
            content: params.content.to_string(),
            ttl: params.ttl,
            priority: None,
            proxied: params.proxied,
        };

        let response = if !self.auth_email.is_empty() {
            // 使用 Email + API Key 认证
            self.client
                .put(&url)
                .header("X-Auth-Email", &self.auth_email)
                .header("X-Auth-Key", &self.auth_key)
                .header("Content-Type", "application/json")
                .json(&update_request)
                .send()
                .await?
        } else {
            // 使用 API Token 认证
            self.client.put(&url).json(&update_request).send().await?
        };

        let status = response.status();
        let response_text = response.text().await?;
        
        // 检查响应状态码
        if !status.is_success() {
            return Err(format!("API 请求失败，状态码 {}: {}。请检查您的 API 凭据。", status, response_text).into());
        }
        
        let response_data: Result<ApiResponse<DnsRecord>, _> = serde_json::from_str(&response_text);
        match response_data {
            Ok(response_data) => {
                if response_data.success {
                    Ok(response_data.result)
                } else {
                    Err(format!("无法更新 DNS 记录: {:?}", response_data.errors).into())
                }
            }
            Err(_) => {
                // 解析失败，可能是认证错误或无效的响应格式
                Err(format!("无法解析 API 响应。请检查您的 API 凭据。\n响应: {}", response_text).into())
            }
        }
    }

    /// 创建新的 DNS 记录
    pub async fn create_dns_record(
        &self,
        zone_id: &str,
        record_type: &str,
        name: &str,
        content: &str,
        ttl: u32,
        proxied: bool,
    ) -> Result<DnsRecord, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );

        let create_request = UpdateDnsRecordRequest {
            record_type: record_type.to_string(),
            name: name.to_string(),
            content: content.to_string(),
            ttl,
            priority: None,
            proxied,
        };

        let response = if !self.auth_email.is_empty() {
            // 使用 Email + API Key 认证
            self.client
                .post(&url)
                .header("X-Auth-Email", &self.auth_email)
                .header("X-Auth-Key", &self.auth_key)
                .header("Content-Type", "application/json")
                .json(&create_request)
                .send()
                .await?
        } else {
            // 使用 API Token 认证
            self.client.post(&url).json(&create_request).send().await?
        };

        let status = response.status();
        let response_text = response.text().await?;
        
        // 检查响应状态码
        if !status.is_success() {
            return Err(format!("API 请求失败，状态码 {}: {}。请检查您的 API 凭据。", status, response_text).into());
        }
        
        let response_data: Result<ApiResponse<DnsRecord>, _> = serde_json::from_str(&response_text);
        match response_data {
            Ok(response_data) => {
                if response_data.success {
                    Ok(response_data.result)
                } else {
                    Err(format!("无法创建 DNS 记录: {:?}", response_data.errors).into())
                }
            }
            Err(_) => {
                // 解析失败，可能是认证错误或无效的响应格式
                Err(format!("无法解析 API 响应。请检查您的 API 凭据。\n响应: {}", response_text).into())
            }
        }
    }
}