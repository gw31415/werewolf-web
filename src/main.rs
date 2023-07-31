use clap::{Parser, Subcommand};

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
struct Args {
    #[clap(subcommand)]
    subcmd: SubCmd,
}

#[derive(Subcommand)]
enum SubCmd {
    Run,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match Args::parse().subcmd {
        SubCmd::Run => {
            let server = MasterRouter::new().start();
            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(server.clone()))
                    .service(index)
                    .service(werewolf_ws)
            })
            .bind(("0.0.0.0", 3232))?
            .run()
            .await
        }
    }
}
