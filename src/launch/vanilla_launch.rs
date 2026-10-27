use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};
use std::process::{Child, Command, ExitStatus, Stdio};
use zip::ZipArchive;
use crate::auth::player::PlayerProfile;
use crate::launch::LAUNCHER_VERSION;
use crate::model::Allowed;
use crate::model::argument::{Argument, ArgumentValue};
use crate::model::vanilla_version::VanillaVersion;

pub struct VanillaLauncher {
    is_iso: bool,
    game_name: String,
    game_dir: PathBuf,
    command: Command,
    vanilla_version: VanillaVersion,
    width: Option<u32>,
    height: Option<u32>,
    profile: PlayerProfile,
    min_mb: usize,
    max_mb: usize,
}

impl VanillaLauncher {
    pub fn new(java_dir: &str, is_iso: bool) -> Self {
        let command = Command::new(java_dir);
        Self {
            is_iso,
            game_name: String::new(),
            game_dir: PathBuf::new(),
            command,
            vanilla_version: VanillaVersion::new(),
            width: None,
            height: None,
            profile: PlayerProfile::new(),
            min_mb: 2048,
            max_mb: 4096,
        }
    }

    pub fn max_mb(&mut self, max_mb: usize) -> &mut Self {
        self.max_mb = max_mb;
        self
    }

    pub fn min_mb(&mut self, min_mb: usize) -> &mut Self {
        self.min_mb = min_mb;
        self
    }

    pub fn profile(&mut self, profile: &PlayerProfile) -> &mut Self {
        self.profile = profile.clone();
        self
    }

    pub fn game_name(&mut self, game_name: &str) -> &mut Self {
        self.game_name = game_name.to_string();
        self
    }

    pub fn game_dir(&mut self, game_dir: &PathBuf) -> &mut Self {
        self.game_dir = game_dir.as_path().to_path_buf();
        self
    }

    pub fn arg(&mut self, string: String) -> &mut Self {
        self.command.arg(string);
        self
    }

    pub fn args(&mut self, vecs: Vec<String>) -> &mut Self {
        self.command.args(vecs);
        self
    }

    pub fn vanilla_version(&mut self, vanilla_version: &VanillaVersion) -> &mut Self {
        self.vanilla_version = vanilla_version.clone();
        self
    }

    pub fn run(&mut self) -> anyhow::Result<Child> {
        self.extract_natives()?;

        let args = self.build_arguments();
        self.command.args(args);
        if self.is_iso {
            self.command.current_dir(&self.game_dir.join("versions").join(self.game_name.as_str()));
        } else {
            self.command.current_dir(&self.game_dir);
        }
        let child = self.command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        Ok(child)
    }

