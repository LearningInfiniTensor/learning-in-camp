#[derive(Args)]
pub struct LearnArgs {
    /// 课程名称
    course: String,
    /// 传入 fork 仓库地址，以 git submodule 方式配置
    #[clap(long)]
    submodule: Option<String>,
}

impl LearnArgs {
    pub fn learn(self) {}
}
