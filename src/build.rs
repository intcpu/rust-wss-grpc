fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        // .build_client(true)
        .out_dir("src/") // 输出目录
        .compile(
            &["src/protos/signal.proto"], // 替换成你的 proto 文件路径
            &["src/"],                    // 替换成你的 proto 文件包含目录路径
        )?;

    Ok(())
}
