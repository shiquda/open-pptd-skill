import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
import os

os.makedirs('open-pptd-skill/tests/decks/01-analysis-industry-research/assets', exist_ok=True)

# Set high-quality font defaults
plt.rcParams['font.sans-serif'] = ['PingFang SC', 'Hiragino Sans GB', 'Heiti SC', 'Arial', 'DejaVu Sans']
plt.rcParams['axes.unicode_minus'] = False

# -------------------------------------------------------------
# Chart 1: Global BESS Additions by Region (2020-2030E)
# -------------------------------------------------------------
fig, ax = plt.subplots(figsize=(5.4, 3.98), dpi=200, layout='constrained')
ax.set_facecolor('#FFFFFF')

years = ['2020', '2021', '2022', '2023', '2024E', '2025E', '2026E', '2028E', '2030E']
x = np.arange(len(years))

china = np.array([3, 7, 16, 48, 78, 102, 130, 195, 275])
na = np.array([4, 11, 16, 32, 48, 68, 94, 145, 215])
europe = np.array([2, 5, 9, 16, 26, 38, 52, 85, 130])
row = np.array([2, 2, 3, 6, 13, 20, 29, 55, 100])

c_china = '#041C2C'   # Deep structural dark
c_na = '#2151FF'      # Primary royal blue
c_europe = '#00A9F4'  # Bright cyan blue
c_row = '#99C4FF'     # Pale soft blue

w = 0.58
b1 = china
b2 = china + na
b3 = china + na + europe
totals = b3 + row

ax.bar(x, china, width=w, label='China (中国)', color=c_china, edgecolor='none')
ax.bar(x, na, width=w, bottom=b1, label='North America (北美)', color=c_na, edgecolor='none')
ax.bar(x, europe, width=w, bottom=b2, label='Europe (欧洲)', color=c_europe, edgecolor='none')
ax.bar(x, row, width=w, bottom=b3, label='Rest of World (其他)', color=c_row, edgecolor='none')

# Totals label on key bars
for i, total in enumerate(totals):
    if i in [0, 2, 3, 4, 6, 8]:
        ax.text(i, total + 12, f"{int(total)}", ha='center', va='bottom', fontsize=9.5, fontweight='bold', color='#041C2C')

# Highlight CAGR from 2024E to 2030E
ax.annotate('', xy=(8, 735), xytext=(4, 180),
            arrowprops=dict(arrowstyle="-|>", color='#2151FF', lw=1.5, ls='--'))
ax.text(6.0, 480, '2024-2030E CAGR: +27.8%\n万亿储能超级赛道', 
        ha='center', va='center', fontsize=9, fontweight='bold', color='#2151FF',
        bbox=dict(boxstyle='square,pad=0.4', facecolor='#EEF2FF', edgecolor='#2151FF', lw=0.8))

# Milestone 100GWh label
ax.axhline(y=100, color='#94A3B8', linestyle=':', linewidth=0.9)
ax.text(0, 108, '100 GWh 历史跨越 (2023)', color='#64748B', fontsize=8, style='italic')

ax.set_ylabel('Annual Additions (GWh / 年新增装机)', fontsize=9.5, fontweight='bold', color='#041C2C')
ax.set_xticks(x)
ax.set_xticklabels(years, fontsize=9.5, color='#041C2C', fontweight='medium')
ax.set_ylim(0, 820)

# Spine & grid formatting (clean consulting style)
ax.spines['top'].set_visible(False)
ax.spines['right'].set_visible(False)
ax.spines['left'].set_color('#CBD5E1')
ax.spines['bottom'].set_color('#041C2C')
ax.spines['bottom'].set_linewidth(1.2)
ax.yaxis.grid(True, linestyle='-', alpha=0.35, color='#E2E8F0')
ax.xaxis.grid(False)

# Legend at top left
ax.legend(loc='upper left', frameon=True, facecolor='#FFFFFF', edgecolor='#E2E8F0', fontsize=8.5, ncol=2)


