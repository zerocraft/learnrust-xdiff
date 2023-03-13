mod xdiff;
mod xreq;

use crate::ExtraArgs;
use anyhow::{Ok, Result};
use async_trait::async_trait;
use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Method, Response,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Write;
use std::str::FromStr;
use tokio::fs;
use url::Url;

pub use xdiff::{DiffConfig, DiffProfile, ResponseProfile};
pub use xreq::ReqConfig;

#[async_trait]
pub trait LoadConfig
where
    Self: Sized + ValidateConfig + DeserializeOwned,
{
    async fn load_yaml(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    fn from_yaml(content: &str) -> Result<Self> {
        let config: Self = serde_yaml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }
}

pub trait ValidateConfig {
    fn validate(&self) -> Result<()>;
}

pub fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    v == &T::default()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestProfile {
    #[serde(with = "http_serde::method", default)]
    pub method: Method,
    pub url: Url,
    #[serde(skip_serializing_if = "empty_json_value", default)]
    pub params: Option<serde_json::Value>,
    #[serde(
        skip_serializing_if = "HeaderMap::is_empty",
        with = "http_serde::header_map",
        default
    )]
    pub headers: HeaderMap,
    #[serde(skip_serializing_if = "empty_json_value", default)]
    pub body: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct ResponseExt(Response);

impl ResponseExt {
    pub fn into_inner(self) -> Response {
        self.0
    }

    pub async fn filter_text(self, profile: &ResponseProfile) -> Result<String> {
        let res = self.0;
        let mut output = String::new();

        write!(&mut output, "{}", get_status_text(&res)?)?;

        write!(
            &mut output,
            "{}",
            get_header_text(&res, &profile.skip_headers)?
        )?;

        write!(
            &mut output,
            "{}",
            get_body_text(res, &profile.skip_body).await?
        )?;

        Ok(output)
    }

    pub fn get_header_keys(&self) -> Vec<String> {
        let res = &self.0;
        let headers = res.headers();
        headers
            .iter()
            .map(|(k, _)| k.as_str().to_string())
            .collect()
    }
}

impl FromStr for RequestProfile {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut url = Url::parse(s)?;
        let qs = url.query_pairs();
        let mut params = serde_json::json!({});
        for (k, v) in qs {
            params[&*k] = v.parse()?;
        }
        url.set_query(None);
        Ok(Self::new(
            Method::GET,
            url,
            Some(params),
            HeaderMap::new(),
            None,
        ))
    }
}

impl RequestProfile {
    pub fn new(
        method: Method,
        url: Url,
        params: Option<serde_json::Value>,
        headers: HeaderMap,
        body: Option<serde_json::Value>,
    ) -> Self {
        Self {
            method,
            url,
            params,
            headers,
            body,
        }
    }

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

    pub fn get_url(&self, args: &ExtraArgs) -> anyhow::Result<String> {
        let (_, params, _) = self.generate(args)?;
        let mut url = self.url.clone();
        if !params.as_object().unwrap().is_empty() {
            let query = serde_qs::to_string(&params)?;
            url.set_query(Some(&query));
        }
        Ok(url.to_string())
    }

    fn generate(&self, args: &ExtraArgs) -> Result<(HeaderMap, serde_json::Value, String)> {
        let mut headers = self.headers.clone();
        let mut query = self.params.clone().unwrap_or_else(|| serde_json::json!({}));
        let mut body = self.body.clone().unwrap_or_else(|| serde_json::json!({}));

        for (k, v) in &args.headers {
            headers.insert(HeaderName::from_str(k)?, v.parse()?);
        }

        if !headers.contains_key(header::CONTENT_TYPE) {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
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

impl ValidateConfig for RequestProfile {
    fn validate(&self) -> Result<()> {
        if let Some(params) = self.params.as_ref() {
            if !params.is_object() {
                return Err(anyhow::anyhow!(
                    "config params error\n {}",
                    serde_yaml::to_string(params)?
                ));
            }
        }
        if let Some(body) = self.body.as_ref() {
            if !body.is_object() {
                return Err(anyhow::anyhow!(
                    "config body error\n {}",
                    serde_yaml::to_string(body)?
                ));
            }
        }
        Ok(())
    }
}

pub fn get_status_text(res: &Response) -> Result<String> {
    Ok(format!("{:?} {}\n", res.version(), res.status()))
}

pub fn get_header_text(res: &Response, skip: &[String]) -> anyhow::Result<String> {
    let mut output = String::new();
    let headers = res.headers();
    for (k, v) in headers.iter() {
        if !skip.iter().any(|sh| sh == k.as_str()) {
            writeln!(&mut output, "{}: {:?}", k, v)?;
        }
    }
    Ok(output)
}

pub async fn get_body_text(res: Response, skip: &[String]) -> anyhow::Result<String> {
    let mut output = String::new();
    let headers = res.headers();
    //output.push_str("\n");
    let ct = get_content_type(&headers);
    let text = res.text().await?;
    match ct.unwrap().as_str() {
        n if n == mime::APPLICATION_JSON => {
            let text = filter_json(&text, skip)?;
            output.push_str(&text);
        }
        _ => {
            output.push_str(&text);
        }
    }
    Ok(output)
}

fn get_content_type(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(";").next().map(|v| v.to_string()))
}

fn filter_json(text: &str, skip: &[String]) -> Result<String> {
    let mut json = serde_json::from_str(&text)?;
    if let serde_json::Value::Object(ref mut obj) = json {
        for k in skip {
            obj.remove(k);
        }
    }
    Ok(serde_json::to_string_pretty(&json)?)
}

fn empty_json_value(v: &Option<serde_json::Value>) -> bool {
    v.as_ref().map_or(true, |v| {
        v.is_null() || (v.is_object() && v.as_object().unwrap().is_empty())
    })
}

#[cfg(test)]
mod tests {
    use mockito::{mock, Mock};
    use reqwest::StatusCode;
    use serde_json::json;

    use super::*;

    #[test]
    fn t1() {
        assert_eq!("application/json", mime::APPLICATION_JSON);
    }

    #[test]
    fn t2() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
        assert_eq!(
            get_content_type(&headers),
            Some("application/json".to_string())
        );
    }

    #[tokio::test]
    async fn t3() {
        let body = json!({"id":1,"title":"go"});
        let path = "/todo?a=1&b=2";
        mock_server(path, &body);
        let url = format!("{}{}", mockito::server_url(), path); //Url::parse("https://httpbin.org/get").unwrap();
        println!("{}", url);
        let url = Url::parse(&url).unwrap();
        let profile = RequestProfile::new(Method::GET, url, None, HeaderMap::new(), Some(body));
        let res = profile
            .send(&Default::default())
            .await
            .unwrap()
            .into_inner();
        assert_eq!(res.status(), StatusCode::OK);
    }

    fn mock_server(path: &str, body: &serde_json::Value) {
        let _m = mock("GET", path)
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "666")
            .with_body(serde_json::to_string(&body).unwrap())
            .create();
    }
}
