mod eval;
mod learn;
mod setup;

use clap::Parser;
use eval::EvalArgs;
use learn::LearnArgs;
use setup::SetupArgs;

#[macro_use]
extern crate clap;

fn main() {
    use Commands::*;
    match Cli::parse().command {
        Setup(args) => args.setup(),
        Learn(args) => args.learn(),
        Eval(args) => args.eval(),
    }
}

#[derive(Parser)]
#[clap(name = "learning-in-camp")]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 安装指定开发环境
    Setup(SetupArgs),
    /// 配置指定课程仓库
    Learn(LearnArgs),
    /// 评分
    Eval(EvalArgs),
}
