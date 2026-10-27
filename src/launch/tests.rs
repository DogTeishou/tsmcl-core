use anyhow::bail;
use crate::auth::player::PlayerProfile;
use crate::launch::VanillaLauncher;
use crate::model::vanilla_version::VanillaVersion;
use crate::parse::Parse;

fn launch_version(version: &str, java_dir: &str) -> anyhow::Result<()> {
    let game_dir = std::env::current_dir()?.join(".minecraft");
    let config_path = game_dir.join("versions").join(version).join(format!("{}.json", version));
    if !config_path.exists() {
        bail!("config doesn't exist");
    }
    let text = std::fs::read_to_string(config_path)?;
    let vanilla_version = VanillaVersion::parse(&text)?;
    let profile = PlayerProfile::offline("DogTeishou".to_string());
    let child = &mut VanillaLauncher::new(java_dir, true)
        .game_name(version)
        .game_dir(&game_dir)
        .profile(&profile)
        .vanilla_version(&vanilla_version).run()?;
    let status = child.wait()?;
    if !status.success() {
        bail!("Minecraft process exited with status {status}");
    }
    Ok(())
}

#[test]
fn launcher_tests_26_3() -> anyhow::Result<()> {
    launch_version("26.3", "C:\\Program Files\\Zulu\\zulu-25\\bin\\java.exe")
}

#[test]
fn launcher_tests_1_18() -> anyhow::Result<()> {
    launch_version("1.18", "C:\\Program Files\\Zulu\\zulu-25\\bin\\java.exe")
}

#[test]
fn launcher_tests_1_12_2() -> anyhow::Result<()> {
    launch_version("1.12.2", "C:\\Program Files\\Zulu\\zulu-8\\bin\\java.exe")
}

#[test]
fn launcher_tests_rd_132211() -> anyhow::Result<()> {
    launch_version("rd-132211", "C:\\Program Files\\Zulu\\zulu-8\\bin\\java.exe")
}

#[test]
fn test_get_native() -> anyhow::Result<()> {
    let file_path = std::env::current_dir()?.join(".minecraft").join("versions").join("1.18").join("1.18.json");
    let text = std::fs::read_to_string(file_path)?;
    let v = VanillaVersion::parse(&text)?;
    if let Some(libs) = v.libraries {
        for lib in libs {
            if let Some(native) = lib.get_natives() {
                println!("{}", native);
                if let Some(class) = &lib.downloads.classifiers {
                   let download = &class[&native];
                    println!("{}", download.path);
                    println!("{}", download.url);
                    println!("{}", download.sha1);
                    println!("{}", download.size);
               }
            }
        }
    }

    Ok(())
}