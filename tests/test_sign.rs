use mbdpay::sign;
use serde_json::json;

#[test]
fn test_sign_produces_valid_md5() {
    let data = json!({
        "app_id": "194024590982810",
        "amount_total": 100,
        "channel": "h5",
        "description": "312"
    });
    let key = "920ee68b4e16df01d0cd6b2ca161195d";
    let result = sign(&data, key);
    assert_eq!(result.len(), 32);
    assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_sign_order_independent() {
    let key = "test_key";

    let data1 = json!({"a": "1", "b": "2", "c": "3"});
    let data2 = json!({"c": "3", "b": "2", "a": "1"});

    assert_eq!(sign(&data1, key), sign(&data2, key));
}

#[test]
fn test_get_openid_redirect_url() {
    let client = mbdpay::Client::new("test_app_id", "test_app_key");
    let url = client.get_openid_redirect_url("https://example.com/cb");
    assert!(url.starts_with("https://mbd.pub/openid?"));
    assert!(url.contains("app_id=test_app_id"));
}
