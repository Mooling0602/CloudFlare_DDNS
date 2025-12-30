# CloudFlare DDNS

一个用 Rust 编写的 CloudFlare 动态域名系统 (DDNS) 工具，用于自动更新 CloudFlare 托管的 DNS 记录，以指向当前的公网 IP 地址。

## 功能

- 自动检测公网 IP 地址变化
- 使用 CloudFlare API 更新 DNS 记录
- 支持 A 记录 (IPv4) 和 AAAA 记录 (IPv6)
- 支持配置文件管理
- 支持定时运行模式
- 支持强制更新选项

## 安装

### 从源码构建

```bash
# 克隆项目
git clone <your-repo-url>
cd CloudFlare_DDNS

# 构建项目
cargo build --release
```

## 配置

创建一个 `config.json` 文件（或使用 `-c` 参数指定其他路径）：

```json
{
  "cloudflare": {
    "auth_type": "token",
    "auth_email": null,
    "auth_key": null,
    "api_token": "your_api_token_here",
    "zone_name": "your_domain.com"
  },
  "dns_records": [
    {
      "name": "subdomain.your_domain.com",
      "type": "A",
      "ttl": 60,
      "proxied": false,
      "ip_version": "v4"
    },
    {
      "name": "ipv6-subdomain.your_domain.com",
      "type": "AAAA",
      "ttl": 60,
      "proxied": false,
      "ip_version": "v6"
    }
  ]
}
```

### 认证方式

- **API Token (推荐)**: 在 CloudFlare 控制台中创建一个具有 DNS 编辑权限的 API Token
- **Email + API Key**: 使用 CloudFlare 账户邮箱和全局 API Key

### 配置项说明

- `auth_type`: 认证类型 (`token` 或 `emailkey`)
- `api_token`: CloudFlare API Token (当 auth_type 为 token 时)
- `auth_email`: CloudFlare 账户邮箱 (当 auth_type 为 email_key 时)
- `auth_key`: CloudFlare 全局 API Key (当 auth_type 为 email_key 时)
- `zone_name`: 要更新 DNS 记录的域名
- `dns_records`: 要更新的 DNS 记录列表
  - `name`: DNS 记录名称
  - `type`: 记录类型 (A, AAAA 等)
  - `ttl`: TTL 值
  - `proxied`: 是否启用 CloudFlare 代理
  - `ip_version`: IP 版本 (v4 或 v6)

## 使用方法

### 单次运行

```bash
# 使用默认配置文件 config.json
./cloudflare_ddns

# 指定配置文件
./cloudflare_ddns -c /path/to/config.json

# 强制更新，即使 IP 没有变化
./cloudflare_ddns --force

# 只检查 IP，不更新 DNS 记录
./cloudflare_ddns --check-only
```

### 定时运行

```bash
# 每 5 分钟检查一次 IP 变化
./cloudflare_ddns --interval 5
```

## 开发

### 项目结构

- `src/main.rs`: 主程序入口
- `src/ip_utils.rs`: IP 地址获取功能
- `src/cloudflare.rs`: CloudFlare API 交互功能
- `src/config.rs`: 配置结构定义
- `src/scheduler.rs`: 定时任务功能

## 贡献

欢迎提交 Issue 和 Pull Request 来改进此项目。

## 许可证

MIT