use std::borrow::Cow;

use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};

use crate::client::HttpClient;

pub mod forms;
mod routes;

pub fn spawn_server((address, port): (Cow<'static, str>, u16)) -> crate::Result<Server> {
    let server = HttpServer::new(move || {
        let endpoints = web::scope("/readme")
            .service(routes::get_repo_info)
            .app_data(Data::new(HttpClient::new()));

        App::new()
            .wrap(actix_cors::Cors::permissive())
            .service(endpoints)
    })
    .bind((&*address, port))?
    .run();

    tracing::info!("Server spawned!");

    Ok(server)
}
