# 08-finance-ledger · 金融财务账本（三张表结构解读与资产负债深度穿透）

## 1. Design Brief (设计简要)

- **Scenario (业务场景)**: 某虚构上市公司 2026 年中报财务账本深度穿透分析，涵盖合并利润表、合并资产负债表（8列×10行宽表）、合并现金流量表（净现比与造血能力）、偿债压力测试敏感性矩阵、细分业务线分部盈利明细、杜邦分析三因子矢量框架及投委会投后监控决策哨卡。
- **Audience (目标受众)**: 投资委员会委员、一级/二级市场专业投资经理、资深信用评级与财务分析师。
- **Chosen Preset (选定设计系统预设)**: `finance/black-gold-ledger`
  - Preset Path: `skills/open-pptd/reference/design_system/finance/black-gold-ledger/design.md`
- **Palette Hexes (调色板规范)**:
  - White background `#FFFFFF`: 承载实证分析与数据明细表（第3–8页）
  - Dark background `#151515`: 封面、执行摘要与行动决策（第1、2、9页）
  - Ink black `#0A0A0A`: 顶端细黑标线（2pt）、主标题、第一行表头线与关键文本
  - Warm dark gray `#3D3C3A`: 正文与表格数据主要字符
  - Secondary gray `#606060`: 副标题、单位说明
  - Supporting gray `#A7A7A7`: 辅助系列、暗色页说明
  - Footnote gray `#B6B6B6`: 脚注来源说明与固定页码
  - Light gray `#D6D6D6` / Hairline `#E9E9E9`: 浅灰表格发丝线与模块分割线
  - Panel gray `#F0F0F0`: 结论横带、底部分析判语卡
  - Track gray `#F3F3F3`: 宽表格交替行浅灰色底色
  - Primary gold `#D6A000`: 核心指标数字、当前期重点、阈值胶囊、短金色封面标线（每页严格单一焦点）
  - Copper brown `#A86038`: 尾部风险预警胶囊（低频）
- **Grid & Typography Decisions (网格与排版)**:
  - Canvas: 16:9 (`[960, 540]`)
  - Margins: 左右边距 `52pt` (5.4%)，上边距 `24-28pt`，下边距 `35pt`，内容跨度 `856pt` (89.2%)。
  - Header Fixture: 白底页固定 2pt 细黑通栏置顶横线 + 结论性大标题（17pt） + 口径副标题（9.5pt） + 1pt 发丝分割线。
  - Reading Discipline: 标题即结论；页脚左侧固定数据源与准则条文，右侧固定 `n / 9` 格式页码。
  - Accounting Standards Cited: 《企业会计准则第30号——财务报表列报》(CAS-30)、《企业会计准则第31号——现金流量表》(CAS-31)、《企业会计准则第35号——分部报告》(CAS-35)、中国证监会《公开发行证券的公司信息披露编报规则第15号》。

---

## 2. PPTD Syntax-Coverage Checklist (特性覆盖清单)

