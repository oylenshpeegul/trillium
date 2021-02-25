use myco::Conn;
use myco_router::{routes, RouterConnExt};

pub fn main() {
    env_logger::init();
    myco_smol_server::run(routes![
        get "/" |conn: Conn| async move { conn.ok("hello everyone") },

        get "/hello/:planet" |conn: Conn| async move {
            let planet = conn.param("planet").unwrap();
            let response_body = format!("hello {}", planet);
            conn.ok(response_body)
        }
    ]);
}