    fn extract_natives(&self) -> anyhow::Result<()> {
        let libraries_dir = self.game_dir.join("libraries");
        let natives_dir = self.game_dir.join("versions").join(self.game_name.clone()).join("natives");
        if !natives_dir.exists() {
            fs::create_dir_all(&natives_dir)?;
        }

        if let Some(libs) = &self.vanilla_version.libraries {
            for lib in libs {
                if lib.allowed() && lib.name.contains("natives") {
                    extract_jar(
                        &libraries_dir.join(&lib.downloads.artifact.as_ref().unwrap().path),
                        &natives_dir,
                    )?;
                }
            }
            for lib in libs { 
                if lib.allowed() {
                    if let Some(native) = &lib.get_natives() {
                        if let Some(classifiers) = &lib.downloads.classifiers {
                            let di = &classifiers[native];
                            extract_jar(
                                &libraries_dir.join(&di.path),
                                &natives_dir,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn stop(mut child: Child) -> io::Result<ExitStatus> {
        child.kill()?;
        child.wait()
    }

    fn build_arguments(&self) -> Vec<String> {
        let mut vec= Vec::new();
        vec.push(format!("-Xms{}M", self.min_mb));
        vec.push(format!("-Xmx{}M", self.max_mb));
        if let Some(arguments) = &self.vanilla_version.arguments {
            if let Some(default) = &arguments.default_user_jvm {
                self.default_user_jvm(default, &mut vec);
            }
            self.jvm(&arguments.jvm, &mut vec);
            self.game(&arguments.game, &mut vec);
        }
        if let Some(arguments) = &self.vanilla_version.minecraft_arguments{
            vec.push("-cp".to_string());
            vec.push(self.classpath());
            if let Some(main) = &self.vanilla_version.main_class {
                vec.push(main.clone());
            }
            let splits = arguments.split(" ").collect::<Vec<&str>>();
            for split in splits {
                let text = self.replace_old(&split.to_string());
                vec.push(text);
            }
        }

        if !vec.iter().any(|argument| argument.starts_with("-Djava.library.path=")) {
            let natives_dir = self
                .game_dir
                .join("versions")
                .join(&self.game_name)
                .join("natives");
            vec.insert(
                2,
                format!("-Djava.library.path={}", natives_dir.to_string_lossy()),
            );
        }

        vec
    }

    fn game(&self, game: &Vec<Argument>, vec: &mut Vec<String>) {
        for argument in game {
            match argument {
                Argument::Plain(text) => {
                    if !self.profile.online {
                        if text.contains("xuid") {
                            continue;
                        } else if text.contains("clientid") {
                            continue;
                        } else if text.contains("clientId") {
                            continue;
                        }
                    }
                    vec.push(self.replace_game(&text.clone()))
                }
                Argument::Conditional(conditional) => {
                    if !conditional.allowed() {
                        continue;
                    }
                    match &conditional.value {
                        ArgumentValue::Single(single) => {
                            vec.push(self.replace_game(&single.clone()));
                        }
                        ArgumentValue::Multiple(multi) => {
                            for text in multi {
                                let string = self.replace_jvm(&text.clone());
                                vec.push(string);
                            }
                        }
                    }
                }
            }
        }
    }

    fn replace_old(&self, string: &String) -> String {
        let mut text = string.clone();
        let mut hash_map = HashMap::new();
        let mut game_dir = self.game_dir.to_string_lossy().to_string();
        if self.is_iso {
            game_dir = self.game_dir.join("versions").join(&self.game_name).to_string_lossy().to_string();
        }
        hash_map.insert("${auth_player_name}", self.profile.username.clone());
        hash_map.insert("${version_name}", self.vanilla_version.id.clone());
        hash_map.insert("${game_directory}", game_dir);
        hash_map.insert("${assets_root}", self.game_dir.join("assets").to_string_lossy().to_string());
        hash_map.insert("${assets_index_name}", self.vanilla_version.assets.as_ref().unwrap().clone());
        hash_map.insert("${auth_uuid}", self.profile.uuid.clone());
        hash_map.insert("${auth_access_token}", self.profile.access_token.clone());
        if self.profile.online {
            hash_map.insert("${user_type}", "msa".to_string());
        } else {
            hash_map.insert("${user_type}", "legacy".to_string());
        }
        hash_map.insert("${version_type}", format!("Launcher: TSMCL {}", LAUNCHER_VERSION));

        for (key, value) in hash_map {
            if string.contains(key) {
                text = string.replace(key, &value);
            }
        }

        text
    }

    fn jvm(&self, jvm: &Vec<Argument>, vec: &mut Vec<String>) {
        for argument in jvm {
            match argument {
                Argument::Plain(text) => {
                    let string = self.replace_jvm(&text.clone());
                    vec.push(string);
                    if text.eq("${classpath}") {
                        let main = self.vanilla_version.main_class.as_ref().unwrap();
                        vec.push(main.clone());
                    }
                }
                Argument::Conditional(conditional) => {
                    if !conditional.allowed() {
                        continue;
                    }
                    match &conditional.value {
                        ArgumentValue::Single(single) => {
                            let string = self.replace_jvm(&single.clone());
                            vec.push(string);
                        }
                        ArgumentValue::Multiple(multi) => {
                            for text in multi {
                                let string = self.replace_jvm(&text.clone());
                                vec.push(string);
                            }
                        }
                    }
                }
            }
        }
    }

    fn replace_game(&self, string: &String) -> String {
        let mut game_dir = self.game_dir.to_string_lossy().to_string();
        if self.is_iso {
            game_dir = self.game_dir.join("versions").join(&self.game_name).to_string_lossy().to_string();
        }
        let mut hash_map = HashMap::new();
        hash_map.insert("${auth_player_name}".to_string(), self.profile.username.to_string());
        hash_map.insert("${version_name}".to_string(), self.game_name.clone());
        hash_map.insert("${game_directory}".to_string(), game_dir);
        hash_map.insert("${assets_root}".to_string(), self.game_dir.join("assets").to_string_lossy().to_string());
        if let Some(assets) = self.vanilla_version.assets.clone() {
            hash_map.insert("${assets_index_name}".to_string(), assets);
        }
        hash_map.insert("${auth_uuid}".to_string(), self.profile.uuid.clone());
        hash_map.insert("${auth_access_token}".to_string(), self.profile.access_token.clone());
        hash_map.insert(
            "${user_type}".to_string(),
            if self.profile.online { "msa" } else { "legacy" }.to_string(),
        );
        if self.profile.online {
            if let Some(client_id) = self.profile.client_id.clone() {
                hash_map.insert("${clientid}".to_string(), client_id);
            }
            if let Some(x_uid) = self.profile.x_uid.clone() {
                hash_map.insert("${auth_xuid}".to_string(), x_uid);
            }
        }
        hash_map.insert("${version_type}".to_string(), format!("Minecraft: {}, Launcher: {}", self.vanilla_version.id, LAUNCHER_VERSION));
        if let Some(width) = self.width {
            hash_map.insert("${resolution_width}".to_string(), width.to_string());
        }
        if let Some(height) = self.height {
            hash_map.insert("${resolution_height}".to_string(), height.to_string());
        }

        let mut text= string.clone();

        for (k, v) in &hash_map {
            if !string.contains("$") {
                continue;
            }
            if string.eq(k) {
                text = string.replace(k, v);
            }
        }

        text
    }

    fn replace_jvm(&self, string: &String) -> String {
        let classpath = self.classpath();
        let natives = self.game_dir.join("versions").join(self.game_name.clone()).join("natives");
        let mut hash_map = HashMap::new();
        hash_map.insert("${natives_directory}".to_string(), natives.to_string_lossy().to_string());
        hash_map.insert("${launcher_name}".to_string(), "Teishou's Minecraft Launcher".to_string());
        hash_map.insert("${launcher_version}".to_string(), LAUNCHER_VERSION.to_string());
        hash_map.insert("${classpath}".to_string(), classpath);

        let mut text= string.clone();

        if string.contains("${natives_directory}") {
            text = string.replace("${natives_directory}", &*hash_map["${natives_directory}"]);
        } else if string.contains("${launcher_name}") {
            text = string.replace("${launcher_name}", &*hash_map["${launcher_name}"]);
        } else if string.contains("${launcher_version}") {
            text = string.replace("${launcher_version}", &*hash_map["${launcher_version}"]);
        } else if string.contains("${classpath}") {
            text = string.replace("${classpath}", &*hash_map["${classpath}"]);
        }

        text
    }

    fn default_user_jvm(&self, default_user_jvm: &Vec<Argument>, vec: &mut Vec<String>) {
        for argument in default_user_jvm {
            match argument {
                Argument::Plain(text) => vec.push(text.clone()),
                Argument::Conditional(conditional) => {
                    if !conditional.allowed() {
                        continue;
                    }
                    match &conditional.value {
                        ArgumentValue::Single(text) => vec.push(text.clone()),
                        ArgumentValue::Multiple(multi) => {
                            vec.extend(multi.clone());
                        }
                    }
                }
            }
        }
    }

    fn classpath(&self) -> String {
        let version_dir = self.game_dir.join("versions").join(self.game_name.clone());
        let libraries_dir = self.game_dir.join("libraries");

        let mut class = vec![];

        if let Some(libraries) = &self.vanilla_version.libraries {
            for library in libraries {
                if !library.allowed() {
                    continue;
                }
                if let Some(classifiers) = &library.downloads.classifiers {
                    if let Some(native) = &library.get_natives() {
                        if let Some(item) = classifiers.get(native) {
                            let lib_path = libraries_dir.join(&item.path).to_string_lossy().to_string();
                            class.push(lib_path);
                        }
                    }
                }
                if let Some(artifact) = &library.downloads.artifact {
                    let lib_path = libraries_dir.join(&artifact.path).to_string_lossy().to_string();
                    class.push(lib_path);
                }
            }
        }

        let game_path = version_dir.join(format!("{}.jar", self.game_name));

        class.push(game_path.to_string_lossy().to_string());

        if cfg!(target_os = "windows") { class.join(";") } else { class.join(":") }
    }
}

fn extract_jar(jar: &Path, dir: &Path) -> anyhow::Result<()> {
    let mut archive = ZipArchive::new(fs::File::open(jar)?)?;
    fs::create_dir_all(dir)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.name()?.starts_with("META-INF/") { continue; }

        // enclosed_name 会拒绝 "../" 和绝对路径，防止解压到目录外面（Zip Slip）
        let Some(rel) = entry.enclosed_name() else {
            continue;
        };
        let out = dir.join(rel);

        if entry.is_dir() {
            fs::create_dir_all(&out)?;
            continue;
        }

        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(&out)?;
        io::copy(&mut entry, &mut file)?;

        // Unix 上保留可执行权限等
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&out, fs::Permissions::from_mode(mode))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;
    use crate::model::library::{Download, DownloadItem, Library};

    #[test]
    fn offline_profile_uses_legacy_user_type() {
        let profile = PlayerProfile::offline("Player".to_string());
        let mut launcher = VanillaLauncher::new("java", true);
        launcher.profile(&profile);

        assert_eq!(
            launcher.replace_game(&"${user_type}".to_string()),
            "legacy"
        );
    }

    #[test]
    fn legacy_arguments_include_natives_directory() {
        let mut version = VanillaVersion::new();
        version.minecraft_arguments = Some("--username Player".to_string());
        version.assets = Some("1.12.2".to_string());

        let game_dir = PathBuf::from("game");
        let mut launcher = VanillaLauncher::new("java", true);
        launcher
            .game_name("1.12.2")
            .game_dir(&game_dir)
            .vanilla_version(&version);

        let expected = format!(
            "-Djava.library.path={}",
            game_dir
                .join("versions")
                .join("1.12.2")
                .join("natives")
                .to_string_lossy()
        );
        assert!(launcher.build_arguments().contains(&expected));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn extracts_existing_native_classifier_archive() -> anyhow::Result<()> {
        let root = std::env::temp_dir().join(format!(
            "tsmcl-native-extraction-{}",
            std::process::id()
        ));
        let libraries_dir = root.join("libraries");
        let archive_path = libraries_dir.join("example").join("native.jar");
        fs::create_dir_all(archive_path.parent().unwrap())?;

        let file = fs::File::create(&archive_path)?;
        let mut archive = zip::ZipWriter::new(file);
        archive.start_file(
            "lwjgl64.dll",
            zip::write::SimpleFileOptions::default(),
        )?;
        archive.write_all(b"native dll fixture")?;
        archive.finish()?;

        let library = Library {
            downloads: Download {
                artifact: None,
                classifiers: Some(HashMap::from([(
                    "natives-windows".to_string(),
                    DownloadItem {
                        path: "example/native.jar".to_string(),
                        sha1: String::new(),
                        size: 0,
                        url: String::new(),
                    },
                )])),
            },
            name: "example:platform:1.0".to_string(),
            rules: None,
            natives: Some(HashMap::from([(
                "windows".to_string(),
                "natives-windows".to_string(),
            )])),
            extract: None,
        };
        let mut version = VanillaVersion::new();
        version.libraries = Some(vec![library]);
        let mut launcher = VanillaLauncher::new("java", true);
        launcher
            .game_name("test")
            .game_dir(&root)
            .vanilla_version(&version);

        launcher.extract_natives()?;

        let extracted_dll = root
            .join("versions")
            .join("test")
            .join("natives")
            .join("lwjgl64.dll");
        assert!(extracted_dll.is_file());
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
