use testcontainers::GenericImage;

fn main() {
    let client = testcontainers::clients::Cli::default();
    let image = GenericImage::new("hello-world", "latest");
    let container = client.run(image);
}
