use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Args)]
pub struct EvalArgs {
    /// 要评分的课程名称，不传则自动对所有已配置课程评分
    #[clap(long)]
    course: Option<String>,
    
    /// 练习目录路径，默认为当前目录
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
    
    /// 是否显示详细输出
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExerciseResult {
    pub name: String,
    pub result: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Statistics {
    pub total_exercations: usize,
    pub total_succeeds: usize,
    pub total_failures: usize,
    pub total_time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GradeResult {
    pub exercises: Vec<ExerciseResult>,
    pub statistics: Statistics,
}

impl EvalArgs {
    pub fn eval(self) {
        if let Err(e) = self.run_eval() {
            eprintln!("{} {}", "评分失败:".red().bold(), e);
        }
    }
    
    fn run_eval(&self) -> Result<()> {
        println!("{}", "开始评测练习...".blue().bold());
        
        // 查找所有练习文件
        let exercise_files = find_exercise_files(&self.path)?;
        
        println!("{} {} {}", "找到".blue().bold(), exercise_files.len(), "个练习文件".blue().bold());
        
        // 创建进度条
        let total_exercises = exercise_files.len() as u64;
        let bar = ProgressBar::new(total_exercises);
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("##-"),
        );
        
        let mut exercise_results = Vec::new();
        let mut total_succeeds = 0;
        let mut total_failures = 0;
        let mut total_time = 0;
        
        for exercise_path in exercise_files.iter() {
            bar.inc(1); // 更新进度条
            
            let (name, result, time) = grade_exercise(exercise_path, self.verbose)?;
            
            if result {
                total_succeeds += 1;
            } else {
                total_failures += 1;
            }
            
            total_time += time;
            
            exercise_results.push(ExerciseResult {
                name,
                result,
            });
        }
        
        bar.finish_with_message("评测完成!");
        
        // 打印统计信息
        println!("{}", "评测结果统计".green().bold());
        println!("{}: {}", "总练习数".blue(), exercise_files.len());
        println!("{}: {}", "通过数量".green(), total_succeeds);
        println!("{}: {}", "失败数量".red(), total_failures);
        println!("{}: {}秒", "总耗时".blue(), total_time);
        
        // 计算通过率
        let pass_rate = if exercise_files.len() > 0 {
            (total_succeeds as f32 / exercise_files.len() as f32) * 100.0
        } else {
            0.0
        };
        println!("{}: {:.2}%", "通过率".green(), pass_rate);
        
        // 显示失败的练习
        if total_failures > 0 {
            println!("");
            println!("{}", "失败的练习:".red().bold());
            for exercise in exercise_results.iter() {
                if !exercise.result {
                    println!("  {}", exercise.name.red());
                }
            }
        }
        
        // 将结果保存到文件
        let result = GradeResult {
            exercises: exercise_results,
            statistics: Statistics {
                total_exercations: exercise_files.len(),
                total_succeeds,
                total_failures,
                total_time,
            },
        };
        
        let json_result = serde_json::to_string_pretty(&result)?;
        fs::write("rustlings_result.json", json_result)?;
        println!("");
        println!("{}", "评测结果已保存到 rustlings_result.json".blue());
        
        Ok(())
    }
}

/// 查找指定目录下的所有Rustlings练习文件
fn find_exercise_files(exercises_path: &Path) -> Result<Vec<PathBuf>> {
    let mut exercise_files = Vec::new();
    let is_learning_lm = exercises_path.to_string_lossy().contains("learning-lm-rs");
    
    // learning-lm-rs项目的测试文件列表 - 只测试这两个文件
    let learning_lm_files = [
        "model.rs",      // test_mlp, test_load_safetensors
        "operators.rs", // test_matmul_transb, test_silu, test_rms_norm
    ];
    
    if is_learning_lm {
        // 对于learning-lm-rs项目，直接在src目录下查找文件
        let src_dir = if exercises_path.ends_with("learning-lm-rs") {
            exercises_path.join("src")
        } else if exercises_path.ends_with("src") {
            exercises_path.to_path_buf()
        } else {
            // 尝试找到learning-lm-rs/src目录
            let mut lm_path = exercises_path.to_path_buf();
            while !lm_path.ends_with("learning-lm-rs") && lm_path.parent().is_some() {
                lm_path = lm_path.parent().unwrap().to_path_buf();
            }
            if lm_path.ends_with("learning-lm-rs") {
                lm_path.join("src")
            } else {
                println!("{} {}", "警告:".yellow().bold(), "找不到learning-lm-rs/src目录");
                return Ok(Vec::new());
            }
        };
        
        // 检查src目录是否存在
        if !src_dir.exists() {
            println!("{} {}", "警告:".yellow().bold(), "src目录不存在，请检查learning-lm-rs项目结构");
            return Ok(Vec::new());
        }
        
        println!("{}", "仅测试model.rs和operators.rs文件".blue().bold());
        
        // 清空之前可能收集的所有文件
        exercise_files.clear();
        
        // 在src目录下只添加model.rs和operators.rs文件
        for file_name in learning_lm_files.iter() {
            let file_path = src_dir.join(file_name);
            if file_path.exists() {
                println!("{} {}", "添加测试文件:".green().bold(), file_path.display());
                exercise_files.push(file_path);
            }
        }
        
        // 确保只返回这两个文件，不进行其他文件的查找
        return Ok(exercise_files);
    } else {
        // Rustlings练习：在exercises目录下查找
        let exercises_dir = Path::new("exercises");
        let base_path = if exercises_path.ends_with(exercises_dir) {
            exercises_path.to_path_buf()
        } else {
            exercises_path.join(exercises_dir)
        };
        
        // 检查exercises目录是否存在
        if !base_path.exists() {
            println!("{} {}", "警告:".yellow().bold(), "exercises目录不存在，请先配置课程");
            return Ok(Vec::new());
        }
        
        for entry in walkdir::WalkDir::new(&base_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.components().any(|c| c.as_os_str() == "target") {
                continue;
            }
            if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                let file_name = path.file_name().unwrap().to_string_lossy();
                
                // Rustlings练习：排除测试文件和辅助文件
                if !file_name.starts_with("test_") && !file_name.starts_with("helper_") {
                    exercise_files.push(path.to_path_buf());
                }
            }
        }
    }
    
    Ok(exercise_files)
}

/// 评测单个练习文件
fn grade_exercise(exercise_path: &Path, verbose: bool) -> Result<(String, bool, u64)> {
    let start = Instant::now();
    let exercise_name = exercise_path
        .file_name()
        .context("无法获取文件名")?
        .to_string_lossy()
        .to_string();
    
    println!("{} {}", "评测练习:".blue().bold(), exercise_name);
    
    // 确保target目录存在
    fs::create_dir_all("target/debug").context("创建target目录失败")?;
    
    // 检查是否为learning-lm-rs项目的文件
    let is_learning_lm = exercise_path.to_string_lossy().contains("learning-lm-rs");
    
    let (compile_output, test_output) = if is_learning_lm {
        // learning-lm-rs项目：使用cargo test
        // 获取子模块项目的Cargo.toml路径
        let module_dir = exercise_path.parent().unwrap().parent().unwrap();
        let manifest_path = module_dir.join("Cargo.toml");
        
        // 获取测试名称，根据文件名确定要运行的测试
        let test_name = if exercise_name == "model.rs" {
            "test_mlp test_load_safetensors"
        } else if exercise_name == "operators.rs" {
            "test_matmul_transb test_silu test_rms_norm"
        } else {
            ""
        };
        
        println!("{} {}", "运行测试:".blue().bold(), test_name);
        
        // 编译项目
        let compile_output = Command::new("cargo")
            .arg("test")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--no-run")  // 仅编译不运行
            .arg("--release") // 使用release模式编译
            .current_dir(module_dir) // 移动到项目根目录
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(format!("编译练习 {} 失败", exercise_name))?;
            
        if !compile_output.status.success() {
            if verbose {
                println!("{}", String::from_utf8_lossy(&compile_output.stderr));
            }
            println!("{} {}", "编译失败:".red().bold(), exercise_name);
            return Ok((exercise_name, false, start.elapsed().as_secs()));
        }
        
        // 构建测试命令，只运行特定的测试
        let mut test_command = Command::new("cargo");
        test_command.arg("test")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--release"); // 使用release模式运行测试
            
        // 添加特定的测试名称
        if !test_name.is_empty() {
            for test in test_name.split_whitespace() {
                test_command.arg(test);
            }
        }
        
        let test_output = test_command
            .current_dir(module_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(format!("运行练习 {} 失败", exercise_name))?;
            
        (compile_output, test_output)
    } else {
        // Rustlings练习：使用rustc
        let compile_output = Command::new("rustc")
            .arg(exercise_path)
            .arg("--test")
            .arg("-o")
            .arg(format!("target/debug/{}", exercise_name))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(format!("编译练习 {} 失败", exercise_name))?;
            
        if !compile_output.status.success() {
            if verbose {
                println!("{}", String::from_utf8_lossy(&compile_output.stderr));
            }
            println!("{} {}", "编译失败:".red().bold(), exercise_name);
            return Ok((exercise_name, false, start.elapsed().as_secs()));
        }
        
        let test_output = Command::new(format!("target/debug/{}", exercise_name))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(format!("运行练习 {} 失败", exercise_name))?;
            
        (compile_output, test_output)
    };
    
    let success = test_output.status.success();
    
    if verbose || !success {
        println!("{}", String::from_utf8_lossy(&test_output.stdout));
        println!("{}", String::from_utf8_lossy(&test_output.stderr));
    }
    
    if success {
        println!("{} {}", "✓".green().bold(), exercise_name);
    } else {
        println!("{} {}", "✗".red().bold(), exercise_name);
    }
    
    Ok((exercise_name, success, start.elapsed().as_secs()))
}
