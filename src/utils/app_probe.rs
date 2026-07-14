use crate::utils::registry::{
  load_known_aur_packages, load_verified_flathub_packages, load_verified_snap_packages,
};

const AUR_SUFFIXES: [&str; 3] = ["-git", "-bin", "-appimage"];

/// Verify if the package is an AUR package or not
pub fn is_aur_package(package_name: &str) -> bool {
  let known_aur_packages = match load_known_aur_packages() {
    Ok(packages) => packages.packages,
    Err(_) => return false, // If we can't load the known AUR packages, assume it's not an AUR package
  };

  // Check if the package name ends with any of the AUR suffixes
  for suffix in AUR_SUFFIXES {
    if package_name.ends_with(suffix) {
      return true;
    }
  }

  // If the package name is in the list of known AUR packages, it's an AUR package
  known_aur_packages.contains(&package_name.to_string())
}

/// Verify if the package is an unfree Nix package or not
pub fn is_unfree_nix_package(package_name: &str) -> bool {
  let known_unfree_nix_packages = match crate::utils::registry::load_known_unfree_nix_packages() {
    Ok(packages) => packages.packages,
    Err(_) => return false, // If we can't load the known unfree Nix packages, assume it's not an unfree Nix package
  };

  let clean_pkg = package_name.trim().to_lowercase();

  // If the package name is in the list of known unfree Nix packages, it's an unfree Nix package
  if known_unfree_nix_packages.contains(&clean_pkg) {
    return true;
  }

  // Check if the package name contains any of the known unfree Nix packages as a substring
  for unfree_pkg in known_unfree_nix_packages {
    if clean_pkg.contains(&unfree_pkg) {
      return true;
    }
  }

  false
}

/// Verify if the Flathub package is verified or not.
pub fn is_flathub_verified(package_name: &str) -> bool {
  let verified_flathub_packages = match load_verified_flathub_packages() {
    Ok(packages) => packages.apps,
    Err(_) => return false, // If we can't load the verified Flathub packages, assume it's not a verified Flathub package
  };

  // If the package name is in the list of verified Flathub packages, it's a verified Flathub package
  verified_flathub_packages.contains(&package_name.to_string())
}

/// Verify if the Snap package is verified or not.
pub fn is_snap_verified(package_name: &str) -> bool {
  let verified_snap_packages = match load_verified_snap_packages() {
    Ok(packages) => packages.apps,
    Err(_) => return false, // If we can't load the verified Snap packages, assume it's not a verified Snap package
  };

  let clean_pkg = package_name.split(" ").next().unwrap_or("");

  // If the package name is in the list of verified Snap packages, it's a verified Snap package
  verified_snap_packages.contains(&clean_pkg.to_string())
}
