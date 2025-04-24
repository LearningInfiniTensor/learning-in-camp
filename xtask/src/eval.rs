#[derive(Args)]
pub struct EvalArgs {
    /// 要评分的课程名称，不传则自动对所有已配置课程评分
    #[clap(long)]
    course: Option<String>,
}

impl EvalArgs {
    pub fn eval(self) {}
}
