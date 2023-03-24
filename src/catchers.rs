use rocket::{request::Request, response::content::RawHtml};

macro_rules! html_error {
    ($code:literal, $status:literal, $description:expr) => {
        RawHtml(format!(
"<!DOCTYPE html>
<html lang=\"en\">
    <head>
        <meta charset=\"utf-8\">
        <title>{0}</title>
    </head>
    <body>
        <main>
            <h1>{0}: {1}</h1>
            <p>{2}</p>
        <hr />
        </main>
    </body>
</html>",
            $code, $status, $description,
        ))
    };
}

#[catch(400)]
pub fn bad_request() -> RawHtml<String> {
    html_error!(
        400,
        "Bad Request",
        "The request could not be understood by the server due to malformed syntax."
    )
}

#[catch(401)]
pub fn unauthorized() -> RawHtml<String> {
    html_error!(
        401,
        "Unauthorized",
        "The request requires user authentication."
    )
}

#[catch(404)]
pub fn not_found(req: &Request) -> RawHtml<String> {
    html_error!(
        404,
        "Not Found",
        format!("The route <code>{}</code> could not be found.", req.uri())
    )
}

#[catch(500)]
pub fn internal_server_error() -> RawHtml<String> {
    html_error!(500, "Internal Server Error", "The server encountered an internal error or misconfiguration and was unable to complete your request.")
}
