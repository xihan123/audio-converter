# 音频转 WAV 格式工具

这是一个基于 Vue 3 + TypeScript 开发的音频转换工具，可以将各种格式的音频文件转换为 WAV 格式。

## ✨ 功能特点

- 🎵 支持多种音频格式输入（MP3、M4A、OGG、FLAC 等）
- 📁 支持拖入文件或文件夹批量处理
- ⏱️ 自动检测音频时长，不足 30 秒自动重复拼接
- 📊 实时显示每个文件的转换进度
- 💾 转换完成后可直接下载
- 📦 批量下载 - 将多个转换后的文件打包为 ZIP 下载
- 🎨 美观的用户界面
- ⚡ 纯前端处理，无需上传服务器

## 🚀 快速开始

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run dev
```

### 构建生产版本

```bash
npm run build
```

### 预览生产版本

```bash
npm run preview
```

## 📖 使用说明

1. **拖入文件**：直接将音频文件或包含音频文件的文件夹拖入页面的拖放区域
2. **选择文件**：点击"选择文件"按钮，从本地选择音频文件
3. **查看进度**：在文件列表中查看每个文件的转换进度
4. **下载文件**：转换完成后，点击"下载"按钮获取单个 WAV 文件
5. **批量下载**：点击"批量下载"按钮，将所有已完成的文件打包为 ZIP 下载
6. **管理文件**：可以随时删除单个文件或清除所有已完成的文件

## 🛠️ 技术栈

- **Vue 3** - 渐进式 JavaScript 框架
- **TypeScript** - JavaScript 的超集，提供类型安全
- **Vite** - 下一代前端构建工具
- **Web Audio API** - 浏览器原生音频处理 API
- **JSZip** - 用于创建 ZIP 压缩文件

## 📝 核心功能说明

### 音频格式转换

使用 Web Audio API 的 `AudioContext` 和 `decodeAudioData` 方法解码各种格式的音频文件，然后将解码后的 `AudioBuffer` 转换为 WAV 格式的 PCM 数据。

### 时长检测与拼接

- 自动检测音频文件时长
- 如果时长小于 30 秒，计算需要重复的次数
- 将原始音频重复拼接，确保最终时长至少达到 30 秒

### 批量处理

- 支持同时处理多个文件
- 每个文件独立显示转换进度
- 异步处理，不阻塞用户界面

### 批量下载

- 使用 JSZip 将多个 WAV 文件打包为 ZIP
- 自动生成带时间戳的 ZIP 文件名
- 实时显示打包进度
- 一键下载所有已完成的文件

### 文件夹遍历

- 递归遍历文件夹中的所有音频文件
- 自动过滤非音频文件
- 支持深层嵌套的文件夹结构

## 🔧 项目结构

```
audio-to-wav/
├── src/
│   ├── components/
│   │   └── AudioConverter.vue    # 文件转换进度组件
│   ├── utils/
│   │   └── audioProcessor.ts     # 音频处理核心逻辑
│   ├── App.vue                    # 主应用组件
│   ├── main.ts                    # 应用入口
│   ├── style.css                  # 全局样式
│   └── vite-env.d.ts             # TypeScript 类型声明
├── index.html                     # HTML 入口
├── package.json                   # 项目配置
├── tsconfig.json                  # TypeScript 配置
├── vite.config.ts                # Vite 配置
└── README.md                      # 项目说明
```

## 🌟 浏览器兼容性

- Chrome 60+
- Firefox 55+
- Safari 11+
- Edge 79+

需要支持以下 Web API：

- Web Audio API
- File API
- Drag and Drop API
- Blob API
