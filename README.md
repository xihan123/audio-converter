# 🎵 Audio Converter

[![Build Release](https://github.com/xihan123/audio-converter/actions/workflows/build.yml/badge.svg)](https://github.com/xihan123/audio-converter/actions/workflows/build.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

一个基于 Rust 和 egui 构建的现代化音频格式转换工具，具有友好的图形用户界面。

特定需求场景：需要将非WAV格式的音频文件转换为WAV格式，并且要求转换后的WAV文件时长不小于指定值（如30秒）。如果原始音频文件时长不足，则通过重复拼接音频内容来满足最小时长要求。

## ✨ 功能特性

- 🎨 **现代化界面**: 基于 egui 框架的原生图形界面，支持中文显示
- 🎵 **多格式支持**: 支持常见音频格式（MP3, WAV, FLAC, AAC, OGG 等）
- 📁 **批量处理**: 支持同时处理多个音频文件
- 🚀 **高性能**: 基于 FFmpeg 引擎，转换速度快
- 💾 **体积小巧**: 使用精简版 FFmpeg，大幅减小程序体积
- 🔧 **灵活配置**: 支持设置最小时长等参数
- 📊 **实时进度**: 显示每个文件的转换进度和状态

## 📦 下载安装

### 方式一：下载预编译版本（推荐）

前往 [Releases](https://github.com/xihan123/audio-converter/releases/latest) 页面下载最新版本：

1. 下载 `audio-converter-windows-x86_64.zip`
2. 解压到任意目录
3. 双击运行 `audio-converter.exe`

### 方式二：从源码编译

#### 前置要求

- [Rust](https://www.rust-lang.org/) 1.70 或更高版本
- Windows 10/11 操作系统

#### 编译步骤

```powershell
# 克隆仓库
git clone https://github.com/xihan123/audio-converter.git
cd audio-converter

# 下载 FFmpeg 精简版（需要手动下载）
# 从 https://github.com/xihan123/FFmpeg-Audio/releases/latest 下载
# 下载 ffmpeg-audio-only-8.0-windows-x64.zip
# 解压后，将 bin 目录下的 ffmpeg.exe 和 ffprobe.exe 复制到项目根目录

# 编译发布版本
cargo build --release

# 运行程序
.\target\release\audio-converter.exe
```

## 🚀 使用方法

### 基本操作

1. **启动程序**: 双击 `audio-converter.exe`
2. **选择文件**:
   - 点击"选择输入文件夹"选择源音频文件目录
   - 点击"选择输出文件夹"选择转换后文件的保存位置
3. **配置参数**:
   - 设置最小时长
   - 并发数(同时处理文件数)
4. **开始转换**: 点击"开始转换"按钮
5. **查看进度**: 在任务列表中查看每个文件的转换状态

### 高级功能

- **拖放支持**: 支持将文件夹拖放到窗口中
- **批量转换**: 自动遍历输入目录下的所有音频文件
- **进度显示**: 实时显示每个文件的转换进度百分比
- **错误处理**: 转换失败时显示详细错误信息

## 🏗️ 项目结构

```plaintext
audio-converter/
├── src/
│   ├── main.rs           # 主程序入口，GUI 界面
│   ├── converter.rs      # 音频转换核心逻辑
│   ├── file_handler.rs   # 文件处理工具
│   └── types.rs          # 数据类型定义
├── .github/
│   └── workflows/
│       └── build.yml     # 自动构建工作流
├── Cargo.toml            # Rust 项目配置
├── SimHei.ttf            # 中文字体文件
├── ffmpeg.exe            # FFmpeg 可执行文件（需下载）
├── ffprobe.exe           # FFprobe 可执行文件（需下载）
└── README.md             # 本文件
```

## 🔧 技术栈

- **语言**: Rust (Edition 2024)
- **GUI 框架**: [egui](https://github.com/emilk/egui) 0.28
- **异步运行时**: [tokio](https://tokio.rs/) 1.47
- **音频处理**: FFmpeg (来自 [FFmpeg-Audio](https://github.com/xihan123/FFmpeg-Audio))
- **文件对话框**: [rfd](https://github.com/PolyMeilex/rfd) 0.11

### 核心依赖

```toml
[dependencies]
eframe = "0.28"          # egui 应用框架
egui = "0.28"            # 即时模式 GUI
tokio = "1.47"           # 异步运行时
anyhow = "1.0"           # 错误处理
walkdir = "2.5"          # 目录遍历
egui_extras = "0.28"     # egui 扩展
rfd = "0.11"             # 文件对话框
tempfile = "3.8"         # 临时文件处理
```

## 📝 开发说明

### 本地开发

```powershell
# 克隆项目
git clone https://github.com/xihan123/audio-converter.git
cd audio-converter

# 安装依赖（自动）
cargo build

# 运行开发版本
cargo run

# 运行优化版本
cargo run --release
```

### 代码风格

```powershell
# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 运行测试
cargo test
```

### 构建发布版本

```powershell
# 构建优化版本
cargo build --release

# 输出位置
# .\target\release\audio-converter.exe
```

发布版本会启用以下优化：

- 最大优化级别 (`opt-level = 3`)
- 链接时优化 (`lto = true`)
- 单个代码生成单元 (`codegen-units = 1`)
- 符号剥离 (`strip = true`)

## 🌟 关于 FFmpeg-Audio

本项目使用 [xihan123/FFmpeg-Audio](https://github.com/xihan123/FFmpeg-Audio) 提供的精简版 FFmpeg。

### 特点

- ✅ **体积小巧**: 相比完整版 FFmpeg 大幅减小体积
- ✅ **音频专用**: 专注于音频处理功能
- ✅ **定期更新**: 跟随 FFmpeg 主线版本更新
- ✅ **自动集成**: CI/CD 流程自动下载最新版本

### 手动更新 FFmpeg

如需手动更新 FFmpeg：

1. 访问 <https://github.com/xihan123/FFmpeg-Audio/releases/latest>
2. 下载 `ffmpeg-audio-only-<version>-windows-x64.zip` （例如：`ffmpeg-audio-only-8.0-windows-x64.zip`）
3. 解压压缩包，找到 `bin` 目录
4. 将 `bin` 目录下的 `ffmpeg.exe` 和 `ffprobe.exe` 复制到项目根目录
5. 重新编译项目

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [egui](https://github.com/emilk/egui) - 优秀的即时模式 GUI 框架
- [FFmpeg](https://ffmpeg.org/) - 强大的音视频处理工具
- [xihan123/FFmpeg-Audio](https://github.com/xihan123/FFmpeg-Audio) - 精简版 FFmpeg
- Rust 社区的所有贡献者

---

Made with ❤️ using Rust and egui
