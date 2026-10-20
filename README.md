# Open PPTD (`open-pptd`)

[简体中文](README.md) | [English](README_EN.md)

[![GitHub Release](https://img.shields.io/github/v/release/shiquda/open-pptd-skill?label=release)](https://github.com/shiquda/open-pptd-skill/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/shiquda/open-pptd-skill/actions/workflows/ci.yml/badge.svg)](https://github.com/shiquda/open-pptd-skill/actions/workflows/ci.yml)
[![Node >= 18](https://img.shields.io/badge/node-%3E%3D18-brightgreen)](https://nodejs.org)
[![Rust >= 1.85](https://img.shields.io/badge/rust-%3E%3D1.85-orange)](https://www.rust-lang.org)

**Open PPTD** 是面向 AI Coding Agent（Claude Code、Cursor、Codex、WorkBuddy、Windsurf、Multica 等）与开发者的开源声明式幻灯片套件。基于开放的 **PPTD**（Declarative Presentation Language）YAML 规范，提供 **100% 本地离线、毫秒级响应** 的原生 Rust 编译器（`pptd` CLI）与开箱即用的 Agent 技能。

彻底剔除对网页端会话、无头浏览器、Cookie 凭证与云端接口的脆弱依赖，在本地毫秒级生成真实可编辑、带矢量图形与数学公式的原生 `.pptx` 成品及双向工程。

---

## ⚡ 快速安装与就绪（首选且唯一推荐）

> [!TIP]
> **无需人工折腾环境与配置！** 我们**首选且仅推荐**直接让您的 AI Coding Agent 安装本 Skill，并由 Agent 根据指引全自动完成本地 `pptd` 原生编译器的就绪与环境验证。

### 📋 复制下方指令发给您的 Agent 即可：

```text
请帮我安装并配置 open-pptd 幻灯片制作技能：
1. 运行命令安装 Skill（若网络克隆慢或超时，推荐使用备用兼容命令）：
   npx skills add https://github.com/shiquda/open-pptd-skill -g -y
   # 备用兼容命令（跳过全量 Git 历史克隆）：
   # npx github:shiquda/open-pptd-skill install -y
2. 运行技能内置的安装脚本以就绪本地原生 `pptd` 编译器（可直接使用绝对路径或先 cd 到技能目录）：
   python3 ~/.agents/skills/open-pptd/scripts/install_binary.py
3. 验证编译器：终端执行 `pptd --version`；若提示 command not found，请执行 `export PATH="$HOME/.local/bin:$PATH"`；
4. 配置完成后向我汇报已就绪。
```

> **安装命令说明**：
> - **标准生态命令**：使用技能包管理器 `npx skills add https://github.com/shiquda/open-pptd-skill -g -y`（或 `npx skills add shiquda/open-pptd-skill -g -y`）。该命令会做整仓克隆：完整 Git 历史约 110MB（当前工作树约 31MB），若在受限网络下克隆超时，请改用下方备用命令。
> - **npm/npx 兼容命令（推荐备用）**：使用 `npx github:shiquda/open-pptd-skill install -y` 直接分发安装，避免全量 Git Clone 超时。
---

## 💬 让 Agent 制作 PPT（日常使用）

配置就绪后，直接向 Agent 描述你的主题与诉求。**Agent 默认将同时交付两份产物**：可二次代码编辑的 PPTD 源码工程目录，以及离线极速编译出的原生 `.pptx` 成品文件。

### 📋 复制发送给 Agent 制作文稿：

```text
使用 open-pptd 为我制作一份关于「分布式架构演进」的 8 页商业技术演讲 PPT：
- 风格主题参考：深色金融科技风（ebony-ledger）或深蓝学术风（deep-blue-atlas）
- 核心内容要求：
  1. 包含封面、架构演进痛点、微服务架构蓝图（必须包含带箭头的连接线与矢量容器框）、核心技术指标对比表格；
  2. 每页均需包含完整的演讲者备注（notes: "..."），供演练参考；
  3. 严格使用 LaTeX 原生数学公式呈现核心算法；
- 交付标准：生成完整的 PPTD 工程及编译好的 PPTX 文件，并在完成后运行 validate_pptx.py 质检通过。
```

---

## 🖼️ 作品展示 (Showcase)

Open PPTD 坚持 **「PPTD 源码工程 + 原生可编辑 PPTX」** 双交付路线。无论是商业路演、前沿技术汇报还是学术研讨，均能稳定产出大师级排版：

| 小米 YU7 商业级发布会演示（图文排版） | DJI Pocket 4 产品发布演示（全景视觉） |
| :---: | :---: |
| [![小米 YU7 导出效果](docs/images/example-yu7-pptx.png)](docs/images/example-yu7-pptx.png) | [![DJI Pocket 4 导出效果](docs/images/example-dji-pocket4-pptx.png)](docs/images/example-dji-pocket4-pptx.png) |

| DeepSeek Liquid Glass 风格演示 | 极简 Starter 标杆模版（表格/连线/备注） |
| :---: | :---: |
| [![DeepSeek Liquid Glass 风格](docs/images/example-deepseek-liquid-glass.png)](docs/images/example-deepseek-liquid-glass.png) | [![Starter 标杆模版](example/starter/starter_expected.png)](example/starter/starter_expected.png) |

---

## 💡 为什么选择 Open PPTD？

目前常见的 AI 幻灯片方案大多存在明显痛点：直接调用代码库拼接坐标容易错位，整页画图模式文字无法二次编辑，而网页 HTML 幻灯片则非正式商务与学术通用的 PPTX。

| 评估维度 | **Open PPTD** | 代码直接拼写（如 pptxgenjs） | 整页出图型 PPT | 网页 HTML 幻灯片 |
|---|---|---|---|---|
| **默认交付物** | **PPTD 源码工程 + 原生 PPTX** | 通常仅 PPTX 文件 | 单张位图嵌入 PPTX | 单文件 HTML |
| **Agent 编写体验** | **YAML 声明式描述**，语义清晰 | 坐标与细节繁杂，极易翻车 | 依赖提示词与文生图模型 | HTML/CSS 标签约束重 |
| **PowerPoint 二次编辑** | **100% 矢量无损**，文字图形可改 | 结构脆弱，二次调整易错位 | 整页图片，无法改字 | 并非真实 PPTX 文件 |
| **构建性能与依赖** | **<0.05 秒**（Rust 原生离线执行） | 1 ~ 5 秒（Node 运行时） | 10 ~ 30 秒（出图排队） | 需浏览器环境播放 |
| **数学公式与排版** | **原生 OOXML 数学公式**（`<latex>` / `\(…\)` 转真实可编辑分数线/根号/大运算符，非法命令硬拦截） | 不支持或需外部插件 | 依赖图片生成 | MathJax / KaTeX 渲染 |
| **安全与隐私** | **零网络调用**，内网离线沙箱安全 | 本地执行 | 需将文本上传云端大模型出图 | 通常依赖 Web 渲染 |

---

## 📋 功能特性矩阵

- 📐 **工业级 DrawingML 矢量形状**：原生支持 `rect`, `roundRect`, `ellipse`, `triangle`, `diamond` 等形状，支持精确描边与阴影。
- 🔗 **直线连接器与箭头系统**：支持相对矢量坐标 `points: "0,0 dx,dy"` 及双向箭头定义（`triangle`, `stealth` 等）。
- 🔢 **原生 OOXML 数学公式 (`<a14:m>`)**：`<latex>…</latex>` 与行内 `\(…\)` 编译为真实可编辑的 Office Math 公式，支持真实分数线 (`\frac`)、根号 (`\sqrt`)、上下标、求和积分大运算符 (`\sum`, `\int`)、矩阵、多行对齐公式（`\begin{align}`/`aligned`/`gather`/`split` → 原生方程数组 `<m:eqArr>`）与公式内宏定义（`\newcommand`/`\def`，展开后再解析）；单段公式独占时编译为居中显示公式，混合段落内联公式额外携带 `mc:Fallback` 文本降级（PowerPoint/WPS 显示可编辑公式，不识别 `a14` 数学扩展的阅读器如 LibreOffice 显示等价的 Unicode 文本，绝不静默丢失）；不支持的 LaTeX 命令硬报错拦截，杜绝静默损坏。
- 📊 **原生双向表格支持**：支持单元格合并（`rowSpan`/`colSpan`）、列宽比例定制、斑马纹与边框，支持反编译回 PPTD 代码。
- 🗣️ **PowerPoint 演讲者备注**：页面级 `notes: "..."` 属性自动关联生成标准 OOXML `notesSlide` 关系链。
- 🎨 **1422 款矢量图标内置**：内嵌 Font Awesome 6.7.2 Solid 图标库，支持 `fa-` 自动前缀容错与静态校验。
- 🔒 **`.sync.hash` 双向同步防覆写**：编译器维护校验哈希，防止外部编辑与代码构建相互意外覆盖。
- 🔤 **跨平台安全字体回退**：中文字体智能安全映射至预装字体（如思源宋体安全映射 `SimSun` + `Georgia`），杜绝草书翩翩体降级。


---

## ⚠️ 暂不支持与能力边界 (Current Boundaries)

为了确保幻灯片编译的 100% 确定性与诚实交付，以下特性当前版本暂不支持，使用时会被静态校验或编译器拦截，请遵循对应替代方案：
- 📊 **原生图表透传，声明式图表语法未实现**：`pptd convert` 产出的原生图表（`chartPart` + 可选 `workbookPart`）会被完整回写，图表 XML 与内嵌工作簿按原字节保留；手写的声明式 `data`/`series` 图表语法仍会被 `check`/`build` 拦截。新建图表请使用原生表格 (`elementType: table`)，或用外部脚本导出图片后通过 `elementType: image` 引入。
- 🎬 **时序动画 (`animations`)**：语法解析已支持（`animations` 数组），但当前版本生成的 OOXML 暂缓写入 `p:timing` 时序动画（🟡 仅解析不写入），文稿以完整排版的纯静态页面呈现。
- 🌐 **HTML 扩展与文档级标签**：
  - 不支持 HTML 列表标签（`<ul>`, `<ol>`, `<li>`）：富文本仅支持常规段落，列表请使用常规段落配合标号或符号（如 `1. 第一点` 或 `• 第一点`）。
  - 支持 HTML 超链接标签 `<a href="https://…">文本</a>`（编译为 run 级点击目标 + 外部关系）；裸 URL 可直接以纯文本书写。
  - 支持 HTML 上标/下标标签 `<sup>` / `<sub>`（编译为 run 级 `a:rPr@baseline`；需要非标准偏移时用 `<span style="baseline:20000">`），复杂排版仍可用原生 LaTeX 数学公式（如 `<latex>x^2</latex>`、`\( \sigma_i \)`）。
  - 不支持 HTML 标题、表格与排版标签（`<h1>`~`<h6>`, `<table>`, `<div>`, `<img>` 等）：页面结构必须使用 PPTD 顶层声明的元素（如 `table`, `image`, `shape`, `line`）。
- 🖼️ **形状图片填充**：形状目前仅支持纯色 (`type: solid`) 与渐变色 (`type: gradient`) 填充；形状的图片填充 (`fill: {type: image}`) 不受支持（`a:blipFill` 目前仅支持页面背景）。若需展示图片，请使用独立的 `image` 元素。
- 📐 **画布越界元素**：元素的布局尺寸应位于幻灯片画布范围之内；超出画布的元素会在 `pptd check` 阶段被捕获（完全出界报 Error，部分出界报 Warning）。
---

## 📁 PPTD 项目结构标准

一个标准、完全自包含的 PPTD 项目结构如下：

```text
my_presentation/
├── deck.pptd             # 主清单：尺寸比例、主题配色、全局字体、页面引用
├── pages/
│   ├── 01_cover.page     # 单页 AST（背景、形状、文本、表格、连线）
│   ├── 02_problem.page
│   └── 03_solution.page
└── assets/               # 页面引用的图片与媒体资产
```

详细 DSL 语法与设计规范请查阅：
- [快速上手指南 (SKILL.md)](skills/open-pptd/SKILL.md)
- [PPTD v2 核心语法规范 (pptd.md)](skills/open-pptd/reference/pptd.md)
- [30+ 款精选主题配色表 (theme.md)](theme.md)
- [CLI 命令参考手册 (cli.md)](skills/open-pptd/reference/cli.md)
- [跨平台安全排版与字体指南 (fonts.md)](skills/open-pptd/reference/fonts.md)

---

## 🛠️ 附录：开发者手动安装与 CLI 命令（高级调试）

<details>
<summary>点击展开：脱离 Agent 手动运行 <code>pptd</code> CLI 指南</summary>

如果您需要脱离 Agent 在本地终端中直接调用 `pptd` CLI 进行开发或调试，可以通过以下方式安装与运行：

### 1. 手动安装 CLI
```bash
# 方式 A：通过 curl 脚本一键下载预编译包到 ~/.local/bin/pptd
curl -fsSL https://raw.githubusercontent.com/shiquda/open-pptd-skill/main/skills/open-pptd/scripts/install_binary.sh | bash

# 方式 B：通过 Python 跨平台安装脚本
python3 skills/open-pptd/scripts/install_binary.py

# 方式 C：通过 Rust / Cargo 从源码编译安装
cargo install --git https://github.com/shiquda/open-pptd-skill.git
```

### 2. 核心 CLI 命令
- **语义与结构静态质检**：
  ```bash
  pptd check my_project/deck.pptd
  ```
- **毫秒级编译为原生 PPTX**：
  ```bash
  pptd build my_project/deck.pptd -o presentation.pptx
  ```
- **反向反编译已有 PPTX 为 PPTD 工程**：
  ```bash
  pptd convert presentation.pptx ./decompiled_project/
  ```
- **自动化深度质检套件**：
  ```bash
  python3 skills/open-pptd/scripts/validate_pptx.py presentation.pptx --expected-pages 8 --check-notes
  ```

### 3. 回归验证（端到端 fixture 套件）
`tests/decks/` 下有 12 套真实场景文稿（98 页），覆盖表格、原生公式、箭头/连线、行列容器、主题、Kimi 基准等语法组合。每次改渲染或校验层后跑一遍：

```bash
cd tests/decks && ./verify_all.sh          # check → build → validate，约 10 秒
RENDER=1 ./verify_all.sh                   # 额外用 LibreOffice 渲染页面图 + 接触表
PPTD=../../target/debug/pptd ./verify_all.sh   # 用本地构建的二进制
```

产物写在各自 `build/`（已 gitignore）。缺陷排查记录与能力边界见 [`tests/decks/README.md`](tests/decks/README.md)。

</details>

---

## 📄 开源协议与致谢 (License & Attribution)

本项目采用 [MIT 许可证](LICENSE)。

### 鸣谢与上游开源项目致谢：
1. **`open-kimi-ppt-skill`**：
   本项目源于并基于 Binaryify Zhuang 开发的 `open-kimi-ppt-skill`（MIT 许可证，[GitHub](https://github.com/acnlie/open-kimi-ppt-skill)）演进重构。特别致谢原作者对 PPTD 声明式幻灯片设计哲学与 Agent 技能形态的奠基性贡献。
2. **`slideforge` (原生 Rust 编译器核心)**：
   底层核心 Rust 编译器引擎源自原作者 **meatRoll** 创建的开源项目 [`meatRoll/slideforge`](https://github.com/meatRoll/slideforge)（MIT 许可证）。由 shiquda 及其贡献者进行了深度二次开发（丰富富文本排版、增强学术 LaTeX 数学公式渲染、实现完整演讲者备注 notesSlide 关系链、扩充 1422 个矢量图标库并强化 check 严格静态校验诊断），现正式合并并入 Open PPTD 单体代码库以 `pptd` CLI 统一维护。在此向原作者 meatRoll 致以诚挚感谢。
3. **Font Awesome Free 6 (Solid Icons)**：
   项目内嵌的矢量图标集合来源于 Font Awesome 6.7.2（CC BY 4.0 许可证，[License](https://fontawesome.com/license/free)）。
