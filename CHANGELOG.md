# 更新日志

[简体中文](CHANGELOG.md) | [English](CHANGELOG_EN.md)

本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### 新增

- **富文本超链接**：`<a href="https://…">文本</a>` 编译为 run 级 `<a:hlinkClick>` + `slideN.xml.rels` 外部关系（`TargetMode="External"`，同一 URL 去重）；`pptd convert` 还原为同样的行内标记，元素级 `link` 字段也完整回编
- **字体名默认透传**：`pptd build` 默认原样写出 PPTD 中的字体名；Kimi 网页字体到系统字体的安全映射改为 `safeFonts: true` 显式开关（`pptd convert` 产出的工程默认关闭，保证回编排版不变）

## [1.4.1] - 2026-10-19

### 新增

- **多行对齐公式**：`\begin{align}`、`align*`、`aligned`、`gather`、`gathered`、`split`、`eqnarray` 编译为原生 OMML 方程数组（`<m:eqArr>`，每行一个 `<m:e>`，`&` 对齐点写成 `&` 文本 run，与 Word 线性格式及微软 `MML2OMML.XSL` 一致）；`align`/`gather`/`eqnarray` 为显示式环境，必须独占段落，混排时明确报错
- **公式内宏定义**：`\newcommand`（含 `[n]` 与 `[n][default]`）、`\renewcommand`、`\providecommand`、`\def`（含 `#1#2` 参数）在解析前展开；宏作用域为单个公式，递归定义触发展开上限错误；`pptd check` 的 LaTeX 诊断改为先展开再检查，用户自定义宏不再误报
- `pptd convert` 反向编译 `<m:eqArr>` 为 `\begin{aligned} … \end{aligned}`，保留 `&` 对齐点

### 修复

- **花括号分组破坏公式结构**：`{\frac{a}{b}} + c` 过去被压平成文本 `a/b+c`，现在保留完整分式结构（`{x + \frac{a}{b}}` 同理）
- **大运算符贪婪吞噬**：`\sum_{i=1}^{n} A + \sum_{j=1}^{m} B` 过去把第二个求和嵌套进第一个求和的被操作数，现在按 TeX 语义只取紧随其后的一个原子
- 幻灯片根节点补上 `mc:Ignorable="a14"`，不理解 `a14` 数学扩展的阅读器会按 OOXML 规则跳过，而不是判定文件损坏

## [1.4.0] - 2026-10-19

### 新增

- `<latex>…</latex>` 与行内 `\(…\)` 编译为**原生 OOXML 数学公式**（`<a14:m>` / OMML）：真实分数线（`\frac`）、根号（`\sqrt`、`\sqrt[n]{}`）、上下标、求和/积分/极限等大运算符、矩阵与 `\left…\right` 定界符；单段公式编译为居中显示公式，混合段落内的行内公式额外携带 `mc:Fallback` 文本降级，保证在不支持 `a14` 数学扩展的阅读器（如 LibreOffice）中也不会丢失
- `pptd convert` 反向编译：原生公式还原为 `<latex>` 源码
- `letterSpacing` 与 `textDirection` 真正写入 OOXML（`a:rPr@spc` 与 `a:bodyPr@vert`）
- `pptd check` 新增诊断：形状图片填充、元素越界画布、动画声明警告、不支持的 LaTeX 命令、图标前缀与未知图标
- `validate_pptx.py --render-images` 同时输出逐页 `slide-NN.png` 与总览 `contact-sheet.jpg`
- README 新增「暂不支持与能力边界」章节

### 修复

- 安装器现在比较版本：PATH 上存在旧版 `pptd`（或历史 `slideforge`）时会自动升级到目标版本，不再需要 `--force`
- `.sync.hash`：工程内输出记录为相对路径，同一路径只保留一条记录，工程外路径不再产生 `.//…` 拼接；「被外部修改」与「无同步点」提示区分开
- 构建可复现：主题 major/minor 字体不再随哈希表遍历顺序变化
- 表格边框：未显式设置的边不再写出 `<a:lnX><a:noFill/>`，避免盖掉相邻行的下边线
- `validate_pptx.py --strict` 的状态行与退出码一致（PASS / WARNINGS / FAIL）；标题识别不再误取每页第一个文本框
- `pptd build` 不再因警告拒绝构建（动画、越界等仅为警告），仅在存在错误时拒绝
- Node CLI 的 `build` 正确转发到原生二进制；`package.json` 不再暴露 `pptd` bin，避免全局安装遮蔽原生编译器
- 修正 `reference/cli.md` 中 404 的 cargo 安装地址与相对路径安装命令

### 变更

