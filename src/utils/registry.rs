use std::env::var;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::{
  PackageDef,
  utils::{
    distro::DistroInfo,
    package::{AppData, SupportedTarget},
  },
};

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

pub fn load_app_registry_by_id(app_id: String) -> Result<AppData, Box<dyn std::error::Error>> {
  let config_home = get_config_home()?;

  // loop through all the files in the app_registry/apps directory and find the app with the given id
  let app_registry_path = Path::new(&config_home).join("app_registry").join("apps");
  for entry in std::fs::read_dir(app_registry_path)? {
    let entry = entry?;
    let path = entry.path();

    if path.is_file() {
      let content = std::fs::read_to_string(&path)?;
      let apps: Vec<AppData> = serde_json::from_str(&content)?;
      for app in apps {
        if app.id == app_id {
          return Ok(app);
        }
      }
    }
  }

  Err(format!("App not found: {}", app_id).into())
}

pub fn resolve_package(
  package: &PackageDef,
  distro_info: &DistroInfo,
) -> Result<String, Option<Vec<String>>> {
  let app_data = match load_app_registry_by_id(package.package_id.clone()) {
    Ok(data) => data,
    Err(_) => return Err(None), // Return None if the app is not found
  };

  let target_id = match package.target_id.clone() {
    Some(id) => id,
    None => match distro_info.variant.as_ref() {
      Some(variant) => variant.to_target(),
      None => return Err(None), // Return None if the target is not specified and cannot be inferred
    },
  };

  // Clone targets to avoid multiple borrows of app_data.targets
  let targets = app_data.targets.clone();

  if let Some(target_package) = targets.get(&target_id) {
    Ok(target_package.clone())
  } else {
    // If the target is not supported, return an error with the list of supported targets
    let supported_targets: Vec<String> = targets
      .keys()
      .filter_map(|t| {
        // Return only the options relevant to the current distro variant, if available, using is_target_supported DistroInfo method
        if distro_info.is_target_supported(&Some(t.clone())) {
          Some(t.to_id().to_string())
        } else {
          None
        }
      })
      .collect::<Vec<String>>();

    Err(Some(supported_targets))
  }
}

// pub fn resolve_packages_by_id_and_target(
//   packages: Vec<String>,
//   target: &Option<SupportedTarget>,
// ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
//   let mut resolved_packages = Vec::new();

//   for package in packages {
//     let app_data = match load_app_registry_by_id(package.clone()) {
//       Ok(data) => data,
//       Err(_) => return Err(format!("App not found: {}", package).into()),
//     };

//     if let Some(target_package) = app_data.targets.get(target.as_ref().unwrap_or_e  (|| )) {
//       resolved_packages.push(target_package.clone());
//     } else {
//       return Err(
//         format!(
//           "The following distribution target ({:?}) is not supported for the TuxMate Package package: {}...\nPlease run `tuxmate info {}` to check the supported targets for this package.",
//           target, package, package
//         )
//         .into(),
//       );
//     }
//   }

//   Ok(resolved_packages)
// }

#[derive(serde::Deserialize, serde::Serialize)]
pub struct KnownPackages {
  #[serde(rename = "_comment", skip_serializing_if = "Option::is_none")]
  pub comment: Option<String>,
  pub packages: Vec<String>,
}

/// Load the known AUR packages from the local app registry, which do follow the AUR common suffix conventions. This is used to verify if a package is an AUR package or not.
pub fn load_known_aur_packages() -> Result<KnownPackages, Box<dyn std::error::Error>> {
  let config_home = get_config_home()?;
  let aur_packages_path = Path::new(&config_home)
    .join("app_registry")
    .join("aur-packages.json");

  if !aur_packages_path.exists() {
    return Err("AUR packages file not found. Please refresh the app registry.".into());
  }

  let content = std::fs::read_to_string(aur_packages_path)?;
  let known_aur_packages: KnownPackages = serde_json::from_str(&content)?;

  Ok(known_aur_packages)
}

/// Load the known unfree Nix packages from the local app registry. This is used to verify if a package is an unfree Nix package or not.
pub fn load_known_unfree_nix_packages() -> Result<KnownPackages, Box<dyn std::error::Error>> {
  let config_home = get_config_home()?;
  let unfree_nix_packages_path = Path::new(&config_home)
    .join("app_registry")
    .join("nix-unfree.json");

  if !unfree_nix_packages_path.exists() {
    return Err("Unfree Nix packages file not found. Please refresh the app registry.".into());
  }

  let content = std::fs::read_to_string(unfree_nix_packages_path)?;
  let known_unfree_nix_packages: KnownPackages = serde_json::from_str(&content)?;

  Ok(known_unfree_nix_packages)
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct VerifiedAppsMetadata {
  #[serde(rename = "fetchedAt", skip_serializing_if = "Option::is_none")]
  pub fetched_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct VerifiedApps {
  pub meta: Option<VerifiedAppsMetadata>,
  pub count: Option<u32>,
  pub apps: Vec<String>,
}

/// Load the verified Flathub packages from the local app registry. This is used to verify if a package is a verified Flathub package or not.
pub fn load_verified_flathub_packages() -> Result<VerifiedApps, Box<dyn std::error::Error>> {
  let config_home = get_config_home()?;
  let flathub_packages_path = Path::new(&config_home)
    .join("app_registry")
    .join("verified-flatpaks.json");

  if !flathub_packages_path.exists() {
    return Err(
      "Verified Flathub packages file not found. Please refresh the app registry.".into(),
    );
  }

  let content = std::fs::read_to_string(flathub_packages_path)?;
  let known_flathub_packages: VerifiedApps = serde_json::from_str(&content)?;

  Ok(known_flathub_packages)
}

/// Load the verified Snap packages from the local app registry. This is used to verify if a package is a verified Snap package or not.
pub fn load_verified_snap_packages() -> Result<VerifiedApps, Box<dyn std::error::Error>> {
  let config_home = get_config_home()?;
  let snap_packages_path = Path::new(&config_home)
    .join("app_registry")
    .join("verified-snaps.json");

  if !snap_packages_path.exists() {
    return Err("Verified Snap packages file not found. Please refresh the app registry.".into());
  }

  let content = std::fs::read_to_string(snap_packages_path)?;
  let known_snap_packages: VerifiedApps = serde_json::from_str(&content)?;

  Ok(known_snap_packages)
}
