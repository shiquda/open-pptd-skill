# 03-management-okr-review: 云枢网络 2026 Q3 经营复盘与 Q4 OKR 承诺

本套演示文稿为 `open-pptd` 原生 Rust 编译器（v1.4.1）与声明式 PPTD 语言构建的工业级经管复盘胶片，严格遵循管理层报告（Management Reporting）业务规范与 `work/warm-jade-annual-report` 设计系统。

---

## 1. 设计概要 (Design Brief)

- **应用场景**：管理层经营复盘与 OKR 决策汇报 (`skills/open-pptd/reference/slides_categories/management-report.md`)
- **汇报对象**：集团董事会与经营管理委员会（CEO、COO、CFO、VP 销售 / 交付 / 产研）
- **虚构标的**：云枢网络科技股份有限公司（CloudPivot SaaS，全案标记为「示例公司 / 假设口径」）
- **风格预设**：`work/warm-jade-annual-report`
  - 预设文档路径：`skills/open-pptd/reference/design_system/work/warm-jade-annual-report/design.md`
- **调色板规范 (Palette Hexes)**：
  - 画布底色：暖象牙白 `#FBF9EE` (`$bg`)
  - 容器与卡片底色：纯白 `#FFFFFF` (`$surface`)、微暖淡米 `#F6F3E7` (`$surfaceAlt`)
  - 核心文字色：炭黑 `#181716` (`$ink`)、暗灰 `#3C3733` (`$inkSecondary`)、次级次灰 `#6B625D` (`$muted`)、淡灰 `#A09B93` (`$faint`)
  - 结构线框色：中性米灰 `#CCC5BE` (`$border`)、浅分界线 `#E5DFD7` (`$borderLight`)、重表格线 `#181716` (`$tableLineHeavy`)
  - 风格特征主强调色：蜜糖琥珀金 `#FDC356` (`$accent`)、深琥珀 `#C68A1B` (`$accentDeep`)、浅琥珀淡底 `#FFF1D6` (`$accentLight`)
  - 语义红黄绿状态系统 (Traffic-Light Semantic System)：
    - 🟢 正常 / 达成 / 超标：翡翠绿 `#2D6A4F`，浅底 `#E8F4EC`，边框 `#A3D9B8`
    - 🟡 预警 / 关注 / 迟滞：暖琥珀 `#D97706`，浅底 `#FEF3C7`，边框 `#FCD34D`
    - 🔴 风险 / 阻滞 / 严重延期：陶土砖红 `#B83A2C`，浅底 `#FEE2E2`，边框 `#FCA5A5`
- **版式与字体系统 (Grid & Typography Decisions)**：
  - 比例与尺寸：16:9 宽屏，`size: [960, 540]`
  - 贯通微导航系统：左上角 3 细水平线指示器 (`bounds: [50, 26, 16, 2]`)，段落章节标签 (`[74, 25, 480, 16]`)，右上角报告定界，顶部细分割线 (`y=46`)。
  - 连贯标题测试 (Sequential-Title Test)：所有正文页标题均为明确业务论点 / 状态决断，连读 9 页标题即构成完整经营因果叙事，无任何「情况汇报」等形式主义空话。
  - 底部规范：固定左下角数据口径来源声明 (`y=506`) 与右下角页码编码 (`NN / 09`)。
  - 字体层级：标题、大数字与关键徽标统一采用现代无衬线体（`MiSans`）；正文分析与长句叙述遵循 Warm Jade 规范采用经典宋体（`思源宋体`）；所有数值列右对齐并严禁使用衬线体数字。

---

## 2. PPTD 语法与特性覆盖清单 (Syntax-Coverage Checklist)

