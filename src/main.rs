use clap::Parser;
use config::Config;
use cloudflare::UpdateDnsRecordParams;

mod ip_utils;
mod cloudflare;
mod config;
mod scheduler;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.json")]
    config: String,
    
    /// 强制更新，即使 IP 没有变化
    #[arg(short, long)]
    force: bool,
    
    /// 只检查 IP，不更新 DNS 记录
    #[arg(long)]
    check_only: bool,
    
    /// 定时运行模式，指定检查间隔（秒）
    #[arg(short, long)]
    interval: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    
    let args = Args::parse();
    println!("程序启动");
    println!("参数解析完成: {:?}", args.config);
    
    // 如果指定了定时运行间隔，则以定时模式运行
    if let Some(interval) = args.interval {
        println!("以定时模式启动 CloudFlare DDNS，间隔 {} 秒", interval);
        
        // 创建一个闭包，用于执行 DDNS 更新逻辑
        let config_path = args.config.clone();
        let force_update = args.force;
        let check_only = args.check_only;
        
        scheduler::run_with_schedule(interval, move || {
            let config_path = config_path.clone();
            let force_update = force_update;
            let check_only = check_only;
            
            async move {
                run_ddns_update(&config_path, force_update, check_only).await
            }
        }).await;
    } else {
        // 单次运行模式
        run_ddns_update(&args.config, args.force, args.check_only).await?;
    }
    
    Ok(())
}

async fn run_ddns_update(config_path: &str, force: bool, check_only: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("准备加载配置文件: {}", config_path);
    // 从配置文件加载配置
    let config = load_config(config_path)?;
    
    // 在 check_only 模式下，我们只获取外部 IP，不进行 API 调用
    if check_only {
        println!("仅检查模式 - 正在获取配置中的记录的外部 IP 地址...");
        
        for record_config in &config.dns_records {
            let ip_version = record_config.get_ip_version()
                .map_err(|e| format!("IP 版本无效: {}", e))?;
            let current_ip = match ip_version {
                config::IpVersion::V4 => ip_utils::get_external_ipv4().await?,
                config::IpVersion::V6 => ip_utils::get_external_ipv6().await?,
            };
            
            println!("外部 IP 地址 {} ({}): {}", record_config.name, record_config.ip_version, current_ip);
        }
        
        println!("仅检查模式完成 - 未更新任何 DNS 记录.");
        return Ok(());
    }
    
    // 创建 CloudFlare 客户端 (仅在非 check_only 模式下)
    let auth_type = config.cloudflare.get_auth_type()
        .map_err(|e| format!("认证类型无效: {}", e))?;
    let cf_client = match auth_type {
        config::AuthType::EmailKey => {
            let email = config.cloudflare.auth_email
                .as_ref()
                .ok_or("使用邮箱+密钥认证时，邮箱是必需的")?;
            let key = config.cloudflare.auth_key
                .as_ref()
                .ok_or("使用邮箱+密钥认证时，密钥是必需的")?;
            cloudflare::CloudflareClient::new(email.clone(), key.clone())
        },
        config::AuthType::Token => {
            let token = config.cloudflare.api_token
                .as_ref()
                .ok_or("使用令牌认证时，API 令牌是必需的")?;
            cloudflare::CloudflareClient::new_with_token(token.clone())
        }
    };
    
    // 获取 Zone ID - 添加更友好的错误处理
    let zone_id = match cf_client.get_zone_id(&config.cloudflare.zone_name).await {
        Ok(id) => {
            println!("区域 ID: {}", id);
            id
        },
        Err(e) => {
            return Err(format!("无法获取区域 ID。请检查您的 API 凭据和域名。错误: {}", e).into());
        }
    };
    
    // 处理每个 DNS 记录
    for record_config in &config.dns_records {
        println!("正在处理记录: {}", record_config.name);
        
        let ip_version = record_config.get_ip_version()
            .map_err(|e| format!("IP 版本无效: {}", e))?;
        let current_ip = match ip_version {
            config::IpVersion::V4 => ip_utils::get_external_ipv4().await?,
            config::IpVersion::V6 => ip_utils::get_external_ipv6().await?,
        };
        
        println!("当前外部 IP: {}", current_ip);
        
        // 获取现有的 DNS 记录 - 添加更友好的错误处理
        match cf_client.get_dns_record_id(&zone_id, &record_config.name).await {
            Ok(record_id) => {
                let existing_record = match cf_client.get_dns_record(&zone_id, &record_id).await {
                    Ok(record) => record,
                    Err(e) => {
                        return Err(format!("无法获取 DNS 记录详情。请检查您的 API 凭据。错误: {}", e).into());
                    }
                };
                
                // 检查 IP 是否发生变化，或者是否强制更新
                if existing_record.content != current_ip || force {
                    println!("IP 已更改或强制更新请求.正在更新 DNS 记录...");
                    
                    let updated_record = match cf_client
                        .update_dns_record(
                            UpdateDnsRecordParams {
                                zone_id: &zone_id,
                                record_id: &record_id,
                                record_type: &record_config.r#type,
                                name: &record_config.name,
                                content: &current_ip,
                                ttl: record_config.ttl,
                                proxied: record_config.proxied,
                            }
                        )
                        .await {
                            Ok(record) => record,
                            Err(e) => {
                                return Err(format!("无法更新 DNS 记录。请检查您的 API 凭据和权限。错误: {}", e).into());
                            }
                        };
                    
                    println!(
                        "DNS 记录更新成功！新 IP: {}",
                        updated_record.content
                    );
                } else {
                    println!("IP 未更改.无需更新.");
                }
            }
            Err(_) => {
                // 如果记录不存在，创建新的记录
                println!("DNS 记录不存在，正在创建新记录...");
                
                let new_record = match cf_client
                    .create_dns_record(
                        &zone_id,
                        &record_config.r#type,
                        &record_config.name,
                        &current_ip,
                        record_config.ttl,
                        record_config.proxied,
                    )
                    .await {
                        Ok(record) => record,
                        Err(e) => {
                            return Err(format!("无法创建 DNS 记录。请检查您的 API 凭据和权限。错误: {}", e).into());
                        }
                    };
                
                println!("新的 DNS 记录已创建: {}", new_record.content);
            }
        }
    }
    
    Ok(())
}

fn load_config(config_path: &str) -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    let content = std::fs::read_to_string(config_path)?;
    println!("正在加载配置文件: {}", config_path);
    println!("配置文件内容: {}", content);
    
    let config: Config = match serde_json::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("JSON 解析错误: {}", e);
            eprintln!("错误位置: 行 {}, 列 {}", e.line(), e.column());
            return Err(Box::new(e));
        }
    };
    Ok(config)
}
