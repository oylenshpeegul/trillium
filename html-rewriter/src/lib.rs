use futures_lite::io::BufReader;
pub use lol_async::html;
use lol_async::{html::Settings, lol};
use myco::async_trait;
use myco::http_types::headers::CONTENT_TYPE;
use myco::http_types::mime::Mime;
use myco::{http_types::Body, Conn, Handler};
use std::str::FromStr;

pub struct HtmlRewriter {
    settings: Box<dyn Fn() -> Settings<'static, 'static> + Send + Sync + 'static>,
}

#[async_trait]
impl Handler for HtmlRewriter {
    async fn run(&self, mut conn: Conn) -> Conn {
        let html = conn
            .headers_mut()
            .get(CONTENT_TYPE)
            .and_then(|c| Mime::from_str(c.as_str()).ok())
            .map(|m| m.subtype() == "html")
            .unwrap_or_default();

        if html && conn.inner().response_body().is_some() {
            let body = conn.inner_mut().take_response_body().unwrap();
            let (fut, reader) = lol(body, (self.settings)());
            async_global_executor::spawn_local(fut).detach();
            conn.body(Body::from_reader(BufReader::new(reader), None))
        } else {
            conn
        }
    }
}

impl HtmlRewriter {
    pub fn new(f: impl Fn() -> Settings<'static, 'static> + Send + Sync + 'static) -> Self {
        Self {
            settings: Box::new(f)
                as Box<dyn Fn() -> Settings<'static, 'static> + Send + Sync + 'static>,
        }
    }
}