pub fn install_env(env: &str) {
    match env {
        "Python" => {
            println!("\x1b[32m[xtask]\x1b[0m Checking if Python is already installed...");
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
                println!(
                    "\x1b[32m[xtask]\x1b[0m Python is already installed. No need to reinstall."
                );
                return;
            }

            println!(
                "\x1b[32m[xtask]\x1b[0m Python not detected. Installing Python environment..."
            );

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
                    println!(
                        "\x1b[31m[xtask]\x1b[0m No supported package manager found (apt-get, dnf, yum, pacman). Please install Python manually."
                    );
                    return;
                }
            };

            match output {
                Ok(status) if status.success() => {
                    println!("\x1b[32m[xtask]\x1b[0m Python installation completed!")
                }
                Ok(status) => {
                    println!(
                        "\x1b[31m[xtask]\x1b[0m Python installation failed, exit code: {:?}",
                        status.code()
                    )
                }
                Err(e) => println!(
                    "\x1b[31m[xtask]\x1b[0m Error occurred while running install command: {e}"
                ),
            }
        }
        "xmake" => {
            println!("\x1b[32m[xtask]\x1b[0m Checking if xmake is already installed...");
            println!();

            // Check if xmake is installed
            let check_installed = std::process::Command::new("xmake")
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if check_installed {
                println!(
                    "\x1b[32m[xtask]\x1b[0m xmake is already installed. No need to reinstall."
                );
                return;
            }

            println!("\x1b[32m[xtask]\x1b[0m xmake not detected. Installing xmake environment...");

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
                Ok(status) if status.success() => {
                    println!("\x1b[32m[xtask]\x1b[0m xmake installation completed!")
                }
                Ok(status) => println!(
                    "\x1b[31m[xtask]\x1b[0m xmake installation failed, exit code: {:?}",
                    status.code()
                ),
                Err(e) => println!(
                    "\x1b[31m[xtask]\x1b[0m Error occurred while running install command: {e}"
                ),
            }
        }
        "CUDA" => {
            println!("\x1b[32m[xtask]\x1b[0m Checking if CUDA Toolkit is already installed...");
            println!();

            // Check if cuda toolkit is installed
            let check_installed = std::process::Command::new("nvcc")
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if check_installed {
                println!(
                    "\x1b[32m[xtask]\x1b[0m CUDA Toolkit is already installed. No need to reinstall."
                );
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
                        println!("\x1b[32m[xtask]\x1b[0m Detected by nvidia-smi: {line}");
                    }
                    if let Some(idx) = line.find("CUDA Version:") {
                        // extract the CUDA version number
                        let version_str = line[idx + "CUDA Version:".len()..]
                            .split_whitespace()
                            .next()
                            .unwrap_or("");
                        println!(
                            "\x1b[32m[xtask]\x1b[0m The highest CUDA version supported by your driver is {version_str}"
                        );
                        println!(
                            "\x1b[33m[xtask]\x1b[0m You can also visit https://docs.nvidia.com/cuda/cuda-toolkit-release-notes/index.html to find the CUDA version compatible with your GPU driver."
                        );
                    }
                }
                println!(
                    "\x1b[32m[xtask]\x1b[0m Please make sure to install a CUDA Toolkit version compatible with your driver."
                );
            } else {
                println!(
                    "\x1b[31m[xtask]\x1b[0m nvidia-smi not found. Please make sure you have an NVIDIA GPU and drivers installed."
                );
            }

            println!();
            println!(
                "\x1b[33m[xtask]\x1b[0m Please visit https://developer.nvidia.com/cuda-toolkit-archive to select and download the appropriate CUDA version for your driver."
            );
        }
        "OpenCL" => {
            println!(
                "\x1b[32m[xtask]\x1b[0m The current automatic installation script only supports OpenCL installation for Intel CPU on Windows or Ubuntu systems."
            );
            println!("\x1b[32m[xtask]\x1b[0m Checking if OpenCL is already installed...");
            println!();

            // Check if OpenCL is installed
            #[cfg(target_os = "windows")]
            {
                let clinfo_path = std::path::Path::new("clinfo.exe");
                if !clinfo_path.exists() {
                    println!("\x1b[32m[xtask]\x1b[0m Downloading clinfo tool...");
                    let download_status = std::process::Command::new("curl")
                        .args(["-o", "clinfo.exe", "https://github.com/ahoylabs/clinfo/releases/download/master-d2baa06/clinfo.exe"])
                        .status();

                    if let Err(e) = download_status {
                        println!("\x1b[31m[xtask]\x1b[0m Failed to download clinfo: {}", e);
                        println!("\x1b[33m[xtask]\x1b[0m You may need to enable proxy.");
                        println!(
                            "\x1b[33m[xtask]\x1b[0m You can also manually download from https://github.com/ahoylabs/clinfo/releases/download/master-d2baa06/clinfo.exe"
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
                            println!("\x1b[32m[xtask]\x1b[0m OpenCL is not installed.");
                        } else {
                            println!(
                                "\x1b[32m[xtask]\x1b[0m OpenCL is installed. Number of platforms: {}",
                                number
                            );
                            return;
                        }
                    } else {
                        println!("\x1b[31m[xtask]\x1b[0m Failed to parse the number of platforms.");
                    }
                } else {
                    println!(
                        "\x1b[31m[xtask]\x1b[0m Failed to find 'Number of platforms' in clinfo output."
                    );
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
                        println!("\x1b[32m[xtask]\x1b[0m Installing clinfo tool...");
                        let install_status = std::process::Command::new("sh")
                            .arg("-c")
                            .arg("sudo apt update && sudo apt install opencl-headers ocl-icd-opencl-dev -y")
                            .status();

                        if let Err(e) = install_status {
                            println!("\x1b[31m[xtask]\x1b[0m Failed to install clinfo: {}", e);
                            return;
                        }
                    } else {
                        println!(
                            "\x1b[31m[xtask]\x1b[0m Unsupported package manager. Please install clinfo manually."
                        );
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
                            println!("\x1b[32m[xtask]\x1b[0m OpenCL is not installed.");
                        } else {
                            println!(
                                "\x1b[32m[xtask]\x1b[0m OpenCL is installed. Number of platforms: {}",
                                number
                            );
                            return;
                        }
                    } else {
                        println!("\x1b[31m[xtask]\x1b[0m Failed to parse the number of platforms.");
                    }
                } else {
                    println!(
                        "\x1b[31m[xtask]\x1b[0m Failed to find 'Number of platforms' in clinfo output."
                    );
                }
            }

            println!(
                "\x1b[32m[xtask]\x1b[0m OpenCL not detected. Installing OpenCL environment..."
            );

            #[cfg(target_os = "windows")]
            {
                let download_status = std::process::Command::new("curl")
                    .args(["-o", "w_opencl_runtime_p_2025.1.0.972.exe", "https://registrationcenter-download.intel.com/akdlm/IRC_NAS/b6dccdb7-b503-41ea-bd4b-a78e9c2d8dd6/w_opencl_runtime_p_2025.1.0.972.exe"])
                    .status();

                if let Err(e) = download_status {
                    println!(
                        "\x1b[31m[xtask]\x1b[0m Failed to download w_opencl_runtime_p_2025.1.0.972: {}",
                        e
                    );
                    println!("\x1b[33m[xtask]\x1b[0m You may need to enable proxy.");
                    println!(
                        "\x1b[33m[xtask]\x1b[0m You can also manually download from https://registrationcenter-download.intel.com/akdlm/IRC_NAS/b6dccdb7-b503-41ea-bd4b-a78e9c2d8dd6/w_opencl_runtime_p_2025.1.0.972.exe"
                    );
                    return;
                }

                println!(
                    "\x1b[33m[xtask]\x1b[0m Download completed. Please manually execute 'w_opencl_runtime_p_2025.1.0.972.exe' to install OpenCL for Intel CPU."
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
                    println!("\x1b[32m[xtask]\x1b[0m Installing opencl-headers...");
                    let install_status = std::process::Command::new("sh")
                        .arg("-c")
                        .arg("sudo apt update && sudo apt install opencl-headers ocl-icd-opencl-dev -y")
                        .status();

                    if let Err(e) = install_status {
                        println!("\x1b[31m[xtask]\x1b[0m Failed to install OpenCL: {}", e);
                        return;
                    }

                    println!("\x1b[32m[xtask]\x1b[0m Installing Intel OpenCL runtime...");
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
                        println!(
                            "\x1b[31m[xtask]\x1b[0m Failed to set up Intel OpenCL repository: {}",
                            e
                        );
                        return;
                    }

                    println!(
                        "\x1b[33m[xtask]\x1b[0m Intel OpenCL runtime installation requires a proxy and may take time."
                    );
                    println!(
                        "\x1b[33m[xtask]\x1b[0m Please manually execute the following command after enabling the proxy:"
                    );
                    println!(
                        "\x1b[33m[xtask]\x1b[0m sudo apt install -y intel-oneapi-runtime-opencl"
                    );
                } else {
                    println!(
                        "\x1b[31m[xtask]\x1b[0m Unsupported package manager. Please install OpenCL manually."
                    );
                }
            }
        }
        _ => println!(
            "\x1b[31m[xtask]\x1b[0m Automatic installation for this environment is not supported: {env}"
        ),
    }
}
