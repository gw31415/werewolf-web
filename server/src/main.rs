use clap::Parser;

mod master_router;
mod session;

use actix::{Actor, Addr};
use actix_files::Files;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use master_router::MasterRouter;

#[get("/ws")]
async fn werewolf_ws(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<MasterRouter>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        session::WsPlayerSession::new(srv.get_ref().clone()),
        &req,
        stream,
    )
}

#[derive(Parser)]
struct Cli {
    /// Port number to listen on.
    #[clap(short, long, default_value_t = 3232)]
    port: u16,
    /// Publish to the network.
    #[clap(long)]
    expose: bool,
    /// Path of the directory where the static files to be delivered are located.
    #[clap(long)]
    serve: Option<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let Cli {
        port,
        expose,
        serve,
    } = Cli::parse();
    let server = MasterRouter::new().start();
    HttpServer::new(move || {
        let app = App::new()
            .app_data(web::Data::new(server.clone()))
            .service(werewolf_ws);
        if let Some(dir) = &serve {
            app.service(Files::new("/", dir).index_file("index.html"))
        } else {
            app
        }
    })
    .bind((if expose { "0.0.0.0" } else { "127.0.0.1" }, port))?
    .run()
    .await
}
