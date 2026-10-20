# Open-PPTD 垂直场景实测与 Kimi 标杆复现测试套件 (DEV-583)

本目录包含针对 `open-pptd` 编译器（Rust 核心）与 AI Agent Skill 的全量真实验收测试工程。覆盖 **8 大商业/工程垂直场景** 与 **2 大 Kimi Slide 官网高难度案例复现**，共计 **12 个独立工程，98 页原生幻灯片**。

所有工程均遵循真实数据源标准、严禁 AI 编造假数据与卡片墙堆砌，经过全量 PPTD 语法压力测试、自动化校验（`pptd check`）、原生二进制编译（`pptd build`）、OpenXML 语义结构审查（`validate_pptx.py`）以及 Headless 高清渲染走查（LibreOffice + pdftoppm）。

---

## 1. 测试工程矩阵 (Deck Matrix)

| 目录 (Slug) | 场景与业务定位 | 画布尺寸 | 页数 | 设计规范 / 主题色彩 | 核心语法与极限压力覆盖 | 自动化状态 |
|---|---|---|---|---|---|---|
| `01-analysis-industry-research` | **分析与决策**：全球储能电芯与中国厂商出海研报 | 960×540 (16:9) | 10 | `consulting/indigo-due-diligence` (深蓝咨询风格) | 复杂合并表头数据表、微笑曲线折线图、决策矩阵、hexagon 形状微调、位置联动导航条、`letterSpacing` | **PASS** (10/10) |
| `02-business-pitch` | **商业方案与融资**：东南亚冷链 SaaS 商业计划书 | 960×540 (16:9) | 10 | `tech/emerald-clean-saas` (祖母绿双主色) | Unit Economics 财务单元表、TAM/SAM 漏斗图、四层架构图、多段对比折线图、`lineHeightPx` | **PASS** (10/10) |
| `03-management-okr-review` | **管理汇报与述职**：B2B 基础软件 Q3 经营复盘与 OKR | 960×540 (16:9) | 9 | `consulting/warm-gray-corporate` (米灰暖调管报) | 红黄绿健康度微章、瀑布归因分析图、大交付延期甘特图、财务三行表、`margin-top` 紧凑段落 | **PASS** (9/9) |
| `04-academic-latex-defense` | **学术科研**：稀疏注意力机制博士毕业答辩 | 960×540 (16:9) | 10 | `academic/teal-green-academic-defense` (松石绿学术) | **重度原生 LaTeX/OMML**：多行方程组 `aligned`/`split`、`pmatrix`/`bmatrix`/`cases`、公式宏 `\newcommand`、booktabs 三线表 | **PASS** (10/10) |
| `05-education-courseware` | **教育培训与科普**：家庭网络故障排查实操课 | 960×540 (16:9) | 10 | `education/academic-navy-curriculum` (海蓝实操课件) | 教学目标行为动词、拓扑链路图、三色指示灯决策树、排障操作清单表、禁忌对比警示卡 | **PASS** (10/10) |
| `06-tech-architecture-review` | **技术工程与架构**：核心交易异地多活容灾演练评审 | 960×540 (16:9) | 10 | `engineering/amber-slate-white` (工科白底高对比) | 3 类语义箭头架构图（同步/异步/仲裁）、时序图、状态机、P50/P90/P99 指标表、代码窗口排版 | **PASS** (10/10) |
| `07-brand-magazine` | **品牌创意与作品集**：精品咖啡品牌年刊与风味实验 | 1280×720 (16:9 大画布) | 9 | `creative/klein-blue-swiss-poster` (克莱因蓝瑞士风) | 瑞士平面栅格、巨大数字排印（180pt）、等高线微调、非对称极值排版、雷达风味图 | **PASS** (9/9) |
| `08-finance-ledger` | **金融财务**：上市公司半年度财务账本与资产负债穿透 | 960×540 (16:9) | 9 | `finance/black-gold-ledger` (黑金审计账本) | 三张表勾稽穿透、多层跨列合并单元格、杜邦分析树、负债预警底色、多维穿透对账表 | **PASS** (9/9) |
| `08b-poster-infographic` | **通用海报与信息图**：新能源汽车出海数据海报 | 1080×1080 (1:1 独立海报) | 1 | `finance/black-gold-poster` (黑金专业信息图) | 正方形海报画布、复合月度交付条形图、Top 5 省市渗透率数据表、超大 KPI 统计徽标 | **PASS** (1/1) |
| `09-kimi-fusion-industry` | **Kimi 标杆复现 A**：可控核聚变商业化全景研报 | 960×540 (16:9) | 16 | `consulting/marine-blue-research` (Kimi 官网同源深蓝) | 完整 1:1 复现 Kimi 16 页超长研报：三重积标度律、三代反应堆物理参数表、LCOE 平价曲线图、TRL 阶段门禁 | **PASS** (16/16) |
| `10a-gwtc5-gravitational-wave` | **Kimi 标杆复现 B1**：引力波目录天体物理全景图谱 | 960×540 (16:9) | 11 | `academic/paper-white-courseware` (极简学术信息图) | 完整复现 Kimi 官网左侧常驻导航轨（Rail Navigation）、全天区定位图、质量间隙双峰分布、LISA 天基天线 | **PASS** (11/11) |
| `10b-urban-heat-island` | **Kimi 标杆复现 B2**：城市热岛微气候热力学信息图 | 960×540 (16:9) | 2 | `scientific/dark-thermodynamic` (暗色热力学图谱) | 暗色信息图、左侧温度对比色阶卡、热平衡公式拆解、30m 空间网格温度微区矩阵 | **PASS** (2/2) |

