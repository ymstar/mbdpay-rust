//! 面包多Pay (MianBaoDuo Pay) Rust SDK
//!
//! 面向个人的支付集成平台面包多Pay的Rust SDK。
//! 支持微信JSAPI支付、微信H5支付、支付宝支付、退款、订单查询等功能。
//!
//! # 示例
//!
//! ```no_run
//! use mbdpay::Client;
//!
//! let client = Client::new("your_app_id", "your_app_key");
//!
//! // 微信H5支付
//! let result = client.wx_h5("测试商品", 100, Some("ORDER_001")).unwrap();
//! println!("H5链接: {}", result.h5_url.unwrap_or_default());
//! ```

mod sign;

pub use sign::sign;

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use thiserror::Error;

/// SDK 错误类型
#[derive(Error, Debug)]
pub enum MbdPayError {
    #[error("API返回错误: {0}")]
    ApiError(String),
    #[error("HTTP请求失败: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON解析失败: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// 面包多Pay客户端
pub struct Client {
    app_id: String,
    app_key: String,
    domain: String,
    http_client: reqwest::blocking::Client,
}

/// 微信JSAPI支付请求参数
#[derive(Debug, Serialize)]
pub struct WxJsapiReq {
    pub openid: String,
    pub description: String,
    pub amount_total: i64,
    pub callback_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_trade_no: Option<String>,
}

/// 微信JSAPI支付返回参数
#[derive(Debug, Deserialize)]
pub struct WxJsapiRes {
    #[serde(rename = "appId")]
    pub app_id: Option<String>,
    #[serde(rename = "timeStamp")]
    pub time_stamp: Option<String>,
    #[serde(rename = "nonceStr")]
    pub nonce_str: Option<String>,
    pub package: Option<String>,
    #[serde(rename = "signType")]
    pub sign_type: Option<String>,
    #[serde(rename = "paySign")]
    pub pay_sign: Option<String>,
}

/// 微信H5支付返回参数
#[derive(Debug, Deserialize)]
pub struct WxH5Res {
    pub h5_url: Option<String>,
}

/// 支付宝支付请求参数
#[derive(Debug, Serialize)]
pub struct AlipayReq {
    pub url: String,
    pub description: String,
    pub amount_total: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_trade_no: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
}

/// 支付宝支付返回参数
#[derive(Debug, Deserialize)]
pub struct AlipayRes {
    pub body: Option<String>,
}

/// 退款返回参数
#[derive(Debug, Deserialize)]
pub struct RefundRes {
    pub code: Option<i64>,
    pub info: Option<String>,
    pub message: Option<String>,
}

/// 订单状态
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderState {
    Unpaid,
    Paid,
    Settled,
    InComplain,
    ComplainFinished,
    ComplainTimeout,
    InComplainHandling,
}

/// 查询订单返回参数
#[derive(Debug, Deserialize)]
pub struct SearchOrderRes {
    pub order_id: Option<String>,
    pub charge_id: Option<String>,
    pub description: Option<String>,
    pub amount: Option<i64>,
    pub state: Option<i64>,
    pub create_time: Option<i64>,
    pub payway: Option<i64>,
    pub refund_state: Option<i64>,
    pub plusinfo: Option<String>,
}

impl Client {
    /// 创建客户端
    ///
    /// # 参数
    /// - `app_id`: 面包多Pay app_id
    /// - `app_key`: 面包多Pay app_key
    pub fn new(app_id: &str, app_key: &str) -> Self {
        Self::with_domain(app_id, app_key, "https://api.mianbaoduo.com")
    }

    /// 创建客户端（自定义域名）
    pub fn with_domain(app_id: &str, app_key: &str, domain: &str) -> Self {
        let http_client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("failed to build HTTP client");
        Self {
            app_id: app_id.to_string(),
            app_key: app_key.to_string(),
            domain: domain.to_string(),
            http_client,
        }
    }

