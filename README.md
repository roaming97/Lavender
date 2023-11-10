# Lavender

The API for my [website](https://roaming97.com). It serves as a quick gateway to a file server while being flexible and letting me automate tasks without having to hop back into it and change things manually.

Lavender is written 100% in **[Rust](https://www.rust-lang.org/)** using the [axum](https://github.com/tokio-rs/axum) web framework, which is very modular and is one of the frameworks supported by [Shuttle](https://shuttle.rs), a Rust deployment tool similar to Vercel.

## Features
* Get any image from a specified directory on the server as a Base64 encoded string, useful for a constantly changing gallery of media. Less frequently modified images stay at the frontend of course.

* Get a determinate amount of the latest files (sorted alphabetically) that are in the media directory.

* Get the total amount of files in the media directory recursively.

* Configurable settings in a [`lavender.toml`](./lavender.toml) file, such as the specified directory where to look for media, file extension settings, and more to be added.

* A helper route that optimizes all of the images from a specified directory, it only works when providing the right API key though. This has to be configured by adding a `LAVENDER_API_HASH` environment variable when running the program or to the system's environment variables.