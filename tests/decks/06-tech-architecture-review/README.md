# 06 · 核心交易系统异地多活容灾架构演进评审 (Tech & Engineering / Architecture Review)

PPTD 工程压力测试稿：单文件 10 页 16:9，覆盖架构评审叙事全链路（决策摘要 → 目标与约束 → 现状/事故因果 → 备选方案 → 目标架构 → 状态机 → 时序 → 配置 → 指标 → 迁移与风险）。

---

## 1. 设计简报 (Design Brief)

**场景与读者**：Tech & Engineering → System architecture review。读者为架构委员会、SRE 可靠性团队与交易中台研发负责人；本稿需要让他们判断「故障边界、依赖、瓶颈与演进路径是否成立」，并据此做出立项决议。依据 `skills/open-pptd/reference/slides_categories/tech-engineering.md`（§1 核心性格、§5 页面节奏、§6 架构图纪律、§7 表格/代码规范、§10 交付前检查表）。

**叙事主线**（§4 推荐的 System architecture review 顺序 + §4 末段兜底顺序）：
决策摘要 → 目标与约束 → 现状与问题（事故因果） → 备选方案 → 目标架构 → 迁移与验证 → 风险与下一步。

**主题/预设**：未套用 `reference/design_system/` 中的 30 套预设之一（本稿为「白底工程文档」自建调色板，风格取向参考 §9 Sample A 的「暖色承载标题与关键路径 / 冷色承载节点与结构」与 Sample C 的「白底 + 短分割线 + 讲解页与证据页交替」，但**未继承**其具体色值、字体与页眉位置）。参考文件：`skills/open-pptd/reference/slides_categories/tech-engineering.md`（§8.1 版式、§8.2 颜色与字体、§9 样本参考）。

**调色板（自建，规避「蓝白 AI 默认」与「蓝紫渐变」）**：

| 语义 | 变量 | Hex | 用途 |
|---|---|---|---|
| 纸张 | `$paper` | `#FFFFFF` | 全局底色（阅读/打印向浅色底，§8.2） |
| 墨色 | `$ink` | `#1E242B` | 正文、节点标题（近黑冷灰，非纯黑） |
| 次要 | `$muted` | `#5C6470` | 注释、来源、脚注 |
| 发丝线 | `$hairline` | `#CBD5E0` | 分隔线、节点细边框 |
| 浅面 | `$surface` | `#F8FAFC` | 系统范围/代码/结论面板（细描边、无阴影） |
| 浅面深 | `$surfaceMuted` | `#EEF2F6` | 代码窗标题条、节点填充 |
| 品牌暖 | `$brand` | `#92400E` | 次级强调（切流/计划迁移路径） |
| 关键路径暖 | `$active` | `#C2410C` | **关键路径与推荐项**（同步业务流、推荐方案行、结论数字） |
| 风险冷红 | `$critical` | `#BE123C` | 故障、熔断红线、SLA 越界 |
| 同步冷青 | `$sync` | `#0F766E` | 异步复制、已验证结论、平账恢复路径 |
| 边框深 | `$borderDark` | `#94A3B8` | 故障域外框（比发丝线重一档，形成边界层级） |

颜色语义全局固定：**暖色=关键路径/推荐，冷红=风险与硬红线，冷青=异步/已验证，灰=现状与次要**；状态除颜色外一律加文字标签（§8.2「关键状态另以文字、线型或形状区分」）。

**栅格与排版**：
- 画布 `960 × 540`，正文安全边距 `45px`；页眉区 `y=24..68`（节标签 + 结论式标题 + 1px 发丝线），页脚区 `y=510..532`（左侧文档标识固定、右侧 `PAGE NN / 10` 固定位置，全 10 页一致）。
- 标题轴固定：`x=45`，`fontSize: 16`，结论句（非章节名）；节标签 `8.5pt` 小字大写英文 + 中文。
- 正文层级：正文 `8.2~9.5pt`，节点/表格 `7.2~8.5pt`，注释 `6.8~7.8pt`；标题 `16~22pt`。
- 字体：全稿 `MiSans`（现代无衬线，§8.2 推荐无衬线正文）。**注**：内置字体清单无等宽字体，`customFonts` 在本工具链下不可用（见 §4 缺陷 D1），故代码页改用「结构化代码窗容器 + 等宽化字符缩进 + 行号」的排版兜底，未伪造等宽字体。
- 版式骨架按页型复用：文字左 / 图右（P2、P9）、时间轴 + 因果双栏（P3）、整幅矩阵表（P4）、整幅架构图（P5）、状态机 + 数学证明（P6）、时序图（P7）、代码窗 + 分析（P8）、四阶段 + 双栏（P10）。无「标题 + 三并列块」公式化版式，无 2×2 / 四等分 / 三分默认切分（§2 禁止项）。

