use std::io;

fn main() -> io::Result<()> {
    tonic_build::configure()
        .format(true)
        .out_dir("src/proto")
        .compile(&["q.proto", "restaurant.proto"], &["proto"])?;
    Ok(())
}
