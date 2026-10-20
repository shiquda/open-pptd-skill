import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np

# Set font
plt.rcParams['font.sans-serif'] = ['PingFang SC', 'Heiti SC', 'Hiragino Sans GB', 'Arial']
plt.rcParams['axes.unicode_minus'] = False

# Colors from warm-jade design system
BG_COLOR = '#FBF9EE'
INK = '#181716'
AMBER = '#FDC356'
GRAY_LIGHT = '#CCC5BE'
GRAY_MID = '#A09B93'
GRAY_DARK = '#6B625D'
GREEN = '#2D6A4F'
RED = '#B83A2C'

# --- Asset 1: Cohort Retention Mix (Page 3) ---
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(7.2, 3.8), dpi=200, gridspec_kw={'width_ratios': [1.3, 1]})
fig.patch.set_facecolor(BG_COLOR)
ax1.set_facecolor(BG_COLOR)
ax2.set_facecolor(BG_COLOR)

# Subplot 1: Cohort retention curves over quarters
quarters = ['Q0', 'Q1', 'Q2', 'Q3', 'Q4', 'Q5', 'Q6', 'Q7', 'Q8']
smb_retention = [100, 104, 108, 111, 114, 115, 116, 116.4, 116.4]
mm_retention = [100, 101, 103, 104, 105, 105.5, 106, 105.8, 105.8]
ka_retention = [100, 106, 110, 108, 104, 99, 97.5, 96.2, 96.2]

ax1.plot(quarters, smb_retention, marker='o', markersize=4, color=GREEN, linewidth=2, label='SMB 标准客户 (NRR 116.4%)')
ax1.plot(quarters, mm_retention, marker='s', markersize=4, color=GRAY_DARK, linewidth=1.8, linestyle='--', label='Mid-Market 腰部 (NRR 105.8%)')
ax1.plot(quarters, ka_retention, marker='^', markersize=5, color=RED, linewidth=2.2, label='KA 战略大客户 (NRR 96.2%)')

ax1.axhline(100, color=GRAY_LIGHT, linestyle=':', linewidth=1)
ax1.set_title('各客群同梯队净金额留存率 (NDR/NRR 走势)', fontsize=10, fontweight='bold', color=INK, pad=10)
ax1.set_ylabel('金额留存率 (%)', fontsize=8, color=GRAY_DARK)
ax1.set_ylim(90, 122)
ax1.tick_params(axis='both', which='major', labelsize=8, colors=GRAY_DARK)
ax1.spines['top'].set_visible(False)
ax1.spines['right'].set_visible(False)
ax1.spines['left'].set_color(GRAY_LIGHT)
ax1.spines['bottom'].set_color(GRAY_LIGHT)
ax1.legend(loc='upper left', fontsize=7.5, frameon=False)

# Annotate turning point for KA
ax1.annotate('Q4交付延期发酵\n留存跌破100%', xy=('Q5', 99), xytext=('Q4', 92),
             arrowprops=dict(facecolor=RED, shrink=0.08, width=0.8, headwidth=4),
             fontsize=7, color=RED, fontweight='bold')

# Subplot 2: ARR Contribution & NRR Bar Comparison
categories = ['KA (>500万)', '腰部 (100-500万)', 'SMB (<100万)']
nrr_vals = [96.2, 105.8, 116.4]
bar_colors = [RED, GRAY_MID, GREEN]

y_pos = np.arange(len(categories))
bars = ax2.barh(y_pos, nrr_vals, height=0.45, color=bar_colors, edgecolor='none')
ax2.axvline(100, color=GRAY_LIGHT, linestyle='--', linewidth=1)
ax2.axvline(108, color=AMBER, linestyle=':', linewidth=1.2, label='Q3 集团目标 108%')

ax2.set_yticks(y_pos)
ax2.set_yticklabels(categories, fontsize=8, color=INK)
ax2.set_xlim(85, 125)
ax2.set_title('客群 Q3 NRR 达成现状 vs 目标', fontsize=10, fontweight='bold', color=INK, pad=10)
ax2.tick_params(axis='x', which='major', labelsize=8, colors=GRAY_DARK)
ax2.spines['top'].set_visible(False)
ax2.spines['right'].set_visible(False)
ax2.spines['left'].set_color(GRAY_LIGHT)
ax2.spines['bottom'].set_color(GRAY_LIGHT)
ax2.legend(loc='lower right', fontsize=7, frameon=False)

