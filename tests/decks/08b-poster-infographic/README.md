# 08b-poster-infographic · 中国新能源汽车 2025 年销量与充电网络单页数据看板

## 1. Design Brief (设计简要)

- **Scenario (业务场景)**: 2025年中国新能源汽车全产业链年度数据看板（General Poster），涵盖整车年交付总量、新能源渗透率演化、全国充电桩基础设施保有量、月度爬坡走势（Matplotlib工业图谱）、头部品牌交付与补能生态结构（原生 DrawingML 表格）、公共充电桩功率结构升级、省市网络密度 TOP 5 及车桩比历史跨越与国际横向对比。
- **Audience (目标受众)**: 汽车产业投资者、宏观产业规划与能源战略决策者、高端制造与供应链分析师。
- **Chosen Design System & Rules Followed (选定风格体系与规范依据)**:
  - 遵循 `skills/open-pptd/reference/general-poster.md` 全部规范与执行清单（详见根目录 `style.md`）。
  - 画布比例：严格采用 1:1 正方形海报画幅 `[1080, 1080]`，规避将 16:9 幻灯片强行拉伸或裁剪的非专业做法。
  - 构图逻辑：非对称高视觉信息图场域（Asymmetric Industrial Data Field）。突破传统「标题 + 3张同构卡片」的刻板 PPT 布局，采用「顶部通栏权威宏观断言 → 核心大数高光区 → 左翼高分辨率月度趋势图谱 + 右翼原生微型矩阵宽表 → 底部三模块空间与功率纵深网络带」的复合信息层级。
  - 调色体系（规避通用 AI 蓝紫渐变）：
    - 画布底色 `#111317`: 深石墨碳黑，构筑深邃冷静空间感
    - 模块背板 `#1A1D24`: 工业深灰背板，建立物理信息容器
    - 销量高光金 `#F59E0B`: 核心整车销量指标与关键结论强调
    - 补能极光绿 `#10B981`: 充电网络、低碳电力及正向增量
    - 主白 `#F9FAFB`: 权威结论标题与第一层级关键文本
    - 金属银灰 `#9CA3AF`: 辅助说明、细分指标、图表刻度标签
    - 模块标尺分割线 `#262B34`: 极细边框与刻度基准线
- **Data Authenticity (数据真实性与权威依据)**:
  - 拒绝编造精度，所有核心宏观指标均源自官方公报：
    - 中国汽车工业协会（CAAM）《2025年汽车工业经济运行情况》
    - 乘用车市场信息联席分会（CPCA）《2025年度全国乘用车市场分析报告》
    - 中国电动汽车充电基础设施促进联盟（EVCIPA）《2025年全国电动汽车充换电基础设施运行情况》

---

## 2. PPTD Syntax-Coverage Checklist (特性覆盖清单)

| PPTD 特性 | 所在页面 | 验证表现 |
|---|---|---|
| 自定义非 16:9 画布尺寸 (`size: [1080, 1080]`) | deck.pptd / P1 | 1:1 方形大画幅，无头渲染像素严格对应 2250×2250，无几何拉伸失真 |
| 原生多列结构化表格 (`elementType: table`) | P1 (右侧品牌交付矩阵) | 5列 × 7行原生 `<a:tbl>`，列宽归一化 `0.24+0.20+0.18+0.18+0.20=1.0` |
| 主题表格样式 (`theme.tableStyles.posterTable`) | P1 | 表头金线与深色底纹、末行金字汇总与底边框、数据行斑马底纹循环 |
| 表格单元格对齐 (`align: [left/right/center, middle]`) | P1 | 品牌文本靠左、交付数值严格靠右、状态标签水平垂直居中 |
| 本地高分辨率数据图表引入 (`elementType: image`) | P1 (左侧月度图) | 引入本地 Matplotlib 渲染生成的抗锯齿深色图表，无缝融合 `#1A1D24` 背板 |
| 图像自适应容器裁剪与放置 (`fit: {mode: contain}`) | P1 | 保持图片原始纵横比自适应容器，无任何边缘拉伸 |
| 矢量几何容器与边框标尺 (`elementType: shape`, `rect`) | P1 | 顶部大数框、中部分析背板、下部三栏空间带均采用原生矩形，可随意解组编辑 |
| 演说者演讲备注 (`notes:`) | P1 | 完整记录看板宏观规模、补能底座与车桩协同三层要点，通过 `--check-notes` 门禁 |

---

## 3. Verification Evidence (验证执行证据)