**现状 / 目标 / 假设 / 已验证 的显式区分**（§3.2、§3.3 强制）：P1 四个支柱各带 `状态定性` 标签；P2 分四轨 `[现状基准] / [目标指标] / [物理与技术约束] / [假设前提与边界]` 并各带证据来源；P9 底部固定「审计与数据真实性红线声明」；P5/P6/P10 节点内注明 `目标态` / `压测仿真` / `方案`。

**数据真实性**：本系统为虚构评审对象，**全部数值均为 `示例指标`**，非实测事实。P9 底部声明给出压测工具、版本、采样窗口、样本量（3.6 亿笔）、负载（25,000 TPS）、单位（ms）、基线与时间窗口，并明确「基于压测仿真假设与历史流量特征推演，严禁作为已投产真实事实引用」。P2 光纤延迟为物理推导（d=1200km、n≈1.468 → 单向 ≥18.4ms、RTT ≥36.8ms），P3 事故时间线为标注为生产回放的示例样本。无虚构引用、无未标注精度。

---

## 2. 语法覆盖清单 (Syntax Coverage Checklist)

| PPTD 能力 | 使用位置（页码） |
|---|---|
| `version: v2` / `size: [960,540]` / `pages` 清单 | `deck.pptd` |
| `theme.colors` 命名色 + `$key` 引用 | 全 10 页（未定义引用会被 check 拦截，P2 探针实测见 §4 D1） |
| `theme.textStyles` 继承 | 全稿；P4 表格经 `$engineering` |
| `theme.tableStyles`（`cellStyle` / `firstRowStyle` / `lastRowStyle` / `border`） | P4、P9 |
| `background: {type: solid}` | 全 10 页 |
| 页面级 `notes:` 演讲者备注 | 全 10 页（validator `--check-notes` 全绿） |
| `elementType: text`（`fontSize` / `bold` / `color` / `align:[h,v]` / `lineHeight`） | 全 10 页 |
| 多行块标量 `|` 文本 | P1、P2、P3、P5、P6、P7、P8、P9、P10 |
| `elementType: shape` + `rect` / `ellipse` | P1（分隔线）、P3（时间轴节点圆）、P5（三故障域容器 / 单元容器 / 节点 / 图例框）、P6（状态框）、P7（角色列头 / 汇总框）、P8（代码窗 + 窗口按钮圆）、P9（工况框）、P10（四阶段卡 / 决议框） |
| `elementType: line` + `arrow` 两端箭头 | 全 10 页；箭头类型见下 |
| `arrow: triangle`（同步业务流） | P5（Anycast→SH01/SH02/SZ01 三条垂直连线）、P7（消息 1~6） |
| `arrow: stealth`（异步 DTS 复制） | P5（跨单元 DTS 双向）、P7（消息 7、8） |
| `arrow: diamond`（仲裁租约 / 故障夺取） | P5（Quorum 租约）、P6（S1→S4 超时隔离） |
| `arrow: oval` / `arrow: stealth` 探针验证 | 独立探针页（见 §4 探针 P-ARROW） |
| `border.style: dash`（虚线语义区分） | P5、P7 |
| 折线 `points` 多段路径（`curve` 默认 round） | P6（S5→S1、S2→S5 环绕回路）、P7（消息 8 折线） |
| `elementType: table` + `columnWidths` / `rowHeights` | P4（4 列 × 8 行）、P9（4 列 × 6 行） |
| `elementType: image` + `fit: {mode: contain}` | P9（`assets/latency_cdf_curve.png`） |
| LaTeX 原生公式 `<latex>...</latex>`（`\frac`、`\int`、`\exp`、`\ge`、下标、`\text{}`） | P2（3 式）、P6（2 式） |
| 禁止项规避：`<ul>/<li>/<a>/<sup>/<sub>` 全部未使用；`elementType: chart` 未使用 | 全稿（validator 无 leaked tag） |

