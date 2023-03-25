use rocket::response::content::RawHtml;

macro_rules! html_error {
    ($code:literal, $name:ident, $status:literal, $description:expr) => {
        #[catch($code)]
        pub fn $name() -> RawHtml<String> {
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
            <code>{2}</code>
        <hr />
        </main>
    </body>
</html>",
                $code, $status, $description,
            ))
        }
    };
}

html_error!(
    400,
    bad_request,
    "Bad Request",
    "The request could not be understood by the server due to malformed syntax."
);
html_error!(
    401,
    unauthorized,
    "Unauthorized",
    "The request requires user authentication."
);
html_error!(
    404,
    not_found,
    "Not Found",
    "The requested resource could not be found."
);
html_error!(500, internal_server_error, "Internal Server Error", "The server encountered an internal error or misconfiguration and was unable to complete your request.");
