pub fn install_env(env: &str) {
    match env {
        "python" => {
            println!("Checking if Python is already installed...");
            // Check if python or python3 is installed
            let check_installed = if cfg!(target_os = "windows") {
                std::process::Command::new("python")
                    .arg("--version")
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            } else {
                std::process::Command::new("python3")
                    .arg("--version")
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            };

            if check_installed {
                println!("Python is already installed. No need to reinstall.");
                return;
            }

            println!("Python not detected. Installing Python environment...");

            #[cfg(target_os = "windows")]
            let output = std::process::Command::new("powershell")
                .args(["-Command", "winget install -e --id Python.Python.3.11"])
                .status();

            #[cfg(not(target_os = "windows"))]
            let output = {
                // Detect package manager
                let has_cmd = |cmd: &str| {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg(format!("command -v {}", cmd))
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                };

                if has_cmd("apt-get") {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg("sudo apt-get update && sudo apt-get install -y python3")
                        .status()
                } else if has_cmd("dnf") {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg("sudo dnf install -y python3")
                        .status()
                } else if has_cmd("yum") {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg("sudo yum install -y python3")
                        .status()
                } else if has_cmd("pacman") {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg("sudo pacman -Sy --noconfirm python")
                        .status()
                } else if has_cmd("brew") {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg("brew install python")
                        .status()
                } else {
                    println!("No supported package manager found (apt-get, dnf, yum, pacman). Please install Python manually.");
                    return;
                }
            };

            match output {
                Ok(status) if status.success() => println!("Python installation completed!"),
                Ok(status) => println!("Python installation failed, exit code: {:?}", status.code()),
                Err(e) => println!("Error occurred while running install command: {e}"),
            }
        }
        "xmake" => {
            println!("Checking if xmake is already installed...");
            // Check if xmake is installed
            let check_installed = std::process::Command::new("xmake")
                .arg("--version")
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if check_installed {
                println!("xmake is already installed. No need to reinstall.");
                return;
            }

            println!("xmake not detected. Installing xmake environment...");

            #[cfg(target_os = "windows")]
            let output = std::process::Command::new("powershell")
                .args(["-Command", "winget install -e --id Xmake-io.Xmake --source winget"])
                .status();

            #[cfg(not(target_os = "windows"))]
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg("curl -fsSL https://xmake.io/shget.text | bash")
                .status();

            match output {
                Ok(status) if status.success() => println!("xmake installation completed!"),
                Ok(status) => println!("xmake installation failed, exit code: {:?}", status.code()),
                Err(e) => println!("Error occurred while running install command: {e}"),
            }
        }
        _ => println!("Automatic installation for this environment is not supported: {env}"),
    }
}