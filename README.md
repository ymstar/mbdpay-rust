# 面包多Pay Rust SDK

面向个人的支付集成平台 [面包多Pay](https://mbd.pub) 的 Rust SDK。

## 安装

### Cargo.toml

```toml
[dependencies]
mbdpay = "0.1"
```

### 手动编译

```bash
git clone https://github.com/yangming/mbdpay-rust.git
cd mbdpay-rust
cargo build
```

## 使用

### 初始化

```rust
use mbdpay::Client;

let client = Client::new("your_app_id", "your_app_key");
```

### 微信 JSAPI 支付

```rust
use mbdpay::{Client, WxJsapiReq};

let client = Client::new("your_app_id", "your_app_key");
let req = WxJsapiReq {
    openid: "用户openid".to_string(),
    description: "商品描述".to_string(),
    amount_total: 100,  // 金额，单位：分
    callback_url: "https://callback".to_string(),
    out_trade_no: Some("ORDER_001".to_string()),
};
let res = client.wx_jsapi(&req)?;
// res.app_id, res.time_stamp, res.nonce_str, res.package, res.sign_type, res.pay_sign
```

### 微信 H5 支付

```rust
let res = client.wx_h5("商品描述", 100, Some("ORDER_001"))?;
println!("H5链接: {}", res.h5_url.unwrap());
```

### 支付宝支付

```rust
use mbdpay::AlipayReq;

let req = AlipayReq {
    url: "https://callback".to_string(),
    description: "商品描述".to_string(),
    amount_total: 100,
    out_trade_no: Some("ORDER_001".to_string()),
    callback_url: None,
};
let res = client.alipay(&req)?;
// res.body 为HTML表单
```

### 退款

```rust
let res = client.refund("面包多订单号")?;
```

### 查询订单

```rust
let res = client.search_order("你的订单号")?;
```

### 获取用户 OpenID 跳转链接

```rust
let url = client.get_openid_redirect_url("https://your-site.com/openid_callback");
```

## 签名算法

参数按 key 排序后拼接为 `key1=value1&key2=value2`，末尾追加 `&key=APP_KEY`，取 MD5。

文档：https://doc.mbd.pub/api/qian-ming-suan-fa

## License

MIT