**全套件统计**：共计 **12 个项目**，**98 张幻灯片**，100% 通过编译与结构校验。

---

## 2. 自动化验证指南 (Verification Guide)

本套件提供了一键全量回归验证脚本 `verify_all.sh`，可自动化执行三级防御门禁：
1. **语法与静态语义校验**：`pptd check <manifest>`
2. **原生二进制编译构建**：`pptd build <manifest> -o build/deck.pptx --force`
3. **OpenXML 语义结构审查**：`validate_pptx.py build/deck.pptx --expected-pages <N> --check-notes`
4. **（可选）Headless 高清预览渲染**：`--render-images build/preview` 导出全量 2001×1125 PNG 及 contact-sheet.jpg

### 运行方式

```bash
# 进入测试目录
cd tests/decks

# 基础校验（检查 + 构建 + OpenXML 语义核验）
PYTHON=../../.venv/bin/python ./verify_all.sh

# 结合本地编译的最新 debug 二进制测试
PPTD=../../target/debug/pptd PYTHON=../../.venv/bin/python ./verify_all.sh

# 仅测试指定项目（如仅跑学术 LaTeX 与 Kimi 核聚变）
ONLY="04-academic" PYTHON=../../.venv/bin/python ./verify_all.sh
ONLY="09-kimi" PYTHON=../../.venv/bin/python ./verify_all.sh

# 开启全量无头渲染生成预览（需要本地安装 LibreOffice / soffice）
RENDER=1 PYTHON=../../.venv/bin/python ./verify_all.sh
```

---

## 3. 核心缺陷与编译器边界排查汇总 (Bug & Defect Catalog)

在实测 98 页复杂场景的过程中，各攻坚智能体深度探针触发并定位了多项编译器底层缺陷与 Skill 文档错漏：

### P0 — 导致 PPTX 损坏的致命错误 (Critical / Corrupt Output)
1. **`curve: smooth` 闭合标签错误导致 XML 格式损坏**：
   - **位置**：`src/pptx/render.rs:2001`
   - **根因**：代码以 `xml.start("a:cubicBezTo", ...)` 打开贝塞尔曲线几何段，但闭合时错误使用了 `xml.end("a:cTo")`。
   - **后果**：`pptd build` 虽不报错，但生成的 slide XML 含有非配对标签，PPTX 文件损坏，Office/Keynote 打开报错，`pptd convert` 自身反向解析崩溃。
   - **修复**：已在本次 PR 中修复为 `xml.end("a:cubicBezTo")`，并补充单元测试。

### P1 — 阻断文档合法语法的编译器/校验器缺陷 (High)
2. **校验器遗漏合法的 `cropShape` 与 `softEdge` 字段**：
   - **位置**：`src/pptd/validate.rs:1239` 图片字段白名单。
   - **根因**：`Image.crop_shape` 与 `soft_edge` 在 AST 与渲染管线中已完全实现，但白名单漏记，导致 `pptd check` 报错退出。已在 PR 中补齐。
3. **校验器遗漏合法的表格级 `fill` 字段**：
   - **位置**：`src/pptd/validate.rs:1249` 表格字段白名单。
   - **根因**：`Table.fill` 已定义并支持，白名单遗漏导致全表底色语法被拦截。已在 PR 中补齐。
4. **校验器遗漏 `customFonts` 根属性**：
   - **位置**：`src/pptd/validate.rs:1118` `VALID_PRES_KEYS`。
   - **根因**：文档与 AST 均声明支持外部自定义字体，但白名单将其当作未知字段报错。已在 PR 中补齐。
