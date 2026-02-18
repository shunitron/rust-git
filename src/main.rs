use sha1::{Digest, Sha1};
use std::{env, fs};

enum GitObject {
    Blob(Blob),
}

impl GitObject {
    fn print_type(&self) {
        match self {
            GitObject::Blob(_) => println!("これはファイル(Blob)です"),
        }
    }
}

struct Blob {
    filename: String,
    content: String,
    hash: String,
}

impl Blob {
    fn new(filename: &str, content: &str) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(content.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        Self {
            filename: filename.to_string(),
            content: content.to_string(),
            hash,
        }
    }

    fn display(&self) {
        println!("--- Blob Info ---");
        println!("File: {}", self.filename);
        println!("Hash: {}", self.hash);
        println!("Size: {} bytes", self.content.len());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("使用法: cargo run -- <ファイル名>");
        return Ok(());
    }

    let filename = &args[1];

    // 1. ファイル読み込みに失敗したら即エラーを返して終了
    let content = fs::read_to_string(filename)?;

    let blob = Blob::new(filename, &content);
    let obj = GitObject::Blob(blob);

    if let GitObject::Blob(inner_blob) = obj {
        println!("ファイル名: {}", inner_blob.filename);
        println!("ハッシュ: {}", inner_blob.hash);

        save_object(&inner_blob.hash, &inner_blob.content)?;
        println!("保存成功！");
    }

    Ok(())
}

fn save_object(hash: &str, content: &str) -> std::io::Result<()> {
    let (dir_name, file_name) = hash.split_at(2);
    let dir_path = format!(".mygit/objects/{}", dir_name);
    let file_path = format!("{}/{}", dir_path, file_name);

    fs::create_dir_all(&dir_path)?;
    fs::write(file_path, content)?;
    Ok(())
}
