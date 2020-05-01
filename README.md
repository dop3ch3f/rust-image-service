# Rust IMAGE microservice

- created with the hyper crate

# Run

- setup rust
- cargo run or cargo-watch -x "run"

# Features

- basic image upload and download

# Trying it out

In the first request, we'll download the rust logo and send to the upload route to get the id of the image

- \$ curl https://www.rust-lang.org/logos/rust-logo-128x128.png | curl -X POST --data-binary @- localhost:8080/upload

Then visit localhost:8080/download/<image-id> in your browser to view the file
