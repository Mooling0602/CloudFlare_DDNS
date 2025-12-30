use std::time::Duration;

/// 获取当前公网 IPv4 地址
pub async fn get_external_ipv4() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    
    let response = client.get("https://4.ipw.cn").send().await?;
    
    if response.status().is_success() {
        let ip = response.text().await?.trim().to_string();
        Ok(ip)
    } else {
        Err(format!("获取 IPv4 地址失败: {}", response.status()).into())
    }
}

/// 获取当前公网 IPv6 地址
pub async fn get_external_ipv6() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    
    let response = client.get("https://6.ipw.cn").send().await?;
    
    if response.status().is_success() {
        let ip = response.text().await?.trim().to_string();
        Ok(ip)
    } else {
        Err(format!("获取 IPv6 地址失败: {}", response.status()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_external_ipv4() {
        let result = get_external_ipv4().await;
        assert!(result.is_ok());
        let ip = result.unwrap();
        println!("Current IPv4: {}", ip);
    }
}