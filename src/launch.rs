use std::ffi::OsStr;
use std::iter::zip;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::download::{get, Source};
use crate::download::library::Allowed;
use crate::model;
use crate::model::version::Version;
use crate::parse::Parse;
use tokio::process::Command;

pub struct Launcher {
    pub game_dir: String,
    pub version: String,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub main_class: String,
    pub args: Vec<String>,
    pub cmd: Command,
}

impl Launcher {
    pub fn new(java_path: String) -> Self {
        let args = vec![];
        Self {
            game_dir: "".to_string(),
            version: "".to_string(),
            username: "".to_string(),
            uuid: "".to_string(),
            access_token: "".to_string(),
            main_class: "".to_string(),
            cmd: Command::new(java_path),
            args,
        }
    }

    pub fn game_dir(&mut self, path: &str) -> &mut Self {
        self.game_dir = path.to_string();
        self
    }

    pub fn version(&mut self, version: &str) -> &mut Self {
        self.version = version.to_string();
        self
    }

    pub fn username(&mut self, username: &str) -> &mut Self {
        self.username = username.to_string();
        self
    }

    pub fn uuid(&mut self, uuid: &str) -> &mut Self {
        self.uuid = uuid.to_string();
        self
    }

    pub fn access_token(&mut self, access_token: &str) -> &mut Self {
        self.access_token = access_token.to_string();
        self
    }

    pub fn main_class(&mut self, main_class: &str) -> &mut Self {
        self.main_class = main_class.to_string();
        self
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.cmd.arg(arg);
        self
    }

    pub fn args(&mut self, args: Vec<String>) -> &mut Self {
        self.args = args;
        self
    }

    pub fn build(&mut self) -> anyhow::Result<&mut Self> {
        println!("Building Minecraft {}...", self.version);
        let version = self.version.clone();
        let game_path = self.game_dir.clone();
        let game_dir = Path::new(game_path.as_str());
        let libraries_dir = game_dir.join("libraries");
        let assets_dir = game_dir.join("assets");
        let version_dir = game_dir.join("versions").join(&version);
        let natives_dir = version_dir.join("natives");
        let config_path = version_dir.join(format!("{}.json", &version));
        let version_path = version_dir.join(format!("{}.jar", &version));

        let access_token = self.access_token.clone();
        let username = self.username.clone();
        let uuid = self.uuid.clone();

        if !version_path.exists() || !config_path.exists() {
            return Err(anyhow::anyhow!("Version:{} not found", self.version));
        }

        let string = std::fs::read_to_string(&config_path).unwrap();

        let version = &Version::parse(string.as_str())?;

        if !natives_dir.exists() {
            std::fs::create_dir_all(&natives_dir)?;
        }

        for library in &version.libraries {
            if library.allowed() && library.name.contains("natives") {
                extract_jar(&libraries_dir.join(&library.downloads.artifact.path), &natives_dir)?;
            }
        }

        let classpath = format!(
            "{}{}",
            &version
                .libraries
                .iter()
                .map(|library| {
                    format!(
                        "{}{}",
                        libraries_dir
                            .join(&library.downloads.artifact.path)
                            .display(),
                        if cfg!(windows) { ";" } else { ":" }
                    )
                })
                .collect::<String>(),
            version_path.display()
        );
        self
            .arg("-Xmx4G")
            .arg("-Dstderr.encoding=UTF-8")
            .arg("-Dfile.encoding=UTF-8")
            .arg("-Dstdout.encoding=UTF-8")
            .arg("-XX:+UseG1GC")
            .arg(format!("-Djava.library.path={}", natives_dir.display()))
            .arg("-cp")
            .arg(classpath)
            .arg(&version.main_class)
            .arg("--username")
            .arg(username)
            .arg("--uuid")
            .arg(uuid)
            .arg("--version")
            .arg(&version.id)
            .arg("--gameDir")
            .arg(game_dir)
            .arg("--assetsDir")
            .arg(assets_dir)
            .arg("--assetIndex")
            .arg(&version.asset_index.id)
            .arg("--accessToken")
            .arg(access_token)
            .arg("--versionType")
            .arg("TSMCL Alpha 0.0.1");

        Ok(self)
    }

    pub async fn launch(&mut self) -> anyhow::Result<()> {
        println!("Launching Minecraft {}...", self.version);
        let cmd = &mut self.cmd; // Set the executable or use self.main_class if needed
        cmd.args(&self.args)
            .stdout(Stdio::inherit())
            .status().await?;
        Ok(())
    }
}

pub async fn get_version_manifest(source: &Source) -> anyhow::Result<model::version_manifest::VersionManifest> {
    let url = match source {
        Source::Official => "https://piston-meta.mojang.com/mc/game/version_manifest.json",
        Source::BMCLAPI => "https://bmclapi2.bangbang93.com/mc/game/version_manifest_v2.json",
    };
    let text = get(url).await?.text().await.map_err(|err| anyhow::anyhow!("{:?}", err))?;
    Ok(model::version_manifest::VersionManifest::parse(&text)?)
}

fn extract_jar(lib: &Path, dir: &Path) -> anyhow::Result<()> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(lib)?)?;
    std::fs::create_dir_all(dir)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.is_file() && !file.name()?.contains("META-INF") {
            let outpath = match file.enclosed_name() {
                Some(path) => Path::new(dir).join(path),
                None => continue,
            };

            if file.name()?.ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::auth::generate_offline_uuid;
    use super::*;

    #[tokio::test]
    async fn run_test() -> anyhow::Result<()> {
        let uuid = generate_offline_uuid("test".to_string());
        let mut launcher = Launcher::new("C:\\Program Files\\Zulu\\zulu-25\\bin\\java.exe".to_string());
            launcher.game_dir(std::env::current_dir()?.join("rust-minecraft-client-launch").to_str().unwrap())
            .version("1.21")
            .username("test")
            .uuid(uuid.as_str())
            .access_token("0")
            .main_class("net.minecraft.client.main.Main")
            .build()?;
        launcher.launch().await?;
        Ok(())
    }
}