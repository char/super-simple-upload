# super-simple-upload

A tiny HTTP application to allow uploading of arbitrary files.

- Uploads are gated by a key check (via the Authorization header).
- Keys are checked against `keys.json` in the current working directory.
- Uploaded files are randomly named and placed into the `./uploads` folder.

`super-simple-upload` does **NOT** serve the uploaded files. Please use a real web server: this means you'll get proper support for things like `Range` requests.

## Usage

1. Compile the binary: `cargo build --release`
2. Create a directory to hold the keys and upload folder
3. Populate `keys.json`

```json
{
    "Bearer MY_KEY_THAT_SHOULD_PROBABLY_BE_RANDOMLY_GENERATED": "Some User"
}
```
4. Create the `uploads` folder
5. Run `super-simple-upload`. You can use the `PORT` environment variable.

### With nginx

If you're using nginx, you could set up an upload endpoint like so:

```
location /u {
    proxy_pass http://super-simple-upload:8080;
}

location ^~ /u/ {
    alias /path/to/super-simple-upload/uploads/;
}
```

Then, a POST to /u will upload a file, and you can access the files through the `/u/<file>` endpoint.