**架构图箭头语义（P5，§6.2 要求「箭头必须有方向与含义」）**：
- `triangle` 实线 → 同步业务请求（gRPC / HTTPS，Anycast 边缘网关按 UID 分片下行）；
- `stealth` 虚线 → 异步数据复制（DTS / Binlog 批量抓取，跨省 36ms RTT）；
- `diamond` 实线 → 仲裁控制流（Raft Quorum 写租约，防脑裂）。
每条连线均配**独立文本盒**标注协议/数据/频率，且不穿越任何节点或文字（P5 连接线仅走节点间空走廊与垂直通道；P6 状态跃迁线走两列之间的干净走廊）。

---

## 3. 验证证据 (Verification Evidence)

全部命令在仓库根 `open-pptd-skill/` 下执行；`<venv>` = `/Users/jim/multica_workspaces_desktop-multica.shiquda.link/dev-b47e50fa6b19/dev-583-f6ebf1094e86/workdir/.venv/bin/python`。

**① 结构校验**
```
$ pptd check open-pptd-skill/tests/decks/06-tech-architecture-review/deck.pptd
open-pptd-skill/tests/decks/06-tech-architecture-review/deck.pptd: 10 page(s) validated — no issues
```

**② 编译**
```
$ pptd build open-pptd-skill/tests/decks/06-tech-architecture-review/deck.pptd \
      -o open-pptd-skill/tests/decks/06-tech-architecture-review/build/deck.pptx --force
wrote open-pptd-skill/tests/decks/06-tech-architecture-review/build/deck.pptx
  sync point recorded: ... = 2473506d2c351ce802aad3659c3d7e22a3ff2ea5a740632354c319152937def6
```

**③ 成品质检（页数 + 备注 + 泄漏标签 + 越界）**
```
$ <venv> open-pptd-skill/skills/open-pptd/scripts/validate_pptx.py \
      open-pptd-skill/tests/decks/06-tech-architecture-review/build/deck.pptx \
      --expected-pages 10 --check-notes
Status: [PASS]  Slides: 10  Errors: 0  Warnings: 0
Slide 1..10: [Notes: Yes]  (text/img/tbl 计数见输出)
All checks passed cleanly! No leaked tags or boundary errors detected.
```
观测：P4 `1 tbl`、P9 `1 img + 1 tbl`，与设计一致；每页标题抽取正确（中文未被截断）。

**④ 无头渲染 + 逐页目视复核**
```
$ <venv> .../validate_pptx.py .../build/deck.pptx --render-images .../build/preview
→ preview/slide-01.png … slide-10.png（2001×1125）+ preview/contact-sheet.jpg
```
逐页放大阅读后修复的问题（均已重渲染确认）：
- P5：Anycast 网关原为居中窄框，导致三条下行连线斜穿单元标题 → 重排为跨三单元的水平网关 + 三条**纯垂直**连线，`UID 0~255/256~511/512~767 (gRPC)` 标签上移至 `y=128` 空带；
- P5：Quorum 连线原斜穿 SZ01 节点与其标签 → 改走底部空走廊 `y=490`，标签右对齐置于线左侧；
- P5：三处 DB 节点文案（3 行）在 28px 盒内溢出 → 盒高 38px、字号 7.2；
- P6：`专线中断 / 租约超时强制隔离` 标签压在 S1→S4 斜线上 → 移至两状态框之间空带；`仲裁授权新单元接管` 标签压在 S4→S5 斜线上 → 移至斜线左侧；
- P1/P3/P10：单字孤行（`期。` / `线。` / `值吞吐。`）→ 调整措辞与断行。

**⑤ 资产生成**（本地 matplotlib，未下载任何外部图片）
```
$ <venv> /tmp/generate_charts.py
→ assets/latency_cdf_curve.png (1920×1140, 300dpi)
```

---

## 4. 缺陷与粗糙点 (Defects & Rough Edges)

### D1 — `customFonts` 被文档与 AST 支持，却被 `check` 硬拦截（**严重 / 高**）
**症状**：按 `SKILL.md`、`reference/fonts.md`（§Custom fonts）与 `reference/pptd.md:349-353` 声明 `customFonts` 后，`pptd check` 直接报错，编译链第一步即失败。

