# Lavender

Lavender is a quick file server while being flexible and letting me automate tasks without having to hop back into the server or do too many manual fetch requests. Originally used as the backend for my [website](https://roaming97.com/) before a server migration.

Lavender is written 100% in **[Rust](https://www.rust-lang.org/)** using the [axum](https://github.com/tokio-rs/axum) web framework, which is very modular when handling data and writing tests (with additional help from [axum_test](https://github.com/JosephLenton/axum-test)).

## Features

- A secure API that only works via a key, restricting who can make requests to the backend.

- Get any image from a specified directory on the server as a Base64 encoded string, useful for a constantly changing gallery of media. More static images stay at the frontend of course.

- Get a determinate amount of the latest files (sorted by last modified date) that are in the media directory.

- Scan the media directory and get the total amount of files, this scan can be either recursive or not.

- Configurable settings in a [`lavender.toml`](./lavender.toml) file, currently only meant for server configuration, additional fields may be added in the future.

## Preparing

Clone the repository in your server:

```shell
$ git clone https://github.com/roaming97/Lavender.git
```

Add a folder with your images. As an example, the file structure could be:

```
├───artwork
  └───thumbnails
├───photo
│   └───thumbnails
├───test_dir
└───video
    └───thumbnails
```

## Testing

It is recommended to run the unit tests in [tests.rs](./src/tests.rs) before building. In case your file structure looks different from the one on the example, you might have to change the tests to locate the directories in your file structure correctly. You can do that by changing the relative path in tests like `get_single_file()`:

```rs
#[tokio::test]
async fn get_single_file() {
    let query = GetFileParams { // <-- Looking at the type definition of `GetFileParams` may help.
        path: "./artwork/thumbnails/day1.webp".into(), // <-- Change this to your desired path.
    };
    let response = test("/file", query, TEST_API_KEY).await;
    // rest of test code
    // [...]
}
```

Once verified that, you can run the tests using:

```shell
$ cargo test --release
```

The testing server will use an example API key, requiring no variables when running the test command. Optionally, you can remove the `--release` flag if you prefer to test in debug mode.

### Test debugging

If you wish to debug testing errors more in detail, you can check the output from them using this command:

```shell
$ cargo test -- --show-output
```

## Building

### API Key Setup

The way the frontend speaks to the backend is by providing an API key as one of the headers, named `lav-api-key` on the frontend and a password hash on the backend. This key is then tested against whatever the frontend sent.

**PASSWORD EXAMPLE**: Let's say I want to use `AL4vend3rBl00mSFr0MTh3S01l` as my password. If you want Lavender to respond correctly to this password being passed by your request headers, you need to tell Lavender what the hash for that password is.

That password after being hashed using the SHA3-256 algorithm would be `4e655e82c64e3e1d0ba853fa69fa599e6a3611fd26a6a977423e0f0c4d5fd542`, which needs to be passed as an environment variable to the Lavender binary, you can generate a hash for your own passwords using an [online hasher](https://emn178.github.io/online-tools/sha3_256.html) for example.

This can be done either by exporting the variable with the value of the hash to the system's environment...

**Unix**

```shell
$ export LAVENDER_API_HASH="4e655e82c64e3e1d0ba853fa69fa599e6a3611fd26a6a977423e0f0c4d5fd542"
$ echo $LAVENDER_API_HASH
```

**Windows (PowerShell)**

```powershell
> $Env:LAVENDER_API_HASH = "4e655e82c64e3e1d0ba853fa69fa599e6a3611fd26a6a977423e0f0c4d5fd542"
> $Env:LAVENDER_API_HASH
```

...or adding the variable alongside the run command: `LAVENDER_API_HASH="4e655e82c64e3e1d0ba853fa69fa599e6a3611fd26a6a977423e0f0c4d5fd542" cargo run --release`

---

Once that's done, you can build/run a Lavender binary using the usual `cargo` command(s):

```bash
$ cargo build --release
$ cargo run --release
```

## Contributing

This is meant to be a personal tool more than a standalone crate, it doesn't strive to be the most performant, just convenient for my use case. Pull requests that modify the codebase substantially will likely not be merged into this repository but feel free to fork it and make your own version!
