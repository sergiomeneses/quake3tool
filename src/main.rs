use quake3::ServerBuilder;
fn main() {
    let server = ServerBuilder::new()
        .ip([104, 243, 45, 183])
        .port(28963_u16)
        .build()
        .expect("new");

    let status_response = server.get_status().expect("status");
    dbg!(status_response);
}