    /// 获取用户openid的跳转URL
    /// 文档: https://doc.mbd.pub/api/huo-qu-yong-hu-openid
    pub fn get_openid_redirect_url(&self, target_url: &str) -> String {
        format!(
            "https://mbd.pub/openid?app_id={}&target_url={}",
            urlencoding_encode(&self.app_id),
            urlencoding_encode(target_url)
        )
    }

    /// 微信JSAPI支付
    /// 文档: https://doc.mbd.pub/api/wei-xin-zhi-fu
    pub fn wx_jsapi(&self, req: &WxJsapiReq) -> Result<WxJsapiRes, MbdPayError> {
        let body = serde_json::to_value(req)?;
        let result = self.post("/release/wx/prepay", body)?;
        check_error(&result)?;
        Ok(serde_json::from_value(result)?)
    }

    /// 微信H5支付
    /// 文档: https://doc.mbd.pub/api/wei-xin-h5-zhi-fu
    pub fn wx_h5(
        &self,
        description: &str,
        amount_total: i64,
        out_trade_no: Option<&str>,
    ) -> Result<WxH5Res, MbdPayError> {
        let mut body = json!({
            "channel": "h5",
            "description": description,
            "amount_total": amount_total,
        });
        if let Some(no) = out_trade_no {
            body["out_trade_no"] = json!(no);
        }
        let result = self.post("/release/wx/prepay", body)?;
        check_error(&result)?;
        Ok(serde_json::from_value(result)?)
    }

    /// 支付宝支付
    /// 文档: https://doc.mbd.pub/api/zhi-fu-bao-zhi-fu
    pub fn alipay(&self, req: &AlipayReq) -> Result<AlipayRes, MbdPayError> {
        let body = serde_json::to_value(req)?;
        let result = self.post("/release/alipay/pay", body)?;
        check_error(&result)?;
        Ok(serde_json::from_value(result)?)
    }

    /// 退款
    /// 文档: https://doc.mbd.pub/api/tui-kuan
    pub fn refund(&self, order_id: &str) -> Result<RefundRes, MbdPayError> {
        let body = json!({ "order_id": order_id });
        let result = self.post("/release/main/refund", body)?;
        check_error(&result)?;
        Ok(serde_json::from_value(result)?)
    }

    /// 查询订单
    /// 文档: https://doc.mbd.pub/api/ding-dan-cha-xun
    pub fn search_order(&self, out_trade_no: &str) -> Result<SearchOrderRes, MbdPayError> {
        let body = json!({ "out_trade_no": out_trade_no });
        let result = self.post("/release/main/search_order", body)?;
        check_error(&result)?;
        Ok(serde_json::from_value(result)?)
    }

    // ==================== 内部方法 ====================

    fn build_request(&self, params: Value) -> Value {
        let mut body = params.clone();
        if let Some(obj) = body.as_object_mut() {
            obj.insert("app_id".to_string(), json!(self.app_id));
            // 移除null值
            obj.retain(|_, v| !v.is_null());
        }
        let sign = self.compute_sign(&body);
        if let Some(obj) = body.as_object_mut() {
            obj.insert("sign".to_string(), json!(sign));
        }
        body
    }

    fn compute_sign(&self, params: &Value) -> String {
        let mut sorted: BTreeMap<&String, &Value> = BTreeMap::new();
        if let Some(obj) = params.as_object() {
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
        sign_str.push_str(&self.app_key);

        let mut hasher = Md5::new();
        hasher.update(sign_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn post(&self, path: &str, params: Value) -> Result<Value, MbdPayError> {
        let body = self.build_request(params);
        let url = format!("{}{}", self.domain, path);
        let resp = self.http_client.post(&url).json(&body).send()?;
        let result: Value = resp.json()?;
        Ok(result)
    }
}

fn check_error(result: &Value) -> Result<(), MbdPayError> {
    if let Some(err) = result.get("error") {
        if !err.is_null() {
            return Err(MbdPayError::ApiError(err.to_string()));
        }
    }
    Ok(())
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => v.to_string(),
    }
}

fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