| 特性分类 | 具体语法与参数 | 覆盖页面 | 落地说明 |
|---|---|---|---|
| **工程配置与清单** | `version: v2`, `size: [960, 540]`, `title` | `deck.pptd` | 规范级工程清单与主题声明 |
| **主题色彩与继承** | `theme.colors` 声明 24 个语义色，元素 `$colorKey` 引用 | `deck.pptd`, Pages 1-9 | 全局严格色彩引用，check 阶段静态校验 |
| **层级文字样式** | `theme.textStyles`: `$title`, `$subtitle`, `$body`, `$metric`, `$footer` 等 | `deck.pptd`, Pages 1-9 | 跨页面统一字体大小、行高、粗细与基线 |
| **高级表格样式** | `theme.tableStyles` 声明 `$varianceTable`，包含 `firstRowStyle`, `firstColumnStyle`, `lastRowStyle`, `bodyStyles`, `rowOverColumn` | `deck.pptd`, Page 4 | 差异定量归因表继承复杂主题样式 |
| **四向边框规范** | `border: [top, right, bottom, left]` 顺时针 BorderSpec 数组 | `deck.pptd`, Page 4 | 财务三线表：首尾行重线、数据行淡线、竖线清空 |
| **单元格合并** | `elementType: table` 中的 `rowSpan` (跨行) 与 `colSpan` (跨列) | Page 4 | 表头 4 列跨 2 行、财务金额跨 3 列、汇总行跨 2 列 |
| **单元格对齐与覆盖** | `align: [right, middle]`, `align: [center, middle]`, 单元格内联富文本 | Page 4 | 财务数据严格右对齐，表头居中，状态列居中 |
| **状态信号灯体系** | `roundRect` (底框) + `ellipse` (信号灯珠) + `text` (状态文字) | Pages 1, 2, 3, 6 | 严格遵守「红黄绿+文字标签」双重编码，绝不单用颜色表意 |
| **环形指示器** | `shapeName: "donut"`, `adjustments: [25000]` | Page 6 | 战役进度完成度环形指示器，矢量无损渲染 |
| **里程碑标记形状** | `shapeName: "diamond"` | Page 9 | 甘特死线硬性门禁节点标记（11/15, 12/25） |
| **线条与连线** | `elementType: line`, `points: "x1,y1 x2,y2"`, `viewBox` | Pages 1-9, 5, 9 | 导航图标、页眉线、页脚线、虚线网格、甘特流程线 |
| **箭头端点** | `arrow: [null, "triangle"]` | Page 9 | 甘特泳道阶段依赖箭头， DrawingML 矢量端点 |
| **非均分指标卡** | 非 1/3 或 1/4 均分的非对称布局 (Hero 38% + 右侧 3 栏叠层 58%) | Page 2 | 突出核心 ARR 达成率，拒绝 AI 式均分排版 |
| **动能瀑布桥接图** | 矢量柱状 shape + 浮动 shape + 虚线连接器 `line` 组合 | Page 5 | 纯原生 DrawingML 矢量构建的 3,480万→3,280万瀑布图 |
| **甘特执行泳道图** | 部门泳道分栏 + 进度条 shape + 连接箭头 + 11/15 核心死线虚线 | Page 9 | 产研、交付、CSM 跨部门工期依赖与硬门禁 |
| **本地图表资产引用** | `elementType: image`, `src: "assets/*.png"`, `fit: { mode: "contain" }` | Pages 3, 7 | matplotlib 生成的留存曲线、漏斗图，零外部下载 |
| **演讲者全量备注** | `.page` 根级 `notes: \|` 详细汇报脚本与数据背景 | Pages 1-9 | 通过 `--check-notes` 校验，无一缺失 |

---

## 3. 验证执行与结果凭证 (Verification Evidence)

所有命令均在终端执行并取得输出：

### 1. PPTD 静态语法检查
```bash
/Users/jim/.local/bin/pptd check open-pptd-skill/tests/decks/03-management-okr-review/deck.pptd
```
- **输出结果**：`open-pptd-skill/tests/decks/03-management-okr-review/deck.pptd: 9 page(s) validated — no issues`
- **结论**：通过，9 页面无未定义色彩引用或语法错误。

### 2. 编译为 OpenXML PPTX
```bash
/Users/jim/.local/bin/pptd build open-pptd-skill/tests/decks/03-management-okr-review/deck.pptd -o open-pptd-skill/tests/decks/03-management-okr-review/build/deck.pptx --force
```
- **输出结果**：`wrote open-pptd-skill/tests/decks/03-management-okr-review/build/deck.pptx` (sync point 记录完成)
- **结论**：编译成功，生成合法 `.pptx` 包。

### 3. PPTX 完整性与备注验证
```bash
.venv/bin/python open-pptd-skill/skills/open-pptd/scripts/validate_pptx.py \
  open-pptd-skill/tests/decks/03-management-okr-review/build/deck.pptx \
  --expected-pages 9 --check-notes
```
- **输出结果**：
  ```text
  =================================================================
  PPTX VALIDATION REPORT: deck.pptx
  =================================================================
  Status:       [PASS]
  Slides:       9
  Errors:       0
  Warnings:     0
  -----------------------------------------------------------------
  Slide Overview:
    Slide  1: [Notes: Yes]   (33 text, 0 img, 0 tbl)
    Slide  2: [Notes: Yes]   (33 text, 0 img, 0 tbl)
    Slide  3: [Notes: Yes]   (20 text, 1 img, 0 tbl)
    Slide  4: [Notes: Yes]   (10 text, 0 img, 1 tbl)
    Slide  5: [Notes: Yes]   (30 text, 0 img, 0 tbl)
    Slide  6: [Notes: Yes]   (33 text, 0 img, 0 tbl)
    Slide  7: [Notes: Yes]   (24 text, 1 img, 0 tbl)
    Slide  8: [Notes: Yes]   (30 text, 0 img, 0 tbl)
    Slide  9: [Notes: Yes]   (30 text, 0 img, 0 tbl)
  All checks passed cleanly! No leaked tags or boundary errors detected.
  =================================================================
  ```
- **结论**：验证通过，无任何泄露 HTML 标签，9 页备注全部就绪。

