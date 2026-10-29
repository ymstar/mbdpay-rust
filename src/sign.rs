//! 面包多Pay签名算法实现
//! 文档: https://doc.mbd.pub/api/qian-ming-suan-fa

use md5::{Digest, Md5};
use serde_json::Value;
use std::collections::BTreeMap;

/// 对参数进行签名
///
/// # 参数
/// - `attributes`: 请求参数（不含sign）
/// - `app_key`: 面包多Pay app_key
///
/// # 返回
/// MD5签名字符串
///
/// # 示例
/// ```
/// use mbdpay::sign;
/// use serde_json::json;
///
/// let data = json!({
///     "app_id": "194024590982810",
///     "amount_total": 100,
///     "channel": "h5",
///     "description": "312"
/// });
/// let key = "920ee68b4e16df01d0cd6b2ca161195d";
/// let result = sign(&data, key);
/// assert_eq!(result.len(), 32);
/// ```
pub fn sign(attributes: &Value, app_key: &str) -> String {
    let mut sorted: BTreeMap<&String, &Value> = BTreeMap::new();
    if let Some(obj) = attributes.as_object() {
        for (k, v) in obj {
            sorted.insert(k, v);
        }
    }

    let mut sign_str = String::new();
    for (i, (k, v)) in sorted.iter().enumerate() {
        if i > 0 {
            sign_str.push('&');
        }
        sign_str.push_str(k);
        sign_str.push('=');
        sign_str.push_str(&value_to_string(v));
    }
    sign_str.push_str("&key=");
    sign_str.push_str(app_key);

    let mut hasher = Md5::new();
    hasher.update(sign_str.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => v.to_string(),
    }
}
