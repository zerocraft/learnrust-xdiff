use std::str::FromStr;

use anyhow::{Ok, Result};
use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Method, Response,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use crate::{ExtraArgs, ResponseProfile};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestProfile {
    #[serde(with = "http_serde::method", default)]
    pub method: Method,
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<serde_json::Value>,
    #[serde(
        skip_serializing_if = "HeaderMap::is_empty",
        with = "http_serde::header_map",
        default
    )]
    pub headers: HeaderMap,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct ResponseExt(Response);

impl ResponseExt {
    pub async fn filter_text(self, profile: &ResponseProfile) -> Result<String> {
        let res = self.0;
        let mut output = String::new();
        output.push_str(&format!("{:?} {}\n", res.version(), res.status()));
        let headers = res.headers();
        for (k, v) in headers.iter() {
            if !profile.skip_headers.iter().any(|st| st == k.as_str()) {
                output.push_str(&format!("{}:{:?}\n", k, v));
            }
        }
        //output.push_str("\n");
        let ct = get_content_type(&headers);
        let text = res.text().await?;
        match ct.unwrap().as_str() {
            n if n == mime::APPLICATION_JSON => {
                let text = filter_json(&text, &profile.skip_body)?;
                output.push_str(&text);
            }
            _ => {
                output.push_str(&text);
            }
        }
        Ok(output)
    }
}

impl RequestProfile {
    pub async fn send(&self, args: &ExtraArgs) -> Result<ResponseExt> {
        let (headers, query, body) = self.generate(args)?;
        let client = reqwest::Client::new();
        let req = client
            .request(self.method.clone(), self.url.clone())
            .headers(headers)
            .query(&query)
            .body(body)
            .build()?;
        let res = client.execute(req).await?;

        Ok(ResponseExt(res))
    }

    pub fn generate(&self, args: &ExtraArgs) -> Result<(HeaderMap, serde_json::Value, String)> {
        let mut headers = self.headers.clone();
        let mut query = self.params.clone().unwrap_or_else(|| json!({}));
        let mut body = self.body.clone().unwrap_or_else(|| json!({}));

        for (k, v) in &args.headers {
            headers.insert(HeaderName::from_str(k)?, v.parse()?);
        }

        if !headers.contains_key(header::CONTENT_TYPE) {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        }

        for (k, v) in &args.querys {
            query[k] = v.parse()?;
        }

        for (k, v) in &args.bodys {
            body[k] = v.parse()?;
        }

        let ct = get_content_type(&headers);
        match ct.unwrap().as_str() {
            n if n == mime::APPLICATION_JSON => {
                let body = serde_json::to_string(&body)?;
                Ok((headers, query, body))
            }
            n if n == mime::APPLICATION_WWW_FORM_URLENCODED || n == mime::FORM_DATA => {
                let body = serde_urlencoded::to_string(&body)?;
                Ok((headers, query, body))
            }
            _ => Err(anyhow::anyhow!("unsupported")),
        }
    }
}

fn get_content_type(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().split(";").next())
        .flatten()
        .map(|v| v.to_string())
}

fn filter_json(text: &str, skip: &[String]) -> Result<String> {
    let mut json = serde_json::from_str(&text)?;
    match json {
        serde_json::Value::Object(ref mut obj) => {
            for k in skip {
                obj.remove(k);
            }
        }
        _ => {}
    }
    Ok(serde_json::to_string_pretty(&json)?)
}

#[cfg(test)]
mod tests {

    #[test]
    fn t1() {
        assert_eq!("application/json", mime::APPLICATION_JSON);
    }
}