- 图标仅内置 Font Awesome Solid（`fas:`）字形；`far:`/`fab:` 前缀与未知图标名改为 `pptd check` 明确报错，不再静默忽略前缀
- 仓库瘦身：`docs/` 83MB → 14MB、`example/` 42MB → 17MB（移除可重新生成的示例产物）；主题预览图随 skill 分发（`skills/open-pptd/theme_previews/`，约 2.5MB），修复安装后 theme.md 的预览链接失效
- README 增加 `npx github:shiquda/open-pptd-skill install -y` 作为整仓克隆超时时的备用安装方式
- 时序动画仍只解析不写入（`check` 会给出警告）

## [1.2.0] - 2026-08-06

### 新增

- CLI 支持 `-h` / `--help` 与 `-V` / `--version`

### 变更

- npm 包名与 CLI 由 `open-kimi-ppt-skills` 统一为 `open-kimi-ppt-skill`（与 GitHub 仓库名一致）。请改用 `npx open-kimi-ppt-skill@latest`；旧包名将不再更新

## [1.1.3] - 2026-08-06

### 新增

- `install` 在交互终端下列出 `.agents` / `.codex` / `.claude` / `.cursor` / `.workbuddy` 技能目录，支持空格多选
- 支持 `-y/--yes`（非交互默认目录）、可重复的 `--target`
- `--all` 仅装到已检测到的 Agent 目录；对应目录不存在的 Agent 会跳过并提示，不会新建目录
- Windows 导出自动启动一个常驻调试浏览器（Chrome，未安装时回退 Edge），规避 agent-browser 无法自行拉起 Chrome 导致的导出失败；实例导出后常驻、后续导出复用同一个，也可用 `AGENT_BROWSER_CDP` 指定自己启动的调试浏览器

### 修复

- 修复 `find_download` 在 Downloads 目录扫描时因 `.crdownload` 被 Chrome 重命名而触发 `FileNotFoundError` 导致导出中断（Related to #4）

### 变更

- README 默认指引 AI 通过 `npx open-kimi-ppt-skill@latest install -y` 安装，不再建议 clone 仓库

## [1.1.2] - 2026-08-06

### 修复

- 修复中文 Windows 下 `export_pptx.py` / `export_images.py` 导出卡住或 GBK 解码失败（stdout 改走临时文件 + UTF-8）
- 规避 agent-browser `--download-path` 导致 Chrome 静默取消下载：改为点击下载并轮询默认 Downloads
- 修复 npm 包误打入 `__pycache__/*.pyc`（`files` 仅列出脚本源文件）

## [1.1.1] - 2026-08-06

### 新增

- PPTD 对齐官方元素级 `animations` 规范；Skill 补充动画 / `notes` 使用边界
- 对齐配图优先级、文风禁令、澄清提问、复刻细则、并行写页等官方策略
- 内置约 30 套 preset design system，并支持点名使用
- 恢复 `customFonts`（Google Fonts）与海报推荐尺寸
- 根目录主题目录：`theme.md` / `theme_EN.md`（含预览图）
- 示例项目 `example/xiaomi-yu7-ppt-animation`（页内元素入场动画）

### 变更

- 场景文档同步官方动画细则与 `customFonts` 引用
- README 补充元素动画、预设主题说明与示例 Prompt

## [1.0.2] - 2026-08-06

### 变更

- `install` 默认直接覆盖已安装的 Skill，无需再加 `--force`（旧的 `--force` 仍可兼容传入）

## [1.0.1] - 2026-08-06

### 新增

- Skill 工作流增加 **step0 本地前置检测**：生成前检查 Node.js 18+、npm/npx、python3，并提示需要 Chromium 系浏览器
- 导出脚本在启动时检测 **Node.js 18+** 与 **npm**；缺失或版本过低时给出明确安装指引
- CLI（`open-kimi-ppt-skill`）启动时校验 Node.js 主版本 ≥ 18
- 缺失 **PyYAML** 时自动 `pip install --user pyyaml`（与 Pillow / websocket-client 行为一致）

### 文档

- 补充多 Agent / 多模型实测截图（ChatGPT·Codex + 5.6 Luna、Reasonix + DeepSeek、WorkBuddy 等）
- 安装说明改为「自动 / 手动二选一」，并补充 Windows 路径说明
- README 结构与示例图更新

## [1.0.0] - 2026-08-05

### 新增

- 首次发布 `open-kimi-ppt-skill`
- PPTD 生成 / 编辑 / 复刻，默认同时交付可编辑 PPTD 项目与 PPTX 成品
- 浏览器侧导出 PPTX（嵌字体、淡入淡出切换），导出前可选多模态视觉质检
- 本地在线 PPTD 编辑器（`npx open-kimi-ppt-skill serve`）
- CLI 安装 Skill 到 `~/.agents/skills`（可用 `--target` 指定其他 Agent 目录）
