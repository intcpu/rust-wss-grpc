fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        // .build_server(true)
        // .build_client(true)
        // .out_dir("src/") // 输出目录
        .compile(
            &["proto/helloworld.proto"], // helloworld 文件路径
            &["."],                      // helloworld 文件包含目录路径
        )?;
    Ok(())
}