### 4. LibreOffice 无头渲染与全页面图像视觉审查
```bash
.venv/bin/python open-pptd-skill/skills/open-pptd/scripts/validate_pptx.py \
  open-pptd-skill/tests/decks/03-management-okr-review/build/deck.pptx \
  --expected-pages 9 --check-notes --render-images open-pptd-skill/tests/decks/03-management-okr-review/build/preview
```
- **输出结果**：在 `build/preview/` 生成 `slide-01.png` ~ `slide-09.png` 及 `contact-sheet.jpg`。
- **视觉审查动作**：通过 `read` 工具逐一读取了所有 9 张全尺寸 PNG 与总览联络单，发现并修复了：
  1. Slide 5 瀑布图原先独立峰值文本标签与副标题文本框发生 Y 轴重叠；已重构成将峰值信息融入柱顶标签并调整 Y 轴间距。
  2. Slide 5 瀑布图左侧网格刻度文字被柱 1 遮挡；已将文字刻度精简为纯粹点状参考虚线，彻底消除重叠。
  3. Slide 7 CAC 折线图 25Q3 标注与阈值参考线间距过近；已优化 Python 绘图标注偏移量。

---

## 4. 工具链缺陷与问题重现 (Toolchain Defects Found)

### 缺陷 1: `validate_pptx.py` 的 `--render-images` 在同目录下重复运行时发生文件名碰撞破坏幂等性
- **现象描述 (Symptom)**：当对同一个输出目录多次执行 `--render-images <outdir>` 时，`pdftoppm` 产生 `slide-1.png` 等文件，但 `output_dir.glob("slide-*.png")` 同时匹配到了上次生成的 `slide-01.png` 以及新生成的 `slide-1.png`，导致文件列表翻倍（9 页变成 18 个文件），重命名循环中旧文件被重命名为 `slide-10.png`..`slide-18.png`，并最终在生成 `contact-sheet.jpg` 时抛出 `FileNotFoundError: [Errno 2] No such file or directory: '.../slide-02.png'` 异常。
- **精准最小复现 (Minimal Repro)**：
  1. 构建包含 9 页的任意合法 `.pptx` 文件。
  2. 执行第一次渲染：
     ```bash
     python skills/open-pptd/scripts/validate_pptx.py out.pptx --render-images ./preview
     ```
     （此时执行成功，目录生成 `slide-01.png` ~ `slide-09.png` 和 `contact-sheet.jpg`）
  3. 再次执行相同命令：
     ```bash
     python skills/open-pptd/scripts/validate_pptx.py out.pptx --render-images ./preview
     ```
- **实际观察到的输出 (Observed Output)**：
  ```text
  [open-pptd-skill] Failed to generate contact sheet: [Errno 2] No such file or directory: '.../preview/slide-02.png'
  Rendered Slide Images:
    - .../preview/slide-01.png
    ...
    - .../preview/slide-18.png
  ```
- **定位怀疑层 (Suspected Layer)**：`skills/open-pptd/scripts/validate_pptx.py` 中的 `render_thumbnails` 函数（第 480-492 行）。未在运行 `pdftoppm` 前清空目标目录下的历史 `slide-*.png`。
- **严重等级**：中高（破坏 CI / 本地多次校验时的幂等性，导致虚假报错）。

### 缺陷 2: 富文本 `<b>` 标签携带 `style="..."` 时编译器静默放行但验证器拦截
- **现象描述 (Symptom)**：当在 PPTD 的行内文本中使用 `<b style="color:#B83A2C">文字</b>` 时，`pptd check` 与 `pptd build` 均正常返回 0 并成功编译出 `.pptx`，但在 `validate_pptx.py` 阶段被拦截，提示 `Found leaked/unrendered HTML tag <b style="...">`。
- **精准最小复现 (Minimal Repro)**：
  在任意 `.page` 的 `text` 字段写入：
  ```yaml
  text: "当前指标：<b style=\"color:#B83A2C\">96.2%</b>"
  ```
  执行校验：
  ```bash
  pptd check deck.pptd  # 结果：无报错通过
  pptd build deck.pptd -o test.pptx  # 结果：无报错通过
  python skills/open-pptd/scripts/validate_pptx.py test.pptx
  ```
- **实际观察到的输出 (Observed Output)**：
  ```text
  [ERROR] Slide 1: Found leaked/unrendered HTML tag `<b style="color:#B83A2C">` in slide 1
  ```
- **定位怀疑层 (Suspected Layer)**：`pptd` 编译器的富文本解析逻辑仅对 `<p>` 和 `<span>` 提取 `style` 属性映射到 OpenXML `<a:rPr>`，对 `<b>` 标签未能剥离未支持的属性，导致整个标签原样漏入 OpenXML 文本节点。
- **规避方案**：统一使用规范的 `<span style="color:#B83A2C"><strong>96.2%</strong></span>`。
- **严重等级**：低中（语法规范未在 check 阶段提前拦截，需统一编译器与文档的属性支持定义）。
