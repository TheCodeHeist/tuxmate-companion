use std::env::var;
use std::path::Path;

// This handles all the application registry related stuff, like fetching the list of verified apps etc., all from the main TuxMate GitHub repository.
const APP_REGISTRY_DIR_URL: &str =
  "https://raw.githubusercontent.com/abusoww/tuxmate/refs/heads/main/src/lib/";

const APP_FILES: [&str; 20] = [
  "apps/ai-tools.json",
  "apps/cli-tools.json",
  "apps/communication.json",
  "apps/creative.json",
  "apps/dev-editors.json",
  "apps/dev-languages.json",
  "apps/dev-tools.json",
  "apps/file-sharing.json",
  "apps/gaming.json",
  "apps/media.json",
  "apps/office.json",
  "apps/security.json",
  "apps/system.json",
  "apps/terminal.json",
  "apps/vpn-network.json",
  "apps/web-browsers.json",
  "aur-packages.json",
  "nix-unfree.json",
  "verified-flatpaks.json",
  "verified-snaps.json",
];

fn get_config_home() -> Result<String, Box<dyn std::error::Error>> {
  match var("XDG_CONFIG_HOME") {
    Ok(dir) => Ok(format!("{}/tuxmate", dir)),
    Err(_) => match var("HOME") {
      Ok(home) => Ok(format!("{}/.config/tuxmate", home)),
      Err(_) => Err("Could not determine config home".into()),
    },
  }
}

fn create_config_dir_if_not_exists() -> Result<(), Box<dyn std::error::Error>> {
  let config_home = get_config_home()?;
  let path = Path::new(&config_home);
  if !path.exists() {
    std::fs::create_dir_all(path)?;
  }

  // Create folder for app registry files
  let app_registry_path = path.join("app_registry/apps");
  if !app_registry_path.exists() {
    std::fs::create_dir_all(app_registry_path)?;
  }

  Ok(())
}

fn get_app_registry_urls() -> Vec<String> {
  APP_FILES
    .iter()
    .map(|file| format!("{}{}", APP_REGISTRY_DIR_URL, file))
    .collect()
}

pub fn refresh_app_registry() -> Result<(), Box<dyn std::error::Error>> {
  create_config_dir_if_not_exists()?;

  let rt = tokio::runtime::Runtime::new()?;
  let urls = get_app_registry_urls();
  for url in urls {
    let (status, content) = rt.block_on(async {
      let resp = reqwest::get(&url).await?;
      let status = resp.status();
      let text = resp.text().await?;
      Ok::<(_, _), reqwest::Error>((status, text))
    })?;

    if status.is_success() {
      let config_home = get_config_home()?;
      let file_path = Path::new(&config_home)
        .join("app_registry")
        .join(url.replace(APP_REGISTRY_DIR_URL, ""));

      std::fs::write(file_path, content)?;
    } else {
      eprintln!("Failed to fetch {}: {}", url, status);
    }
  }
  Ok(())
}
