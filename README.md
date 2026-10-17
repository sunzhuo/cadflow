# CADFlow - 基于 WebAssembly 的 CAD 建模与 Markdown 文档协同平台

CADFlow 是一个复用 CADCraft 核心底层引擎（`cadcraft-engine`、`cadcraft-ui-egui`、`cadcraft-io`、`cadcraft-render` 等）、针对工程设计协同深度定制的 WebAssembly CAD 应用。

## 🌟 核心特性与定制改进

1. **全中文界面与字体支持**：
   - 默认启用简体中文界面（菜单、面板、状态栏、快捷提示全汉化）。
   - 内置高质量中文字体（华文细黑 / 黑体）动态加载支持，彻底解决 egui 在 WebAssembly 环境下的中文乱码/方块（Tofu）问题。

2. **定制化启动界面布局**：
   - **左上部分**：`Layers`（图层）面板与 `Tool Sets`（工具集）面板以标签页形式组合在左上方，便于图层与绘图工具间快速切换。
   - **左下部分**：`Properties`（特性/属性）面板独占左下方，可便捷查看与修改图元参数。
   - **中间区域**：三维 CAD 绘图视口（支持 WebGPU / WebGL2 高性能硬件加速与命令行）。
   - **右侧区域**：新增全功能 **文档面板 (Document Panel)**，启动时显示在右侧，**无尺寸限制**（可自由调整宽度至全屏或收起）。

3. **内置 Markdown 文档编辑器**：
   - **三模式视图**：支持【编辑】、【预览】与【双栏实时联动】模式。
   - **快捷排版工具条**：一键插入标题（H1-H3）、粗体、斜体、引用块、代码块、列表、任务待办清单、数据表格、分割线等。
   - **本地交互与导出**：支持一键复制全文、快速重置为工程设计规范模板，以及将文档保存导出为本地 `.md` 文件。

4. **符合 Cloudflare Pages 的工程架构**：
   - 结构完整的 `dist/` 静态资产部署目录，配备专用的 `_headers` 安全头策略（COOP/COEP、wasm MIME 类型、静态缓存策略）。
   - 配备 `wrangler.toml` 与 `package.json`，支持一键本地模拟调试与部署。

---

## 📁 目录结构

```
cadflow/
├── cadcraft/                     # CADCraft 官方子模块（保持 100% 纯净，更新无冲突）
│   ├── crates/                   # engine, io, geom, dxf, ui-egui 等底层库
│   └── ...
├── cadflow-web/                  # 【CADFlow 独立定制前端应用包】
│   ├── Cargo.toml                # 独立包定义（依赖 cadcraft crates）
│   ├── Trunk.toml                # Trunk WebAssembly 构建配置
│   ├── index.html                # 定制中文页面与加载器
│   ├── _headers                  # Cloudflare Pages 安全头配置
│   ├── assets/                   # 中文字体 (font.ttf) 静态资源
│   └── src/
│       ├── main.rs               # 主应用入口、中文初始化、启动停靠布局配置
│       ├── doc_panel.rs          # 独立 Markdown 文档面板（无尺寸限制、全功能编辑与导出）
│       ├── download.rs           # 浏览器本地文件下载支持
│       ├── upload.rs             # 浏览器文件读取与拖拽支持
│       └── host.rs               # 外部 iframe 容器通信桥接
├── dist/                         # 【Cloudflare Pages 发布产物目录】
│   ├── assets/                   # 中文字体 (font.ttf) 与静态资源
│   ├── index.html                # 启动页面
│   ├── cadflow-web-*.js          # WASM 加载与胶水脚本
│   ├── cadflow-web-*_bg.wasm     # Release 优化的 WebAssembly 二进制
│   └── _headers                  # Cloudflare Pages 响应头配置文件
├── Cargo.toml                    # 顶层 Workspace 配置（exclude = ["cadcraft"]）
├── package.json                  # NPM 脚本与 Wrangler 依赖定义
├── wrangler.toml                 # Cloudflare Pages 配置文件
├── deploy.ps1                    # 一键部署脚本
└── README.md                     # 项目说明文档
```

---

## 🚀 本地开发与测试

### 1. 本地启动 Cloudflare Pages 模拟环境
```bash
# 使用 Wrangler 本地启动 Pages 模拟服务器
npx wrangler pages dev dist --port 8788
```
浏览器打开 `http://127.0.0.1:8788` 即可体验。

### 2. 重新编译 WebAssembly
```bash
# 调试构建
npm run build:dev

# 生产优化构建
npm run build
```

---

## ☁️ 通过 Cloudflare Pages 发布

### 方法一：使用 Wrangler CLI 命令行发布（推荐）

1. **登录 Cloudflare 账号**（仅首次需要）：
   ```bash
   npx wrangler login
   ```
2. **一键发布**：
   ```bash
   # 直接运行部署脚本
   .\deploy.ps1

   # 或通过 npm 脚本发布
   npm run deploy
   ```

> **提示**：若在 CI/CD 或自动化流水线中，可直接设置环境变量：
> ```powershell
> $env:CLOUDFLARE_API_TOKEN = "<你的 API Token>"
> npm run deploy
> ```

### 方法二：在 Cloudflare Dashboard 网页端导入

1. 打开 [Cloudflare Dashboard](https://dash.cloudflare.com/) -> **Workers & Pages** -> **Create application** -> **Pages**。
2. 选择 **Upload Assets**（直接上传）或绑定本 Git 仓库。
3. 若直接上传资产，将本地的 `dist` 文件夹上传即可。
4. 若绑定 Git 仓库构建：
   - **Framework preset**: None
   - **Build command**: `cd cadcraft/apps/cadcraft-web && trunk build --release`
   - **Build output directory**: `dist`