fig.savefig('open-pptd-skill/tests/decks/01-analysis-industry-research/assets/global_bess_additions.png', dpi=200)
plt.close(fig)

# -------------------------------------------------------------
# Chart 2: BESS System Cost vs Lithium Carbonate Price
# -------------------------------------------------------------
fig, ax1 = plt.subplots(figsize=(5.1, 3.98), dpi=200, layout='constrained')
fig.patch.set_facecolor('#FFFFFF')
ax1.set_facecolor('#FFFFFF')

periods = ['2020', '2021', '2022H1', '2022H2', '2023H1', '2023H2', '2024H1', '2024Q3']
xp = np.arange(len(periods))

# Lithium Carbonate Spot Price (10k RMB / ton)
li_prices = [4.5, 27.5, 48.0, 56.5, 31.0, 14.5, 10.2, 7.6]

# BESS 2h EPC System Cost (RMB / Wh)
epc_costs = [1.68, 1.45, 1.58, 1.62, 1.18, 0.86, 0.64, 0.56]

# Left axis: Lithium Carbonate (Area/bar or line)
color_li = '#94A3B8'
ax1.plot(xp, li_prices, color='#64748B', linewidth=2.0, marker='s', markersize=5, label='Battery-grade Li2CO3 (万元/吨)')
ax1.fill_between(xp, li_prices, alpha=0.12, color='#64748B')
ax1.set_ylabel('Li2CO3 Spot Price (万元/吨)', fontsize=9.5, fontweight='bold', color='#475569')
ax1.set_ylim(0, 65)
ax1.tick_params(axis='y', labelcolor='#475569')

# Annotate peak
ax1.annotate('峰值 56.5万元/吨\n(2022Q4)', xy=(3, 56.5), xytext=(1.15, 46.0),
             arrowprops=dict(arrowstyle="->", color='#B91C1C', lw=1.0),
             ha='center', fontsize=7.5, fontweight='bold', color='#B91C1C')

# Right axis: BESS EPC Cost
ax2 = ax1.twinx()
color_epc = '#2151FF'
ax2.plot(xp, epc_costs, color=color_epc, linewidth=2.6, marker='o', markersize=6, label='BESS 2h EPC Price (元/Wh)')
ax2.set_ylabel('2h BESS EPC System Bid (元/Wh)', fontsize=9.5, fontweight='bold', color=color_epc)
ax2.set_ylim(0.4, 2.0)
ax2.tick_params(axis='y', labelcolor=color_epc)

# Annotate current low
ax2.annotate('破0.60临界点: 0.56元/Wh\n较峰值下跌超65%', xy=(7, 0.56), xytext=(4.8, 0.72),
             arrowprops=dict(arrowstyle="->", color=color_epc, lw=1.2),
             ha='center', fontsize=8.5, fontweight='bold', color=color_epc,
             bbox=dict(boxstyle='square,pad=0.3', facecolor='#EEF2FF', edgecolor=color_epc, lw=0.8))

ax1.set_xticks(xp)
ax1.set_xticklabels(periods, fontsize=9.5, color='#041C2C', fontweight='medium')

# Spines
ax1.spines['top'].set_visible(False)
ax2.spines['top'].set_visible(False)
ax1.spines['left'].set_color('#CBD5E1')
ax2.spines['right'].set_color('#2151FF')
ax1.spines['bottom'].set_color('#041C2C')
ax1.yaxis.grid(True, linestyle='-', alpha=0.3, color='#E2E8F0')

# Combined legend
lines1, labels1 = ax1.get_legend_handles_labels()
lines2, labels2 = ax2.get_legend_handles_labels()
ax1.legend(lines1 + lines2, labels1 + labels2, loc='upper right', frameon=True, facecolor='#FFFFFF', edgecolor='#E2E8F0', fontsize=7.5)


fig.savefig('open-pptd-skill/tests/decks/01-analysis-industry-research/assets/bess_cost_curve_and_lithium.png', dpi=200)
plt.close(fig)
print("Charts generated successfully!")
