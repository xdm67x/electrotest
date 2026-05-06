//! Application launcher for Electron processes
//! Handles starting Electron with remote debugging, waiting for CDP to be available,
//! and gracefully shutting down the process.

use anyhow::{Result, anyhow, bail};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Maximum number of parent directories to search for node_modules
const MAX_PARENT_LEVELS: usize = 5;

/// Maximum number of ports to try when auto-incrementing
const MAX_PORT_ATTEMPTS: u16 = 10;

/// Timeout for waiting on CDP to become available
const CDP_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

/// Detect Electron executable path from a project directory.
/// Searches for node_modules/.bin/electron (Unix) or node_modules/.bin/electron.cmd (Windows).
/// Walks up parent directories (max 5 levels) for monorepo support.
/// Returns the first valid executable path found, or None if not found.
pub fn detect_electron_path(start_path: &Path) -> Option<PathBuf> {
    // Determine the search root: if start_path is a file, use its parent directory
    let search_root = if start_path.is_file() {
        start_path.parent()?
    } else {
        start_path
    };

    // Check current directory and up to MAX_PARENT_LEVELS parent levels
    let mut current = search_root.to_path_buf();
    for _ in 0..=MAX_PARENT_LEVELS {
        // Check Unix executable
        let bin_electron = current.join("node_modules/.bin/electron");
        if bin_electron.exists() {
            return Some(bin_electron);
        }

        // Check Windows batch file
        let bin_electron_cmd = current.join("node_modules/.bin/electron.cmd");
        if bin_electron_cmd.exists() {
            return Some(bin_electron_cmd);
        }

        // Move up one level
        if !current.pop() {
            break;
        }
    }
    None
}


/// Launches and manages an Electron application process
pub struct AppLauncher {
    child: Child,
    port: u16,
}

impl AppLauncher {
    /// Trouve un port CDP disponible en auto-incrémentant depuis base_port
    /// Un port est considéré disponible si la connexion TCP échoue (port libre)
    pub async fn find_available_port(base_port: u16) -> Result<u16> {
        use tokio::net::TcpStream;
        
        for offset in 0..MAX_PORT_ATTEMPTS {
            let port = base_port
                .checked_add(offset)
                .ok_or_else(|| anyhow!("Port overflow after {} attempts", MAX_PORT_ATTEMPTS))?;
            
            // Essayer de se connecter en TCP directement
            let addr = format!("127.0.0.1:{}", port);
            
            // Utiliser un timeout court pour éviter de bloquer
            match tokio::time::timeout(
                Duration::from_millis(100),
                TcpStream::connect(&addr)
            ).await {
                Ok(Ok(_)) => {
                    // Connexion réussie - port est occupé
                    // Vérifier si c'est un serveur CDP valide
                    let client = reqwest::Client::new();
                    let url = format!("http://127.0.0.1:{}/json/list", port);
                    
                    match client.get(&url).timeout(Duration::from_millis(100)).send().await {
                        Ok(response) if response.status().is_success() => {
                            // Port est occupé par un serveur CDP
                            continue;
                        }
                        _ => {
                            // Port répond mais n'est pas un serveur CDP valide
                            // ou erreur de connexion - essayer le prochain
                            continue;
                        }
                    }
                }
                Ok(Err(_)) | Err(_) => {
                    // Connection refused ou timeout = port libre
                    return Ok(port);
                }
            }
        }
        
        bail!(
            "Could not find an available CDP port after trying {} ports starting from {}",
            MAX_PORT_ATTEMPTS,
            base_port
        );
    }

    /// Lance une application Electron avec --remote-debugging-port
    /// 
    /// Construit la commande: `{electron_path} {app_path} --remote-debugging-port={port} {app_args}`
    pub fn launch(
        electron_path: &Path,
        app_path: &Path,
        port: u16,
        extra_args: &str,
    ) -> Result<Self> {
        let electron_path_str = electron_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid Electron path: {:?}", electron_path))?;
        let app_path_str = app_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid app path: {:?}", app_path))?;
        
        // Construire les arguments de la commande
        let mut args = vec![
            app_path_str.to_string(),
            format!("--remote-debugging-port={}", port),
        ];
        
        // Ajouter les arguments supplémentaires si fournis
        if !extra_args.is_empty() {
            args.extend(extra_args.split_whitespace().map(|s| s.to_string()));
        }
        
        println!("🚀 Launching Electron: {} {}", electron_path_str, args.join(" "));
        
        // Lancer le processus
        let child = Command::new(electron_path_str)
            .args(&args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                anyhow!(
                    "Failed to launch Electron at {:?}: {}",
                    electron_path,
                    e
                )
            })?;
        
        Ok(Self { child, port })
    }

    /// Attend que le port CDP soit disponible (max 30s)
    pub async fn wait_for_cdp(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/json/list", self.port);
        let start = Instant::now();
        
        loop {
            if start.elapsed() > CDP_WAIT_TIMEOUT {
                bail!(
                    "Timeout waiting for CDP on port {} after {:?}",
                    self.port,
                    CDP_WAIT_TIMEOUT
                );
            }
            
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        // Vérifier qu'il y a des targets
                        if let Ok(targets) = response.json::<serde_json::Value>().await {
                            if targets.as_array().map_or(false, |arr| !arr.is_empty()) {
                                println!("✅ CDP available on port {}", self.port);
                                return Ok(());
                            }
                        }
                    }
                }
                Err(_) => {
                    // Connection refused, continuer
                }
            }
            
            sleep(Duration::from_millis(500)).await;
        }
    }

    /// Récupère le PID de l'application lancée
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Tue l'application et attend sa terminaison
    pub fn kill(&mut self) -> Result<()> {
        let pid = self.child.id();
        println!("💀 Killing Electron process {}...", pid);
        
        // Tuer le processus (ne pas attendre - ça peut bloquer)
        let _ = self.child.kill();
        
        println!("✅ Electron process {} kill signal sent", pid);
        Ok(())
    }


}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_find_available_port() {
        // Tester avec un port qui devrait être libre (port très élevé)
        let port = AppLauncher::find_available_port(29999).await;
        assert!(port.is_ok());
        let port = port.unwrap();
        assert!(port >= 29999 && port < 29999 + MAX_PORT_ATTEMPTS);
    }

    #[test]
    fn test_detect_electron_path() {
        // Test with the example electron-app which has node_modules
        let example_path = Path::new("examples/electron-app");
        if example_path.exists() {
            let detected = detect_electron_path(example_path);
            assert!(detected.is_some(), "Should detect electron in examples/electron-app");
            let path = detected.unwrap();
            assert!(path.to_string_lossy().contains("node_modules"));
        }
    }
}
