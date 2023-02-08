use api_server::routes;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = routes::create_router();

    println!("Starting server in http://0.0.0.0:3000/ ");

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