for bar in bars:
    w = bar.get_width()
    ax2.text(w + 1.2, bar.get_y() + bar.get_height()/2, f"{w:.1f}%",
             va='center', ha='left', fontsize=8, fontweight='bold', color=INK)

plt.tight_layout()
plt.savefig('open-pptd-skill/tests/decks/03-management-okr-review/assets/cohort_retention_mix.png', facecolor=BG_COLOR, edgecolor='none')
plt.close()
print("Generated cohort_retention_mix.png")

# --- Asset 2: CAC Payback & Funnel (Page 7) ---
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(7.2, 3.8), dpi=200, gridspec_kw={'width_ratios': [1.2, 1]})
fig.patch.set_facecolor(BG_COLOR)
ax1.set_facecolor(BG_COLOR)
ax2.set_facecolor(BG_COLOR)

# Subplot 1: CAC Payback Trend (Quarters)
quarters_cac = ['25Q1', '25Q2', '25Q3', '25Q4', '26Q1', '26Q2', '26Q3']
cac_values = [12.4, 12.8, 13.5, 14.1, 15.2, 16.4, 17.5]

ax1.plot(quarters_cac, cac_values, marker='o', markersize=5, color=RED, linewidth=2.2, label='实际回本周期 (月)')
ax1.axhline(14.0, color=GREEN, linestyle='--', linewidth=1.5, label='健康阈值上限 (14.0个月)')
ax1.axhspan(14.0, 19.0, color='#FEE2E2', alpha=0.4, label='超标恶化区间')

ax1.set_title('获客 CAC 回本周期演变趋势 (月)', fontsize=10, fontweight='bold', color=INK, pad=10)
ax1.set_ylabel('回本周期 (月)', fontsize=8, color=GRAY_DARK)
ax1.set_ylim(10, 19)
ax1.tick_params(axis='both', which='major', labelsize=8, colors=GRAY_DARK)
ax1.spines['top'].set_visible(False)
ax1.spines['right'].set_visible(False)
ax1.spines['left'].set_color(GRAY_LIGHT)
ax1.spines['bottom'].set_color(GRAY_LIGHT)
ax1.legend(loc='upper left', fontsize=7.5, frameon=False)

for q, v in zip(quarters_cac, cac_values):
    if q in ('25Q2', '25Q3'):
        ax1.text(q, v - 0.45, f"{v}m", ha='center', va='top', fontsize=7.5, fontweight='bold', color=INK)
    elif q == '25Q4':
        ax1.text(q, v + 0.55, f"{v}m", ha='center', va='bottom', fontsize=7.5, fontweight='bold', color=INK)
    else:
        ax1.text(q, v + 0.35, f"{v}m", ha='center', va='bottom', fontsize=7.5, fontweight='bold', color=INK)

# Subplot 2: Pipeline Stage Conversion Funnel (KA vs Standard)
stages = ['MQL 获客', 'SQL 线索', 'POC 验证', '商务谈判', '签约成交']
ka_conv = [100, 48, 22, 12, 6.2]
std_conv = [100, 62, 45, 32, 21.0]

x = np.arange(len(stages))
width = 0.35

ax2.barh(x + width/2, std_conv, width, label='SMB/标准客 (%)', color=GRAY_MID)
ax2.barh(x - width/2, ka_conv, width, label='KA 复杂大客 (%)', color=AMBER)

ax2.set_yticks(x)
ax2.set_yticklabels(stages, fontsize=8, color=INK)
ax2.set_xlim(0, 115)
ax2.set_title('商机漏斗转化率 (POC重度堵塞)', fontsize=10, fontweight='bold', color=INK, pad=10)
ax2.tick_params(axis='x', which='major', labelsize=8, colors=GRAY_DARK)
ax2.spines['top'].set_visible(False)
ax2.spines['right'].set_visible(False)
ax2.spines['left'].set_color(GRAY_LIGHT)
ax2.spines['bottom'].set_color(GRAY_LIGHT)
ax2.legend(loc='lower right', fontsize=7, frameon=False)

for i in range(len(stages)):
    ax2.text(ka_conv[i] + 1.5, x[i] - width/2, f"{ka_conv[i]}%", va='center', fontsize=7, fontweight='bold', color=INK)
    ax2.text(std_conv[i] + 1.5, x[i] + width/2, f"{std_conv[i]}%", va='center', fontsize=7, color=GRAY_DARK)

plt.tight_layout()
plt.savefig('open-pptd-skill/tests/decks/03-management-okr-review/assets/cac_payback_and_funnel.png', facecolor=BG_COLOR, edgecolor='none')
plt.close()
print("Generated cac_payback_and_funnel.png")
