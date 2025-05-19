use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Args)]
pub struct LearnArgs {
    /// 课程名称
    course: String,
    /// 传入 fork 仓库地址，以 git submodule 方式配置
    #[clap(long)]
    submodule: Option<String>,
}

impl LearnArgs {
    pub fn learn(self) {
        if let Err(e) = self.run_learn() {
            eprintln!("{} {}", "配置课程失败:".red().bold(), e);
        }
    }

    fn run_learn(&self) -> Result<()> {
        println!("{} {}", "开始配置课程:".blue().bold(), self.course);
        
        // 确保exercises目录存在
        let exercises_dir = Path::new("exercises");
        if !exercises_dir.exists() {
            fs::create_dir_all(exercises_dir)
                .context("创建exercises目录失败")?;
        }
        
        // 如果提供了子模块地址，则克隆仓库
        if let Some(repo_url) = &self.submodule {
            println!("{} {}", "克隆仓库:".blue().bold(), repo_url);
            
            // 检查是否已存在同名子模块
            let course_dir = exercises_dir.join(&self.course);
            if course_dir.exists() {
                println!("{} {}", "警告:".yellow().bold(), format!("目录 {} 已存在，将被覆盖", course_dir.display()));
                fs::remove_dir_all(&course_dir)
                    .context(format!("删除已存在的目录 {} 失败", course_dir.display()))?;
            }
            
            // 使用git命令添加子模块
            let status = Command::new("git")
                .args(["submodule", "add", "-f", repo_url, &format!("exercises/{}", self.course)])
                .status()
                .context("执行git submodule add命令失败")?;
            
            if !status.success() {
                return Err(anyhow::anyhow!("git submodule add命令执行失败"));
            }
            
            println!("{} {}", "成功配置课程:".green().bold(), self.course);
            println!("{} {}", "练习已克隆到:".green(), format!("exercises/{}", self.course));
            println!("{}", "你现在可以使用 'cargo xtask eval' 命令来评测练习".blue());
        } else {
            println!("{}", "未提供仓库地址，请使用 --submodule 参数指定仓库地址".yellow());
        }
        
        Ok(())
    }
}
