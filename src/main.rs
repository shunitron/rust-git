use sha1::{Digest, Sha1};
use std::{env, fs, time::SystemTime};

trait Serializable {
    fn serialize(&self) -> String;

    fn calculate_hash(&self) -> String {
        let content = self.serialize();
        let mut hasher = Sha1::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Clone)]
struct Author {
    name: String,
    email: String,
}

#[derive(Debug)]
struct Commit {
    tree_hash: String,           // このコミットが指すTreeのハッシュ
    parent_hash: Option<String>, // 親コミット(初回コミットは None)
    author: Author,
    message: String,
    timestamp: u64, // UNIXタイムスタンプ
}

impl Commit {
    fn new(
        tree_hash: String,
        parent_hash: Option<String>,
        author: Author,
        message: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            tree_hash,
            parent_hash,
            author,
            message,
            timestamp,
        }
    }
}

impl Serializable for Commit {
    fn serialize(&self) -> String {
        let mut result = String::new();

        result.push_str(&format!("tree {}\n", self.tree_hash));

        // 親ハッシュが存在する場合のみ
        if let Some(parent) = &self.parent_hash {
            result.push_str(&format!("parent {}\n", parent));
        }

        result.push_str(&format!(
            "author {} <{}> {}\n",
            self.author.name, self.author.email, self.timestamp
        ));

        result.push_str("\n");
        result.push_str(&self.message);

        result
    }
}

#[derive(Debug, Clone)]
enum FileMode {
    Regular,    // 100644
    Executable, // 100755
    Directory,  // 040000
}

impl FileMode {
    fn as_str(&self) -> &str {
        match self {
            FileMode::Regular => "100644",
            FileMode::Executable => "100755",
            FileMode::Directory => "040000",
        }
    }
}

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

#[derive(Debug)]
enum EntryContent {
    BlobHash(String),   // ファイルの場合はハッシュ値
    SubTree(Box<Tree>), // ディレクトリの場合は「別のTree」をBoxに入れて持つ
}

#[derive(Debug)]
struct TreeEntry {
    mode: FileMode, // ファイルモード
    name: String,   // ファイル名
    // hash: String,   // そのファイルの中身(Blob)のSHA-1
    content: EntryContent, // hash(String)の代わりにEnumを使う
}

#[derive(Debug)]
struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn add_entry(&mut self, entry: TreeEntry) {
        self.entries.push(entry);
    }
}

impl Serializable for Tree {
    fn serialize(&self) -> String {
        let mut content = String::new();
        for entry in &self.entries {
            let hash = match &entry.content {
                EntryContent::BlobHash(h) => h.clone(),
                EntryContent::SubTree(t) => t.calculate_hash(),
            };

            content.push_str(&format!(
                "{} {} {}\n",
                entry.mode.as_str(),
                entry.name,
                hash
            ));
        }
        content
    }

    fn calculate_hash(&self) -> String {
        let content = self.serialize();
        let mut hasher = Sha1::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
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
}

impl Serializable for Blob {
    fn serialize(&self) -> String {
        self.content.clone()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(".mygit/objects")?;

    let sekine = Author {
        name: "sekine".to_string(),
        email: "sekine@example.com".to_string(),
    };

    let blob1 = Blob::new("hello.txt", "Hello Rust content");
    save(&blob1)?;

    let mut tree1 = Tree::new();
    tree1.add_entry(TreeEntry {
        mode: FileMode::Regular,
        name: blob1.filename.clone(),
        content: EntryContent::BlobHash(blob1.hash.clone()),
    });
    let tree1_hash = save(&tree1)?;

    let commit1 = Commit::new(tree1_hash, None, sekine.clone(), "First Commit".to_string());
    let commit1_hash = save(&commit1)?;
    println!("Commit 1 saved: {}", commit1_hash);

    update_head(&commit1_hash)?;
    println!("Commit 1 saved and HEAD updated: {}", commit1_hash);

    let blob2 = Blob::new("notes.txt", "Learning Rust is exciting!");
    save(&blob2)?;

    let mut tree2 = Tree::new();
    tree2.add_entry(TreeEntry {
        mode: FileMode::Regular,
        name: blob2.filename.clone(),
        content: EntryContent::BlobHash(blob2.hash.clone()),
    });
    let tree2_hash = save(&tree2)?;

    let commit2 = Commit::new(
        tree2_hash,
        Some(commit1_hash),
        sekine.clone(),
        "Second commit: Added notes.txt".to_string(),
    );
    let commit2_hash = save(&commit2)?;
    update_head(&commit2_hash)?;
    println!("Commit 2 saved and HEAD updated: {}", commit2_hash);

    return Ok(());

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

fn save<T: Serializable>(obj: &T) -> std::io::Result<String> {
    let content = obj.serialize();
    let hash = obj.calculate_hash();
    save_object(&hash, &content)?;
    Ok(hash)
}

fn update_head(commit_hash: &str) -> std::io::Result<()> {
    let head_path = ".mygit/HEAD";
    fs::write(head_path, commit_hash)?;
    Ok(())
}
