use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
use tempfile;

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
    
    /// 评测learning-lm-rs项目
    fn eval_learning_lm(&self, lm_path: &Path) -> Result<(Vec<ExerciseResult>, usize, usize, usize)> {
        println!("{}", "评测 learning-lm-rs 项目...".blue().bold());
        
        let manifest_path = lm_path.join("Cargo.toml");
        if !manifest_path.exists() {
            println!("{} {} {}", "警告:".yellow().bold(), "找不到 learning-lm-rs/Cargo.toml 文件:", manifest_path.display());
            return Ok((Vec::new(), 0, 0, 0));
        }

        println!("{} {}", "运行测试:".blue().bold(), "cargo test --release");
        let test_output = Command::new("cargo")
            .arg("test")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--release")
            .current_dir(lm_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("运行 learning-lm-rs 测试失败")?;

        let success = test_output.status.success();

        if self.verbose || !success {
            println!("{}", String::from_utf8_lossy(&test_output.stdout));
            println!("{}", String::from_utf8_lossy(&test_output.stderr));
        }

        // learning-lm-rs 只包含 model.rs 和 operators.rs
        let lm_exercises = ["model.rs", "operators.rs"];
        let total_exercations = lm_exercises.len();
        let mut exercise_results = Vec::new();
        let mut total_succeeds = 0;
        let mut total_failures = 0;

        for &exercise_name in lm_exercises.iter() {
            exercise_results.push(ExerciseResult {
                name: exercise_name.to_string(),
                result: success,
            });
            if success {
                total_succeeds += 1;
                println!("{} {}", "✓".green().bold(), exercise_name);
            } else {
                total_failures += 1;
                println!("{} {}", "✗".red().bold(), exercise_name);
            }
        }
        println!("评测完成!");
        
        Ok((exercise_results, total_succeeds, total_failures, total_exercations))
    }
    
    /// 评测learning-cxx项目
    fn eval_learning_cxx(&self, course_path: &Path) -> Result<(Vec<ExerciseResult>, usize, usize, usize)> {
        println!("{}", "评测 learning-cxx 项目...".blue().bold());
        
        // 运行xmake run summary命令获取评测结果
        let output = Command::new("xmake")
            .arg("run")
            .arg("summary")
            .current_dir(course_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("运行 xmake run summary 失败")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let error_str = String::from_utf8_lossy(&output.stderr);
        
        if self.verbose {
            println!("{}", output_str);
            println!("{}", error_str);
        }

        let mut exercise_results = Vec::new();
        let mut total_succeeds = 0;
        let mut total_failures = 0;
        let mut total_exercations = 0;

        // 移除ANSI转义序列的正则表达式
        let re = regex::Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();

        // 解析输出结果
        for line in output_str.lines() {
            // 跳过错误信息行
            if line.contains("error:") {
                continue;
            }

            if line.contains("exercise") && (line.contains("passed") || line.contains("failed")) {
                total_exercations += 1;
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(exercise_name) = parts.first() {
                    // 移除ANSI转义序列
                    let clean_name = re.replace_all(exercise_name, "").to_string();
                    let result = line.contains("passed");
                    if result {
                        total_succeeds += 1;
                        println!("{} {}", "✓".green().bold(), clean_name);
                    } else {
                        total_failures += 1;
                        println!("{} {}", "✗".red().bold(), clean_name);
                    }
                    exercise_results.push(ExerciseResult { name: clean_name, result });
                }
            }
        }

        // 如果没有从stdout找到结果，尝试从stderr中解析
        if total_exercations == 0 {
            for line in error_str.lines() {
                // 跳过错误信息行
                if line.contains("error:") {
                    continue;
                }

                if line.contains("exercise") && (line.contains("passed") || line.contains("failed")) {
                    total_exercations += 1;
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(exercise_name) = parts.first() {
                        // 移除ANSI转义序列并处理exercise前缀
                        let clean_name = re.replace_all(exercise_name, "").to_string();
                        let name = clean_name.trim_start_matches("exercise").to_string();
                        let result = line.contains("passed");
                        if result {
                            total_succeeds += 1;
                            println!("{} exercise{}", "✓".green().bold(), name);
                        } else {
                            total_failures += 1;
                            println!("{} exercise{}", "✗".red().bold(), name);
                        }
                        exercise_results.push(ExerciseResult { name: format!("exercise{}", name), result });
                    }
                }
            }
        }

        println!("评测完成!");
        Ok((exercise_results, total_succeeds, total_failures, total_exercations))
    }

    /// 评测rustlings或其他项目
    fn eval_rustlings(&self, course_path: &Path) -> Result<(Vec<ExerciseResult>, usize, usize, usize)> {
        println!("{}", "评测 rustlings 项目...".blue().bold());
        
        // 使用 rustc 编译和运行测试来评测
        println!("{}", "使用 rustc 编译和运行测试来评测...".blue().bold());
        
        // 处理 Rustlings 或其他非 learning-lm-rs 项目
        let exercise_files = find_exercise_files(course_path, &None)?;
        let total_exercations = exercise_files.len();
        println!("{} {} {}", "找到".blue().bold(), total_exercations, "个练习文件".blue().bold());

        if total_exercations == 0 {
            println!("{}", "未找到练习文件，评测结束。".yellow());
            return Ok((Vec::new(), 0, 0, 0));
        }

        let bar = ProgressBar::new(total_exercations as u64);
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("##-"),
        );

        let mut exercise_results = Vec::new();
        let mut total_succeeds = 0;
        let mut total_failures = 0;
        
        for exercise_path in exercise_files.iter() {
            bar.inc(1);
            let (name, result, _time) = grade_exercise(exercise_path, self.verbose)?;
            if result {
                total_succeeds += 1;
            } else {
                total_failures += 1;
            }
            exercise_results.push(ExerciseResult { name, result });
        }
        bar.finish_with_message("评测完成!");
        
        Ok((exercise_results, total_succeeds, total_failures, total_exercations))
    }
    
    fn run_eval(&self) -> Result<()> {
        println!("{}", "开始评测练习...".blue().bold());
        let start_time = Instant::now();

        // 获取当前工作目录
        let current_dir = std::env::current_dir().context("无法获取当前工作目录")?;
        
        // 确定exercises目录
        let absolute_path = current_dir.join(&self.path);
        let exercises_dir = if absolute_path.ends_with("exercises") {
            absolute_path.clone()
        } else {
            absolute_path.join("exercises")
        };

        if !exercises_dir.exists() {
            println!("{} {}", "警告:".yellow().bold(), "找不到exercises目录");
            return Ok(());
        }

        let mut exercise_results = Vec::new();
        let mut total_succeeds = 0;
        let mut total_failures = 0;
        let mut total_exercations = 0;

        // 如果指定了course参数，只评测指定的课程
        if let Some(course) = &self.course {
            let course_path = exercises_dir.join(course);
            if !course_path.exists() {
                println!("{} {}", "警告:".yellow().bold(), format!("找不到课程目录: {}", course_path.display()));
                return Ok(());
            }

            println!("{} {}", "评测指定课程:".blue().bold(), course);
            let (results, succeeds, failures, exercations) = match course.as_str() {
                "learning-lm-rs" => self.eval_learning_lm(&course_path)?,
                "learning-cxx" => self.eval_learning_cxx(&course_path)?,
                _ => self.eval_rustlings(&course_path)?
            };

            exercise_results.extend(results);
            total_succeeds += succeeds;
            total_failures += failures;
            total_exercations += exercations;
        } else {
            // 自动评测所有课程
            println!("{}", "自动评测所有课程...".blue().bold());
            
            // 获取exercises目录下的所有子目录
            let entries = fs::read_dir(&exercises_dir)
                .context(format!("无法读取目录: {}", exercises_dir.display()))?;

            for entry in entries {
                let entry = entry.context("读取目录项失败")?;
                let path = entry.path();
                
                if path.is_dir() {
                    let course_name = path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("未知课程");
                    
                    println!("{} {}", "\n评测课程:".blue().bold(), course_name);
                    
                    let (results, succeeds, failures, exercations) = match course_name {
                        "learning-lm-rs" => self.eval_learning_lm(&path)?,
                        "learning-cxx" => self.eval_learning_cxx(&path)?,
                        _ => self.eval_rustlings(&path)?
                    };
                    
                    exercise_results.extend(results);
                    total_succeeds += succeeds;
                    total_failures += failures;
                    total_exercations += exercations;
                }
            }
        }

        let total_time = start_time.elapsed().as_secs();

        // 打印统计信息
        println!("{}", "评测结果统计".green().bold());
        println!("{}: {}", "总练习数".blue(), total_exercations);
        println!("{}: {}", "通过数量".green(), total_succeeds);
        println!("{}: {}", "失败数量".red(), total_failures);
        println!("{}: {}秒", "总耗时".blue(), total_time);

        let pass_rate = if total_exercations > 0 {
            (total_succeeds as f32 / total_exercations as f32) * 100.0
        } else {
            0.0
        };
        println!("{}: {:.2}%", "通过率".green(), pass_rate);

        if total_failures > 0 {
            println!("");
            println!("{}", "失败的练习:".red().bold());
            for exercise in exercise_results.iter() {
                if !exercise.result {
                    println!("  {}", exercise.name.red());
                }
            }
        }

        let result = GradeResult {
            exercises: exercise_results,
            statistics: Statistics {
                total_exercations,
                total_succeeds,
                total_failures,
                total_time,
            },
        };

        // 使用固定的结果文件名
        let result_filename = "eval_result.json";
        let json_result = serde_json::to_string_pretty(&result)?;
        fs::write(&result_filename, json_result)?;
        println!("");
        println!("{} {}", "评测结果已保存到".blue(), result_filename.blue());

        Ok(())
    }
}



