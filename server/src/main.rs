use clap::Parser;

mod master_router;
mod session;

use actix::{Actor, Addr};
use actix_files::NamedFile;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use master_router::MasterRouter;

#[get("/")]
async fn index() -> impl Responder {
    NamedFile::open_async("./index.html").await.unwrap()
}

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
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let Cli { port, expose } = Cli::parse();
    let server = MasterRouter::new().start();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .service(index)
            .service(werewolf_ws)
    })
    .bind((if expose { "0.0.0.0" } else { "127.0.0.1" }, port))?
    .run()
    .await
}