1. **静态语法检查 (`pptd check`)**:
   ```bash
   pptd check open-pptd-skill/tests/decks/08b-poster-infographic/deck.pptd
   ```
   *执行结果*:
   ```text
   open-pptd-skill/tests/decks/08b-poster-infographic/deck.pptd: 1 page(s) validated — no issues
   ```

2. **二进制编译 (`pptd build`)**:
   ```bash
   pptd build open-pptd-skill/tests/decks/08b-poster-infographic/deck.pptd \
     -o open-pptd-skill/tests/decks/08b-poster-infographic/build/deck.pptx --force
   ```
   *执行结果*:
   ```text
   wrote open-pptd-skill/tests/decks/08b-poster-infographic/build/deck.pptx
     sync point recorded: open-pptd-skill/tests/decks/08b-poster-infographic/build/deck.pptx = 3e05b138d9480fc4b43b1c85907f8d3dd904565fd0c7470dbdfd115bec227d35
   ```

3. **合规性验证与讲者备注检查 (`validate_pptx.py`)**:
   ```bash
   python skills/open-pptd/scripts/validate_pptx.py \
     open-pptd-skill/tests/decks/08b-poster-infographic/build/deck.pptx \
     --expected-pages 1 --check-notes
   ```
   *执行结果*:
   ```text
   =================================================================
   PPTX VALIDATION REPORT: deck.pptx
   =================================================================
   Status:       [PASS]
   Slides:       1
   Errors:       0
   Warnings:     0
   -----------------------------------------------------------------
   Slide Overview:
     Slide  1: [Notes: Yes]   (20 text, 1 img, 1 tbl)    - "2025年中国新能源汽车销量达 1,287 万辆，充电网络突破 1,352 万台迈向高质量协同"

   All checks passed cleanly! No leaked tags or boundary errors detected.
   ```

4. **无头渲染与图像视觉全尺寸审查 (`validate_pptx.py --render-images`)**:
   ```bash
   python skills/open-pptd/scripts/validate_pptx.py \
     open-pptd-skill/tests/decks/08b-poster-infographic/build/deck.pptx \
     --render-images open-pptd-skill/tests/decks/08b-poster-infographic/build/preview
   ```
   *执行结果*:
   - 生成 `slide-01.png`（2250×2250 px）及 `contact-sheet.jpg`。
   - 使用 `read` 工具完成全尺寸视觉审查：版面各文字、表格、指标与本地绘制图表无缝咬合，无任何元素重叠、边界溢出或文本折行截断，呈现专业出版物级信息图质感。

---

## 4. Defects & Rough Edges Found (编译器与文档缺陷分析)

### 缺陷 1: 富文本标签解析器将 `<strong style="...">` 视作未知标签并原样泄漏打印在幻灯片上 (严重性: 中)
- **Layer**: Compiler Text Body Renderer (`src/pptx/render.rs`)
- **Symptom**: 当在富文本中书写形如 `<strong style="color: #F9FAFB;">文本</strong>` 时，编译器未能提取属性或容错，而是将整串标签 `"<strong style="color: #F9FAFB;">"` 作为纯文字内容直接渲染到 PPTX 画面上，导致严重的画面标签文字泄漏。
- **Minimal Repro**:
  ```yaml
  - elementId: t
    elementType: text
    bounds: [50, 50, 400, 50]
    content:
      text: "<p><strong style=\"color: #F9FAFB;\">加粗带颜色</strong></p>"
  ```
  执行命令: `pptd build deck.pptd -o out.pptx`
- **Observed Output**:
  渲染出的画面文字直接显示为：`<strong style="color: #F9FAFB;">加粗带颜色`。
- **Root Cause**:
  `src/pptx/render.rs` 第 1204–1237 行的 `parse_runs` 函数中：
  - 仅对 `name == "span style=..."` 做了 CSS 属性解析；
  - 对加粗标签仅精确匹配 `name == "strong" || name == "b"`（无任何参数属性）；
  - 遇到带有属性的 `strong` 标签时，分支直接落入 `else { buf.push_str(&tag[..=end]); }`，导致标签被原样塞入普通文本缓冲区发射给 PowerPoint。
  - **规范建议与规避方案**:
    1. 编译器解析器应支持提取任意标签的 `style` 属性或在忽略未知属性时剥除外层尖括号；
    2. PPTD 撰写时必须改写为规范嵌套形式：`<span style="color: #F9FAFB;"><strong>文本</strong></span>`。