/// 查找指定目录下的所有练习文件
fn find_exercise_files(course_path: &Path, _course: &Option<String>) -> Result<Vec<PathBuf>> {
    let mut exercise_files = Vec::new();
    
    if !course_path.exists() {
        println!("{} {}", "警告:".yellow().bold(), format!("找不到课程目录: {}", course_path.display()));
        return Ok(Vec::new());
    }

    // 对于learning-lm-rs项目，只返回model.rs和operators.rs
    if course_path.file_name().map_or(false, |name| name == "learning-lm-rs") {
        let src_path = course_path.join("src");
        if src_path.exists() {
            println!("{} {}", "找到learning-lm-rs项目:".blue().bold(), src_path.display());
            let model_path = src_path.join("model.rs");
            let operators_path = src_path.join("operators.rs");
            
            if model_path.exists() {
                println!("{} {}", "找到练习文件:".blue(), model_path.display());
                exercise_files.push(model_path);
            } else {
                println!("{} {}", "警告:".yellow().bold(), "找不到model.rs文件");
            }
            
            if operators_path.exists() {
                println!("{} {}", "找到练习文件:".blue(), operators_path.display());
                exercise_files.push(operators_path);
            } else {
                println!("{} {}", "警告:".yellow().bold(), "找不到operators.rs文件");
            }
            
            return Ok(exercise_files);
        }
    }

    // 对于rustlings项目，只查找exercises目录下的文件
    if course_path.file_name().map_or(false, |name| name == "rustlings") {
        let exercises_path = course_path.join("exercises");
        if !exercises_path.exists() {
            println!("{} {}", "警告:".yellow().bold(), "找不到rustlings的exercises目录");
            return Ok(Vec::new());
        }

        for entry in walkdir::WalkDir::new(&exercises_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                let file_name = path.file_name().unwrap().to_string_lossy();
                if !file_name.starts_with("test_") && !file_name.starts_with("helper_") {
                    exercise_files.push(path.to_path_buf());
                }
            }
        }
    } else {
        // 对于其他项目，遍历目录查找练习文件
        for entry in walkdir::WalkDir::new(course_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.components().any(|c| c.as_os_str() == "target") {
                continue;
            }
            if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                let file_name = path.file_name().unwrap().to_string_lossy();
                if !file_name.starts_with("test_") && !file_name.starts_with("helper_") {
                    exercise_files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(exercise_files)
}

/// 评测单个 Rustlings 练习文件 (不再处理 learning-lm-rs)
fn grade_exercise(exercise_path: &Path, verbose: bool) -> Result<(String, bool, u64)> {
    let start = Instant::now();
    let exercise_name = exercise_path
        .file_name()
        .context("无法获取文件名")?
        .to_string_lossy()
        .to_string();

    println!("{} {}", "评测练习:".blue().bold(), exercise_name);

    // 检查是否是 clippy 练习
    let is_clippy_exercise = exercise_path.to_string_lossy().contains("clippy");

    // 如果是 clippy 练习，使用 cargo clippy 命令检查
    if is_clippy_exercise {
        // 创建一个临时目录来存放 Cargo.toml 和源文件
        let temp_dir = tempfile::tempdir().context("创建临时目录失败")?;
        let temp_dir_path = temp_dir.path();
        
        // 创建 Cargo.toml 文件
        let cargo_toml_content = r#"[package]
name = "clippy_check"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "clippy_check"
path = "src/main.rs"
"#;
        let cargo_toml_path = temp_dir_path.join("Cargo.toml");
        fs::write(&cargo_toml_path, cargo_toml_content).context("写入 Cargo.toml 失败")?;
        
        // 创建 src 目录
        let src_dir = temp_dir_path.join("src");
        fs::create_dir(&src_dir).context("创建 src 目录失败")?;
        
        // 复制练习文件到 src/main.rs
        let exercise_content = fs::read_to_string(exercise_path).context("读取练习文件失败")?;
        let main_rs_path = src_dir.join("main.rs");
        fs::write(&main_rs_path, exercise_content).context("写入 main.rs 失败")?;
        
        // 运行 cargo clippy
        let clippy_output = Command::new("cargo")
            .arg("clippy")
            .arg("--manifest-path")
            .arg(&cargo_toml_path)
            .arg("--")
            .arg("-D")
            .arg("warnings")
            .current_dir(temp_dir_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(format!("运行 cargo clippy 检查 {} 失败", exercise_name))?;
        
        let clippy_success = clippy_output.status.success();
        
        if !clippy_success {
            if verbose {
                println!("{}", String::from_utf8_lossy(&clippy_output.stdout));
                println!("{}", String::from_utf8_lossy(&clippy_output.stderr));
            }
            println!("{} {}", "✗".red().bold(), exercise_name);
            return Ok((exercise_name, false, start.elapsed().as_secs()));
        }
    }

    // 对于rustlings练习，直接使用rustc编译和运行测试
    let test_output = Command::new("rustc")
        .arg(exercise_path)
        .arg("--test")
        .arg("-o")
        .arg(format!("target/debug/{}", exercise_name))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context(format!("编译练习 {} 失败", exercise_name))?;

    let success = test_output.status.success();

    if !success {
        if verbose {
            println!("{}", String::from_utf8_lossy(&test_output.stdout));
            println!("{}", String::from_utf8_lossy(&test_output.stderr));
        }
        println!("{} {}", "✗".red().bold(), exercise_name);
        return Ok((exercise_name, false, start.elapsed().as_secs()));
    }

    // 编译成功，运行测试
    let test_output = Command::new(format!("target/debug/{}", exercise_name))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context(format!("运行练习 {} 失败", exercise_name))?;

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
