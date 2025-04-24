#[derive(Args)]
pub struct SetupArgs {
    /// 配置的环境名称
    env: String,
}

impl SetupArgs {
    pub fn setup(self) {}
}
