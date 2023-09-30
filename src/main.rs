mod api;
mod catchers;
mod file;
mod routes;
#[cfg(test)]
mod tests;

use catchers::*;
use routes::*;

#[macro_use]
extern crate rocket; // use all rocket macros

#[launch]
fn rocket() -> _ {
    let routes = routes![
        get_file,
        file_amount,
        get_latest_files,
        create_optimized_images
    ];
    let catchers = catchers![bad_request, unauthorized, not_found, internal_server_error];
    rocket::build().register("/", catchers).mount("/", routes)
}