**最小复现**：
```bash
mkdir -p /tmp/probe_cf && cat > /tmp/probe_cf/deck.pptd <<'EOF'
version: v2
title: "Probe"
size: [960, 540]
customFonts:
  - family: "JetBrains Mono"
    src: "https://fonts.googleapis.com/css2?family=JetBrains+Mono"
pages:
  - 01.page
EOF
pptd check /tmp/probe_cf/deck.pptd
```
**实测输出**：
```
- unknown field `customFonts` in deck manifest
check failed: 1 error(s)   # 退出码 2
```
**疑似层级**：**编译器**（`src/pptd/validate.rs:1118-1127` 的 `VALID_PRES_KEYS` 白名单缺 `customFonts`），而 `src/pptd/ast.rs:23-24` 已定义 `custom_fonts: Vec<CustomFont>`（`#[serde(rename_all="camelCase")]`）→ 校验白名单与 AST 不同步。**次要问题**：即便绕过校验，`pptd` 也未把 Google Fonts 拉取/嵌入包内，仅在 DrawingML 中写出原始 `typeface`（实测 `pptx/slides/slide1.xml` 含 `typeface="JetBrains Mono"`），离线渲染端只能靠系统回退 → 文档所暗示的「引入等宽字体」在本工具链下**实际不可达**。
**影响**：技术类稿件最常用的代码/配置等宽排版能力缺失；本稿被迫降级为结构化代码窗 + 纯文本对齐。

### D2 — 同一份清单改名即绕过未知字段校验（**中**）
**症状**：同一份含 `customFonts` 的清单，文件名为 `deck.pptd` 时报错；改名为 `test_custom_font.pptd` 时 `pptd check` 通过并成功 `build`。
**最小复现**：
```bash
pptd check /tmp/test_custom_font.pptd   # → 1 page(s) validated — no issues
pptd build /tmp/test_custom_font.pptd -o /tmp/x.pptx --force   # → wrote /tmp/x.pptx
```
**疑似层级**：**编译器**（`validate.rs:1105-1141` 的 `check_presentation_raw_yaml` 只扫描 entry candidates，疑似仅匹配 `deck.pptd` 等固定文件名），导致校验强度随文件名变化，CI 中易出现「本地过、改名后不过」的假阴性。

### D3 — `sys_dot` 是未文档化的线型枚举值（**低**）
`reference/pptd.md:199` 与 `:1138` 只声明 `LineStyle = "solid" | "dash" | "dot"`；实际编译器接受 `sys_dot`。以 `dashed`（文档外的自然拼写）书写时错误信息为：
```
error: failed to parse pages/05_...page as PPTD: elements: unknown variant `dashed`,
expected one of `solid`, `dash`, `dot`, `sys_dot` at line 10 column 3
```
**评估**：错误信息**准确且有帮助**（列出了全部合法值），属**文档缺口**而非编译器缺陷：建议把 `sys_dot` 补入 `pptd.md` 的 `LineStyle` 定义。

### D4 — 内置字体无等宽族，且无可用替代路径（**中**）
`reference/fonts.md` 的英文/中文/混排三张表共 20 款字体全部为无衬线/衬线/书法/像素，**无 monospace**；唯一建议路径是 `customFonts`（被 D1 阻断）。**综合结论**：当前工具链下无法产出「真等宽」代码排版。本稿的折中方案：代码窗容器 + 行号 + 手工对齐 + `$surfaceMuted` 底，并在 P8 明写该发现。

### D5 — `pptd check` 的溢出启发式偏保守（**低 / 观察**）
`check` 对多行文本盒给出溢出预警（例：`estimated text height ~40pt exceeds box height 28pt by 43%`、`~34pt exceeds 20pt by 68%`）。这些预警**方向正确且有用**（三处 DB 节点确实溢出，已修），但估算按行高上限计算，对刻意紧凑的 2~3 行节点会偏保守；渲染实测无裁切。属可用但略吵的粗糙点。

### 未复现的项
- `elementType: chart`（编译器按文档明确报错拦截）与 `animations`（仅解析不写时序）为**文档已声明的限制**，本稿未使用，故不列为缺陷。
- 未发现编译器 panic、崩溃、几何拉伸（`viewBox`/bounds）或箭头渲染异常；P5/P6/P7 的三种箭头（triangle / stealth / diamond）在 LibreOffice 无头渲染下方向、端点与形状均正确。