| PPTD 特性 | 所在页面 | 验证表现 |
|---|---|---|
| 原生表格元素 (`elementType: table`) | P3, P4, P5, P6, P7 | 5个独立页面生成原生 DrawingML `<a:tbl>`，非图片拼接 |
| 宽表格 (8+ 列 × 8+ 行) | P4 (资产负债表) | 8列 × 10行，列宽和与行高和严格为 1.0，绘制无溢出 |
| `columnWidths` 归一化 (sum=1.0) | P3, P4, P5, P6, P7 | 各表列宽比例均精准归一化，无几何拉伸 |
| `rowHeights` 归一化 (sum=1.0) | P3, P4, P5, P6, P7 | 各表行高比例均精准归一化，单元格文字垂直居中 |
| 表头横向跨列合并 (`colSpan`) | P4, P6 | P4表头 `colSpan: 2` 与 `colSpan: 3`，OOXML生成 `gridSpan` + `hMerge` |
| 行首纵向跨行合并 (`rowSpan`) | P4, P5 | P4 `rowSpan: 4` 与 `rowSpan: 3`；P5 `rowSpan: 3` 与 `rowSpan: 2`；OOXML生成 `rowSpan` + `vMerge` |
| 单元格渐变填充 (`fill: gradient`) | P5 (现金流表) | 2026H1经营现金流净额单元格采用线性双色金渐变，LibreOffice渲染正常 |
| 单元格独立填充 (`fill: solid`) | P3, P4, P5, P6, P7 | 合计行、关键指标单元格独立高亮填充 |
| 四向边框规范 (`BorderSpec: Clockwise [T,R,B,L]`) | P6 (偿债矩阵) | 速动比率单元格单独配置上下金线、左右浅灰线 |
| 边框显式清除 (`width: 0`) | P6 (偿债矩阵) | 显式覆盖并清除继承边框，OOXML输出 `<a:noFill/>` |
| 主题表格样式 (`theme.tableStyles`) | P3, P4, P5, P6, P7 | `$ledgerDefault`, `$ledgerWide`, `$cashFlowMatrix`, `$stressMatrix` |
| 表格分类样式覆盖 (`firstRowStyle` / `lastRowStyle` / `firstColumnStyle`) | P3, P4, P5, P6 | 第一行加粗与黑实线底边、末行双线、第一列加粗底纹 |
| 交替行底纹循环 (`bodyStyles`) | P3, P4, P7 | 斑马条纹在数据行间按索引循环交替填充 |
| 单元格对齐规则 (`align: [left/right/center, middle]`) | P3, P4, P5, P6, P7 | 项目文字靠左/居中，财务数据全部靠右严格按位对齐 |
| 表头字间距 (`letterSpacing`) | P3, P4, P5 | 表头文字配置 `letterSpacing: 0.5~0.8`，OOXML输出 `spc` 属性 |
| 行列优先级覆盖 (`rowOverColumn: false`) | P5 (现金流表) | 列分类样式优先于行分类样式生效，第一列底纹贯穿至表头 |
| 表格级阴影声明 (`shadow`) | P6 (偿债矩阵) | 在表格级配置 drop shadow，探测编译器行为 |
| 表格边界极限溢出探针 | P7 (分部明细表) | 表格底部延伸至 484pt（画布高540pt，边距仅18pt），密集多行文字 |
| 原生矢量图形与连接线 (`elementType: shape`, `line`) | P1, P2, P8, P9 | P8 完整杜邦三因子架构图（矩形节点 + 箭头连接线）保持完全可编辑 |
| 演说者演讲备注 (`notes:`) | 全部 P1–P9 | 每张幻灯片均包含结构化投研口径陈述，通过 `--check-notes` 门禁 |

---

## 3. Verification Evidence (验证执行证据)

1. **语法合法性预检 (`pptd check`)**:
   ```bash
   pptd check open-pptd-skill/tests/decks/08-finance-ledger/deck.pptd
   ```
   *执行结果*:
   ```text
   open-pptd-skill/tests/decks/08-finance-ledger/deck.pptd: 9 page(s) validated — no issues
   ```

2. **原生演示文稿编译 (`pptd build`)**:
   ```bash
   pptd build open-pptd-skill/tests/decks/08-finance-ledger/deck.pptd -o open-pptd-skill/tests/decks/08-finance-ledger/build/deck.pptx --force
   ```
   *执行结果*:
   ```text
   wrote open-pptd-skill/tests/decks/08-finance-ledger/build/deck.pptx
     sync point recorded: open-pptd-skill/tests/decks/08-finance-ledger/build/deck.pptx = 9cd84179e2b49b4c17c794035f908678000fa7a2b7fd21fd242ef543d764b5af
   ```

