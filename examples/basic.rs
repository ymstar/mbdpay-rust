use mbdpay::{AlipayReq, Client, WxJsapiReq};

fn main() {
    // 运行前请替换为你自己的 app_id 和 app_key
    // 获取地址: https://mbd.pub/dev
    let client = Client::new("your_app_id", "your_app_key");

    // === 1. 获取用户 openid 跳转链接 ===
    println!("=== 获取 openid 跳转链接 ===");
    let url = client.get_openid_redirect_url("https://your-site.com/openid_callback");
    println!("跳转URL: {}", url);
    println!();

    // === 2. 微信 JSAPI 支付 ===
    println!("=== 微信 JSAPI 支付 ===");
    let req = WxJsapiReq {
        openid: "用户openid".to_string(),
        description: "测试商品".to_string(),
        amount_total: 100,
        callback_url: "https://your-site.com/pay/callback".to_string(),
        out_trade_no: Some("ORDER_001".to_string()),
    };
    match client.wx_jsapi(&req) {
        Ok(res) => println!("JSAPI参数: {:?}", res),
        Err(e) => println!("错误: {}", e),
    }
    println!();

    // === 3. 微信 H5 支付 ===
    println!("=== 微信 H5 支付 ===");
    match client.wx_h5("测试商品", 100, Some("ORDER_002")) {
        Ok(res) => println!("H5支付链接: {}", res.h5_url.unwrap_or_default()),
        Err(e) => println!("错误: {}", e),
    }
    println!();

    // === 4. 支付宝支付 ===
    println!("=== 支付宝支付 ===");
    let alipay_req = AlipayReq {
        url: "https://your-site.com/pay/callback".to_string(),
        description: "测试商品".to_string(),
        amount_total: 100,
        out_trade_no: Some("ORDER_003".to_string()),
        callback_url: None,
    };
    match client.alipay(&alipay_req) {
        Ok(res) => {
            let body = res.body.unwrap_or_default();
            let preview = if body.len() > 100 { &body[..100] } else { &body };
            println!("支付宝表单HTML: {}...", preview);
        }
        Err(e) => println!("错误: {}", e),
    }
    println!();

    // === 5. 查询订单 ===
    println!("=== 查询订单 ===");
    match client.search_order("ORDER_001") {
        Ok(res) => println!("订单信息: {:?}", res),
        Err(e) => println!("错误: {}", e),
    }
    println!();

    // === 6. 退款 ===
    println!("=== 退款 ===");
    match client.refund("面包多订单号") {
        Ok(res) => println!("退款结果: {:?}", res),
        Err(e) => println!("错误: {}", e),
    }
}
