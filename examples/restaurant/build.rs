use std::io;

fn main() -> io::Result<()> {
    tonic_build::configure()
        .format(true)
        .out_dir("src/proto")
        //.include_file("mod.rs")
        .compile(&["q.proto", "example_service.proto"], &["proto"])?;
    Ok(())
}
