# learning in camp

[![CI](https://github.com/LearningInfiniTensor/learning-in-camp/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/LearningInfiniTensor/learning-in-camp/actions)
[![license](https://img.shields.io/github/license/LearningInfiniTensor/learning-in-camp)](https://mit-license.org/)
![GitHub repo size](https://img.shields.io/github/repo-size/LearningInfiniTensor/learning-in-camp)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/LearningInfiniTensor/learning-in-camp)

[![GitHub Issues](https://img.shields.io/github/issues/LearningInfiniTensor/learning-in-camp)](https://github.com/LearningInfiniTensor/learning-in-camp/issues)
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/LearningInfiniTensor/learning-in-camp)](https://github.com/LearningInfiniTensor/learning-in-camp/pulls)
![GitHub contributors](https://img.shields.io/github/contributors/LearningInfiniTensor/learning-in-camp)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/LearningInfiniTensor/learning-in-camp)

Opencamp 训练营通用学习工具。

## 环境准备

- cargo；
- git；

## 功能

```shell
cargo xtask help
```

```plaintext
Usage: xtask <COMMAND>

Commands:
  setup  安装指定开发环境
  learn  配置指定课程仓库
  eval   评分
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
### 配置课程
```bash
# 配置 learning-lm-rs 课程
cargo xtask learn learning-lm-rs --submodule https://github.com/LearningInfiniTensor/learning-lm-rs.git

# 配置 rustlings 课程
cargo xtask learn rustlings --submodule https://github.com/rust-lang/rustlings.git

cargo xtask learn learning-cxx --submodule

# 评测 learning-lm-rs 课程
cargo xtask eval --course learning-lm-rs --path exercises/learning-lm-rs

# 评测 rustlings 课程 
cargo xtask eval --course rustlings --path exercises/rustlings

# 评测 learning-cxx 课程
cargo xtask eval --course learning-cxx

```
