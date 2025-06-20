use colored::*;
use std::io::{self, Write};

pub fn info(msg: &str) {
    println!("{} {}", "[xtask]".green().bold(), msg);
}

pub fn error(msg: &str) {
    println!("{} {}", "[xtask]".red().bold(), msg);
}

pub fn warning(msg: &str) {
    println!("{} {}", "[xtask]".yellow().bold(), msg);
}

pub fn install_env(env: &str) {
    match env {
        "Python" => {
            info("Checking if Python is already installed...");
            println!();

            // Check if python or python3 is installed
            let check_installed = if cfg!(target_os = "windows") {
                std::process::Command::new("python")
                    .arg("--version")
                    .stdout(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            } else {
                std::process::Command::new("python3")
                    .arg("--version")
                    .stdout(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            };

            if check_installed {
                info("Python is already installed. No need to reinstall.");
                return;
            }

            info("Python not detected. Installing Python environment...");

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
                        .stdout(std::process::Stdio::null())
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
                    error(
                        "No supported package manager found (apt-get, dnf, yum, pacman). Please install Python manually.",
                    );
                    return;
                }
            };

            match output {
                Ok(status) if status.success() => info("Python installation completed!"),
                Ok(status) => error(&format!(
                    "Python installation failed, exit code: {:?}",
                    status.code()
                )),
                Err(e) => error(&format!(
                    "Error occurred while running install command: {e}"
                )),
            }
        }
        "xmake" => {
            info("Checking if xmake is already installed...");
            println!();

            // Check if xmake is installed
            let check_installed = std::process::Command::new("xmake")
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if check_installed {
                info("xmake is already installed. No need to reinstall.");
                return;
            }

            info("xmake not detected. Installing xmake environment...");

            #[cfg(target_os = "windows")]
            let output = std::process::Command::new("powershell")
                .args([
                    "-Command",
                    "winget install -e --id Xmake-io.Xmake --source winget",
                ])
                .status();

            #[cfg(not(target_os = "windows"))]
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg("curl -fsSL https://xmake.io/shget.text | bash")
                .status();

            match output {
                Ok(status) if status.success() => info("xmake installation completed!"),
                Ok(status) => error(&format!(
                    "xmake installation failed, exit code: {:?}",
                    status.code()
                )),
                Err(e) => error(&format!(
                    "Error occurred while running install command: {e}"
                )),
            }
        }
        "CUDA" => {
            info("Checking if CUDA Toolkit is already installed...");
            println!();

            // Check if cuda toolkit is installed
            let check_installed = std::process::Command::new("nvcc")
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if check_installed {
                info("CUDA Toolkit is already installed. No need to reinstall.");
                return;
            }

            // nvidia-smi, get the highest CUDA version supported by the driver
            let has_nvidia_smi = std::process::Command::new("nvidia-smi")
                .stdout(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if has_nvidia_smi {
                let output = std::process::Command::new("nvidia-smi")
                    .output()
                    .expect("Failed to execute nvidia-smi");
                let smi_info = String::from_utf8_lossy(&output.stdout);
                for line in smi_info.lines() {
                    if line.contains("CUDA Version") {
                        info(&format!("Detected by nvidia-smi: {line}"));
                    }
                    if let Some(idx) = line.find("CUDA Version:") {
                        // extract the CUDA version number
                        let version_str = line[idx + "CUDA Version:".len()..]
                            .split_whitespace()
                            .next()
                            .unwrap_or("");
                        info(&format!(
                            "The highest CUDA version supported by your driver is {version_str}"
                        ));
                        warning(
                            "You can also visit https://docs.nvidia.com/cuda/cuda-toolkit-release-notes/index.html to find the CUDA version compatible with your GPU driver.",
                        );
                    }
                }
                info(
                    "Please make sure to install a CUDA Toolkit version compatible with your driver.",
                );
            } else {
                error(
                    "nvidia-smi not found. Please make sure you have an NVIDIA GPU and drivers installed.",
                );
                return;
            }

            println!();
            warning(
                "Please visit https://developer.nvidia.com/cuda-toolkit-archive to select and download the appropriate CUDA version for your driver.",
            );
        }
        "OpenCL" => {
            warning(
                "The current automatic installation script only supports OpenCL installation for Intel CPU on Windows or Ubuntu systems.",
            );
            warning("type 'y' to continue, or any other key to exit.");
            let mut input = String::new();
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");
            if input.trim().to_lowercase() != "y" {
                info("Exiting OpenCL installation.");
                return;
            }

            info("Checking if OpenCL is already installed...");
            println!();

            // Check if OpenCL is installed
            #[cfg(target_os = "windows")]
            {
                let clinfo_path = std::path::Path::new("clinfo.exe");
                if !clinfo_path.exists() {
                    info("Downloading clinfo tool...");
                    let download_status = std::process::Command::new("curl")
                        .args(["-o", "clinfo.exe", "https://github.com/ahoylabs/clinfo/releases/download/master-d2baa06/clinfo.exe"])
                        .status();

                    if let Err(e) = download_status {
                        error(&format!("Failed to download clinfo: {}", e));
                        warning("You may need to enable proxy.");
                        warning(
                            "You can also manually download from https://github.com/ahoylabs/clinfo/releases/download/master-d2baa06/clinfo.exe",
                        );
                        return;
                    }
                }

                let output = std::process::Command::new("clinfo.exe")
                    .output()
                    .expect("Failed to execute clinfo.exe");

                let clinfo_output = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = clinfo_output
                    .lines()
                    .find(|line| line.contains("Number of platforms"))
                {
                    if let Some(number) = line.split_whitespace().last() {
                        if number == "0" {
                            info("OpenCL is not installed.");
                        } else {
                            info(&format!(
                                "OpenCL is installed. Number of platforms: {}",
                                number
                            ));
                            return;
                        }
                    } else {
                        error("Failed to parse the number of platforms.");
                        return;
                    }
                } else {
                    error("Failed to find 'Number of platforms' in clinfo output.");
                    return;
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let has_cmd = |cmd: &str| {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg(format!("command -v {}", cmd))
                        .stdout(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                };

                if !has_cmd("clinfo") {
                    if has_cmd("apt") {
                        info("Installing clinfo tool...");
                        let install_status = std::process::Command::new("sh")
                            .arg("-c")
                            .arg("sudo apt update && sudo apt install clinfo -y")
                            .status();

                        if let Err(e) = install_status {
                            error(&format!("Failed to install clinfo: {}", e));
                            return;
                        }
                    } else {
                        error("Unsupported package manager. Please install clinfo manually.");
                        return;
                    }
                }

                let output = std::process::Command::new("clinfo")
                    .output()
                    .expect("Failed to execute clinfo");

                let clinfo_output = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = clinfo_output
                    .lines()
                    .find(|line| line.contains("Number of platforms"))
                {
                    if let Some(number) = line.split_whitespace().last() {
                        if number == "0" {
                            info("OpenCL is not installed.");
                        } else {
                            info(&format!(
                                "OpenCL is installed. Number of platforms: {}",
                                number
                            ));
                            return;
                        }
                    } else {
                        error("Failed to parse the number of platforms.");
                        return;
                    }
                } else {
                    error("Failed to find 'Number of platforms' in clinfo output.");
                    return;
                }
            }

            info("OpenCL not detected. Installing OpenCL environment...");

            #[cfg(target_os = "windows")]
            {
                let download_status = std::process::Command::new("curl")
                    .args(["-o", "w_opencl_runtime_p_2025.1.0.972.exe", "https://registrationcenter-download.intel.com/akdlm/IRC_NAS/b6dccdb7-b503-41ea-bd4b-a78e9c2d8dd6/w_opencl_runtime_p_2025.1.0.972.exe"])
                    .status();

                if let Err(e) = download_status {
                    error(&format!(
                        "Failed to download w_opencl_runtime_p_2025.1.0.972: {}",
                        e
                    ));
                    warning("You may need to enable proxy.");
                    warning(
                        "You can also manually download from https://registrationcenter-download.intel.com/akdlm/IRC_NAS/b6dccdb7-b503-41ea-bd4b-a78e9c2d8dd6/w_opencl_runtime_p_2025.1.0.972.exe",
                    );
                    return;
                }

                warning(
                    "Download completed. Please manually execute 'w_opencl_runtime_p_2025.1.0.972.exe' to install OpenCL for Intel CPU.",
                );
            }
            #[cfg(not(target_os = "windows"))]
            {
                let has_cmd = |cmd: &str| {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg(format!("command -v {}", cmd))
                        .stdout(std::process::Stdio::null())
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                };

                if has_cmd("apt") {
                    info("Installing opencl-headers...");
                    let install_status = std::process::Command::new("sh")
                        .arg("-c")
                        .arg("sudo apt update && sudo apt install opencl-headers ocl-icd-opencl-dev -y")
                        .status();

                    if let Err(e) = install_status {
                        error(&format!("Failed to install OpenCL: {}", e));
                        return;
                    }

                    info("Installing Intel OpenCL runtime...");
                    let setup_status = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(
                            "wget -O- https://apt.repos.intel.com/intel-gpg-keys/GPG-PUB-KEY-INTEL-SW-PRODUCTS.PUB \
                            | gpg --dearmor | sudo tee /usr/share/keyrings/oneapi-archive-keyring.gpg > /dev/null && \
                            echo \"deb [signed-by=/usr/share/keyrings/oneapi-archive-keyring.gpg] https://apt.repos.intel.com/oneapi all main\" | sudo tee /etc/apt/sources.list.d/oneAPI.list && \
                            sudo apt update"
                        )
                        .status();

                    if let Err(e) = setup_status {
                        error(&format!("Failed to set up Intel OpenCL repository: {}", e));
                        return;
                    }

                    warning(
                        "Intel OpenCL runtime installation requires a proxy and may take time.",
                    );
                    warning(
                        "Please manually execute the following command after enabling the proxy:",
                    );
                    warning("sudo apt install -y intel-oneapi-runtime-opencl");
                } else {
                    error("Unsupported package manager. Please install OpenCL manually.");
                }
            }
        }
        _ => error(&format!(
            "Automatic installation for this environment is not supported: {env}"
        )),
    }
}