3. **合规性验证与讲者备注检查 (`validate_pptx.py`)**:
   ```bash
   python skills/open-pptd/scripts/validate_pptx.py \
     open-pptd-skill/tests/decks/08-finance-ledger/build/deck.pptx \
     --expected-pages 9 --check-notes
   ```
   *执行结果*:
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
     Slide  1: [Notes: Yes]   (11 text, 0 img, 0 tbl)
     Slide  2: [Notes: Yes]   (20 text, 0 img, 0 tbl)
     Slide  3: [Notes: Yes]   (10 text, 0 img, 1 tbl)
     Slide  4: [Notes: Yes]   (5 text, 0 img, 1 tbl)
     Slide  5: [Notes: Yes]   (10 text, 0 img, 1 tbl)
     Slide  6: [Notes: Yes]   (5 text, 0 img, 1 tbl)
     Slide  7: [Notes: Yes]   (4 text, 0 img, 1 tbl)
     Slide  8: [Notes: Yes]   (17 text, 0 img, 0 tbl)
     Slide  9: [Notes: Yes]   (15 text, 0 img, 0 tbl)

   All checks passed cleanly! No leaked tags or boundary errors detected.
   ```

4. **无头渲染与图像审查 (`validate_pptx.py --render-images`)**:
   ```bash
   python skills/open-pptd/scripts/validate_pptx.py \
     open-pptd-skill/tests/decks/08-finance-ledger/build/deck.pptx \
     --render-images open-pptd-skill/tests/decks/08-finance-ledger/build/preview
   ```
   *执行结果*:
   - 生成 9 张单页渲染图 `slide-01.png` ~ `slide-09.png` 及全览图 `contact-sheet.jpg`。
   - 通过 `read` 工具逐页完成 100% 全尺寸视觉校验：各宽表文字未发生折叠溢出或重叠，合并单元格边界清晰无错位，杜邦分析箭头精确定位。

5. **底料 DrawingML XML 深度解包查验 (`unzip -p deck.pptx ppt/slides/slide4.xml`)**:
   - `colSpan: 2` 成功映射至 `<a:tc gridSpan="2">`，覆盖单元格生成 `<a:tc hMerge="1">`。
   - `rowSpan: 4` 成功映射至 `<a:tc rowSpan="4">`，下方连续三行对应位置生成 `<a:tc vMerge="1">`。
   - `letterSpacing: 0.5` 映射为 `<a:rPr spc="38">`。
   - 渐变单元格生成 `<a:gradFill><a:gsLst><a:gs pos="0">...<a:gs pos="100000">...</a:gradFill>`。

---

## 4. Defects & Rough Edges Found (编译器与文档缺陷分析)

### 缺陷 1: 表格级 `fill` 字段被 `pptd check` 拦截为未知字段 (严重性: 高)
- **Layer**: Compiler Validator (`src/pptd/validate.rs`)
- **Symptom**: 在 `Table` 元素上配置 `fill` 时，`pptd check` 与 `pptd build` 均报错拒绝编译：`element 'tbl' has unknown field 'fill'`。
- **Minimal Repro**:
  ```yaml
  - elementId: t
    elementType: table
    bounds: [50, 50, 400, 200]
    columnWidths: [0.5, 0.5]
    rowHeights: [0.5, 0.5]
    fill: { type: solid, color: "#F0F0F0" }
    rows:
      - - text: "A"
        - text: "B"
      - - text: "C"
        - text: "D"
  ```
  执行命令: `pptd check deck.pptd`
- **Observed Output**:
  ```text
  - page 1: element `t` has unknown field `fill`
  check failed: 1 error(s)
  ```
- **Root Cause**:
  `src/pptd/elements.rs` 的 `Table` 结构体以及 `src/pptx/table.rs` 的 `render_table` 均已完整实现 `table.fill`（向 `<a:tblPr>` 写入 `fill_xml`）。但 `src/pptd/validate.rs` 第 1240 行硬编码的白名单 `["rows", "columnWidths", "rowHeights", "style", "border", "shadow", "cellPadding"]` 遗漏了 `"fill"`，导致项目校验器误报并拦截合法属性。

---

### 缺陷 2: 表格级 `shadow` 属性被代码层完全丢弃，不生成任何 DrawingML 效果 (严重性: 中)
- **Layer**: Compiler DrawingML Writer (`src/pptx/table.rs`)
- **Symptom**: 在表格上配置 `shadow`，`pptd check` 校验通过，但编译生成的 PPTX 中表格完全没有阴影，DrawingML XML 中未写入任何 `<a:effectLst>`。
- **Minimal Repro**:
  ```yaml
  - elementId: t
    elementType: table
    bounds: [50, 50, 400, 200]
    columnWidths: [0.5, 0.5]
    rowHeights: [0.5, 0.5]
    shadow:
      blur: 8
      color: "#00000040"
      offset: [2, 4]
    rows:
      - - text: "A"
        - text: "B"
      - - text: "C"
        - text: "D"
  ```
  执行命令: `pptd build deck.pptd -o out.pptx && unzip -p out.pptx ppt/slides/slide1.xml | grep effectLst`
- **Observed Output**: 空输出（无 `<a:effectLst>`）。
- **Root Cause**:
  在 `src/pptx/table.rs` 的 `render_table` 函数中，直接生成 `<p:graphicFrame>` 及其 `<a:tbl>`，整个函数从未调用 `shadow_xml`。DrawingML 规范中表格属于 `graphicFrame`，不支持直接附加图形边框阴影；PPTD 编译器未将阴影映射到容器形状，而是直接静默忽略了 `table.shadow`。

---

### 缺陷 3: 单元格边框 `border: null` 与 `[null, null, null, null]` 无法清除继承边框 (严重性: 中)
- **Layer**: Compiler Serde Deserializer / Border Resolution
- **Symptom**: 按照规范说明书，`border: null` 应用于单元格以清除继承自 `cellStyle` 的边框。然而，写 `border: null` 时，继承边框依然保留；写 `border: [null, null, null, null]` 时，反而生成了 4 条无颜色无填充的 1pt 空线 (`<a:lnL w="12700" cap="flat"/>`)。
- **Minimal Repro**:
  ```yaml
  style:
    cellStyle:
      border: { style: solid, width: 2, color: "#CCCCCC" }
  rows:
    - - text: "A"
        border: null
      - text: "B"
        border: [null, null, null, null]
  ```
  执行命令: `pptd build deck.pptd -o out.pptx && unzip -p out.pptx ppt/slides/slide1.xml`
- **Observed Output**:
  - 单元格 A: 保留了继承自 `cellStyle` 的 `<a:lnL w="25400">...CCCCCC...</a:lnL>`。
  - 单元格 B: 生成了 `<a:lnL w="12700" cap="flat"/>`（无填充颜色），在 PPT 中显示为异常细线。
- **Root Cause**:
  1. `Cell.border` 在 Rust 模型中为 `Option<BorderSpec>`，当 YAML 输入 `border: null` 时，Serde 直接反序列化为 `None`，因此 `apply_inline` 认为该字段未设置，跳过了覆盖逻辑。
  2. 当输入 `[null, null, null, null]` 时，由于 `Border` 结构体各字段均有 default，`Option<Border>` 被 Serde 反序列化为 `Some(Border::default())`，导致 `cell_border_xml` 将其视为有效边框并使用默认宽度 `1.0pt (12700 EMU)` 发射 XML 标签。
  3. **实际工作替代方案**: 必须显式声明 `{ width: 0 }`，此时编译器会正确发射 `<a:noFill/>` 清除边框。

---

### 缺陷 4: `pptd check` 缺少对表格单元格文字溢出 (Text Overflow) 的检测机制 (严重性: 低)
- **Layer**: Project Validator (`src/pptd/validate.rs`)
- **Symptom**: 在 P7 中，表格单元格包含大量多行文本，整张表格高密度贴紧画布底边缘（底部达 484pt），文字充盈单元格，但 `pptd check` 没有任何溢出警告。
- **Repro & Observation**:
  `check_text_overflow` 仅在 `Element::Text` 上运行，完全跳过了 `Element::Table` 的每格 `Cell.text`。即使单元格文字在渲染端被行高截断或溢出，`pptd check` 也无法给出任何事前诊断。
