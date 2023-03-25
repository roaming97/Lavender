# Lavender

The backend for my [website](https://roaming97.com). It serves as a quick gateway to an image server I have while being flexible and letting me automate tasks without having to hop back into the Linux server and change things manually.

Lavender is written 100% in **[Rust](https://www.rust-lang.org/)** using the [Rocket](https://rocket.rs/) web framework, which is very reminiscent of Flask for Python.

## Features
* Get any image from a specified directory on the server as a Base64 encoded string, useful for a constantly changing gallery of media. Static and often requested images stay at the frontend of course.

* Configurable settings in a [`lavender.toml`](./lavender.toml) file, such as the specified directory where to look for media, extension settings, and more to be added.

* A helper route that optimizes all of the images from a specified directory, it only works when providing the right API key though. (This has to be configured by adding a `LAVENDER_API_HASH` environment variable when running the program or to the system's environment variables.)