5. **`BorderSpec` untagged 枚举反序列化顺序反转导致四元素 null 数组无法清空边框**：
   - **位置**：`src/pptd/shared.rs:172`
   - **根因**：`Border` 结构体自身刚好有 4 个字段且实现序列反序列化。`Uniform(Border)` 排在 `Clockwise` 前面时，`[null, null, null, null]` 会被误解析为默认 `Uniform(Border)`，在四边绘制 1pt 黑色边框。已在 PR 中将数组变体调整到 `Uniform` 之前。
6. **LaTeX `\lim` 生成非法单字符约束 `<m:chr m:val="lim"/>`**：
   - **位置**：`src/pptx/math.rs`
   - **根因**：`\lim` 被错误映射到 `<m:nary>`，其 `m:chr` 属性按 OOXML 规范只能为单个字符。LibreOffice 渲染为乱码倒置问号 `¿`。

### P2 — 体验与表现层缺陷 (Medium)
7. **富文本携带样式的 `<b>`/`<strong>` 标签原样泄漏**：
   - 带有行内属性的 `<b style="...">` 或 `<strong style="...">` 未能正确解析为带有样式的 Run，而是作为转义纯文本输出到幻灯片上。已在 `validate_pptx.py` 增强检测。
8. **表格阴影（`Table.shadow`）静默丢弃**：
   - 表格语法允许声明 `shadow`，但渲染管线未向 `graphicFrame` 输出 `<a:effectLst>`，阴影无视觉效果。
9. **`lineHeightPx` 与 `lineHeight` 优先级文档与实现相反**：
   - 文档声明冲突时 `lineHeightPx` 优先，但实现代码分支中 `lineHeight` 百分比优先。已在文档中明确纠正。
10. **验证器 `--render-images` 存在旧图片残留累积与重复计数**：
    - 已在 `validate_pptx.py` 中增加渲染前清理逻辑，杜绝多次渲染导致接触表包含旧页面的 Bug。

---

## 4. Kimi Slide 复现差异度与架构能力极限评估 (Kimi Benchmark Gap Analysis)

通过对 Kimi 官网核聚变（16页）、引力波（11页）和城市热岛（2页）的 1:1 深度复现，评估 open-pptd 达成 Kimi 级视觉高度的可行性与核心短板：

### 已经完全匹敌的能力 (100% Parity)
- **多栏分析排版与视觉节奏**：完全可以通过声明式的 `bounds` 栅格计算实现 Kimi 标志性的 3 栏/4 栏严谨对齐。
- **色彩与设计体系继承**：通过 `theme.colors` 命名系统，可以完美还原咨询风深蓝金灰（Marine Blue）、工科白底高对比等权威色彩。
- **混合公式与密集表格排版**：学术公式内联/独立居中渲染非常扎实；原生表格的复杂跨行跨列、差异化底色与边框完全可以胜任审计级报告。

### 核心短板与架构演进建议 (Fundamental Gaps)
1. **缺乏组件化 / 宏模板复用能力（Component / Template Gap）**：
   - *痛点*：在复现 11 页引力波报告时，左侧导航轨（Rail Navigation）包含 6 个缩略图卡片、时间轴和状态徽章，由于 PPTD 缺乏 `<template>` 或外部 YAML 片段引用机制，必须在 11 个 `.page` 文件中手工复制 300+ 行重复坐标。
   - *建议*：引入 `template` 或 `snippet` 局部复用语法，允许通过 `useTemplate: rail-nav` 统一注入公共组件。
2. **缺乏原生声明式图表支持（Native Chart Gap）**：
   - *痛点*：Kimi 呈现的三重积折线图、LCOE 平价带、热力学散点图，在 PPTD 中只能依赖外部 Python/matplotlib 脚本生成高分辨率 PNG，再通过 `elementType: image` 导入。用户在 PowerPoint 中无法双击编辑底层数据。
   - *建议*：未来打通 `elementType: chart`，生成原生 OOXML `<c:chartSpace>`。
3. **公式对暗色模式（Dark Themes）的自适应不足**：
   - *痛点*：LaTeX 数学公式在编译为 OMML `<m:r>` 时，未能自动继承外层文本框的 `color` 属性（如 `#F8FAFC`），默认输出纯黑文字，导致暗色背景上公式完全“隐形”。在城市热岛（10b）工程中不得不回退至富文本 Unicode 符号排版。
   - *建议*：在 `math.rs` 中将当前段落的有效前景色透传至 `<m:rPr><a:solidFill>`。
