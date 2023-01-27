use axum::{http::HeaderMap, body::{Bytes}};

use tracing::{info, trace};
use serde::{Serialize, Deserialize};
use serde_json as json;
use crate::{utils::{hash_hmac_sha256,shell_exec},config::get_config};


#[derive(Serialize, Deserialize, Debug)]
pub struct GithubPayload {
    repository: GithubRequestDataRepository,
    #[serde(rename="ref")]
    _ref: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct GithubRequestDataRepository {
    full_name: String,
}

pub async fn github(headers: HeaderMap, body_bytes:Bytes) -> () {
    // 读取bytes用于sha256校验
    let body_bytes : Vec<u8> = body_bytes.into_iter().collect();
    // 取body的string
    let string:String = String::from_utf8(body_bytes.clone()).expect("");
    trace!("github request body string: {}",string);
    // 转化为所需结构体
    let github_request : GithubPayload = json::from_str(&string).unwrap();
    info!("github request: {}",json::to_string(&github_request).unwrap());

    // 取github当前的事件
    let x_github_event = headers.get("x-github-event").unwrap().to_str().unwrap().to_string();
    // 取sha256签名
    let x_hub_signature_256 = headers.get("x-hub-signature-256").unwrap().to_str().unwrap().to_string();

    // 起线程异步处理
    tokio::spawn(async move {
        let config = get_config();
        // 遍历配置的钩子事件
        for item in config.github {
            // 验证签名
            if !item.password.is_empty() && !check_signature_sha256(&body_bytes, &item.password, &x_hub_signature_256) {
                continue;
            }
            // 匹配
            if !item.name.is_empty() && item.name != github_request.repository.full_name {
                continue;
            }
            if let Some(ref config_event) = item.event {
                if *config_event != x_github_event {
                    continue;
                }
                
            }
            if let Some(ref config_ref) = item._ref {
                if let Some(ref github_ref) = github_request._ref {
                    if config_ref != github_ref {
                        continue;
                    }
                }
            }
            // 执行钩子事件
            for cmd in item.cmds {
                println!("CMD: {}", cmd);
                shell_exec(cmd.as_str()).await.unwrap();
            }
        }
    });
    
}

fn check_signature_sha256(bytes: &Vec<u8>, secret: &String, signature: &String) -> bool {
    let hash = hash_hmac_sha256(bytes.as_slice(), secret.as_bytes());
    let index = signature.find("=");
    if index.is_none() {
        return false;
    }
    return hash == signature.split_at(index.unwrap() + 1).1;
}