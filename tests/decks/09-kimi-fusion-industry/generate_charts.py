import matplotlib.pyplot as plt
import numpy as np

plt.rcParams['font.sans-serif'] = ['Arial', 'Helvetica', 'DejaVu Sans']
plt.rcParams['axes.edgecolor'] = '#94A3B8'
plt.rcParams['axes.linewidth'] = 0.8
plt.rcParams['grid.color'] = '#E2E8F0'
plt.rcParams['grid.linestyle'] = '--'
plt.rcParams['grid.linewidth'] = 0.6

# -------------------------------------------------------------------------
# Chart 1: Triple Product Progress (Slide 4)
# -------------------------------------------------------------------------
fig, ax = plt.subplots(figsize=(7.6, 4.4), dpi=300)
fig.patch.set_facecolor('#FFFFFF')
ax.set_facecolor('#F8FAFC')

points = [
    (1968, 2.0e17, "T-3 (USSR)", "MCF"),
    (1978, 1.2e18, "PLT (US)", "MCF"),
    (1986, 1.5e19, "TFTR (US)", "MCF"),
    (1991, 4.5e19, "JET PTE (EU)", "MCF"),
    (1994, 1.8e20, "TFTR DT (10.7 MW)", "MCF"),
    (1997, 4.2e20, "JET DTE1 (16.1 MW, Q=0.67)", "MCF"),
    (1998, 1.4e21, "JT-60U (JP, equiv Q=1.25)", "MCF"),
    (2021, 3.8e20, "JET DTE2 (59 MJ)", "MCF"),
    (2022.9, 3.2e21, "NIF Ignition (Q=1.54)", "ICF"),
    (2023.8, 4.3e20, "JET DTE3 (69.3 MJ)", "MCF"),
]

years = [p[0] for p in points]
values = [p[1] for p in points]

ax.plot(years[:7], values[:7], color='#0064BC', linewidth=1.8, linestyle='-', alpha=0.85, label='MCF Historical Run')
ax.plot([1997, 2021, 2023.8], [values[5], values[7], values[9]], color='#0064BC', linewidth=1.5, linestyle=':', alpha=0.7)

for yr, val, lbl, cat in points:
    c = '#00295F' if cat == 'MCF' else '#F17E00'
    m = 'o' if cat == 'MCF' else 'D'
    ax.scatter(yr, val, color=c, s=55, marker=m, zorder=5, edgecolors='#FFFFFF', linewidth=1)

ax.annotate('1968 T-3\nTokamak breakthrough', xy=(1968, 2.0e17), xytext=(1966, 7.0e17),
            fontsize=7, color='#1E293B', fontweight='bold',
            arrowprops=dict(arrowstyle='->', color='#64748B', lw=0.7))

ax.annotate('1997 JET DTE1\n16.1 MW, Q=0.67', xy=(1997, 4.2e20), xytext=(1982, 8.0e20),
            fontsize=7, color='#00295F', fontweight='bold',
            arrowprops=dict(arrowstyle='->', color='#0064BC', lw=0.7))

ax.annotate('2022 Dec NIF\nTarget Ignition Q=1.54', xy=(2022.9, 3.2e21), xytext=(2006, 5.5e21),
            fontsize=7.5, color='#F17E00', fontweight='bold',
            arrowprops=dict(arrowstyle='->', color='#F17E00', lw=0.8))

ax.annotate('2023 JET DTE3\n69.3 MJ Record', xy=(2023.8, 4.3e20), xytext=(2012, 1.2e20),
            fontsize=7, color='#00295F',
            arrowprops=dict(arrowstyle='->', color='#64748B', lw=0.7))

ax.scatter([2027], [2.5e21], color='#00ACEE', s=70, marker='*', zorder=5, edgecolors='#FFFFFF', label='SPARC Target (Q>2)')
ax.annotate('CFS SPARC Target (2027)', xy=(2027, 2.5e21), xytext=(2008, 1.8e21),
            fontsize=7, color='#0064BC', fontstyle='italic',
            arrowprops=dict(arrowstyle='->', color='#00ACEE', lw=0.7))

ax.axhline(y=3.0e21, color='#DC2626', linestyle='--', linewidth=1.2, zorder=3)
ax.text(1966, 3.4e21, r'Lawson Ignition Threshold (DT: ~3 $\times$ 10$^{21}$ keV$\cdot$s$\cdot$m$^{-3}$)',
        color='#DC2626', fontsize=7.0, fontweight='bold')

ax.set_yscale('log')
ax.set_ylim(5e16, 1.5e22)
ax.set_xlim(1965, 2031)
ax.set_xlabel('Calendar Year', fontsize=7.8, color='#334155', fontweight='bold')
ax.set_ylabel(r'Triple Product $n \cdot T \cdot \tau_E$ ($\mathrm{keV}\cdot\mathrm{s}\cdot\mathrm{m}^{-3}$)',
              fontsize=7.8, color='#334155', fontweight='bold', labelpad=6)
ax.grid(True, which="both", axis="y", alpha=0.5)
ax.grid(True, which="major", axis="x", alpha=0.4)
ax.tick_params(axis='both', which='major', labelsize=7.2)

fig.text(0.18, 0.02, 'Source: Nuclear Fusion database (IAEA), EUROfusion JET records (2023), LLNL NIF ignition data (2022).',
         fontsize=6.2, color='#64748B', style='italic')

plt.subplots_adjust(left=0.18, right=0.96, top=0.95, bottom=0.15)
plt.savefig('open-pptd-skill/tests/decks/09-kimi-fusion-industry/assets/chart_triple_product.png', dpi=300)
plt.close()

# -------------------------------------------------------------------------
# Chart 2: Cumulative Private Fusion Investment (Slide 8)
# -------------------------------------------------------------------------
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(7.8, 4.2), dpi=300, gridspec_kw={'width_ratios': [1.1, 1.0]})
fig.patch.set_facecolor('#FFFFFF')
ax1.set_facecolor('#F8FAFC')
ax2.set_facecolor('#F8FAFC')

inv_years = ['2021', '2022', '2023', '2024', '2025 (E)']
inv_totals = [1.87, 4.86, 6.21, 7.12, 8.20]
annual_add = [1.87, 2.99, 1.35, 0.91, 1.08]

bars = ax1.bar(inv_years, inv_totals, color='#0064BC', width=0.55, edgecolor='#00295F', linewidth=0.8, alpha=0.9)
bars[-1].set_color('#00ACEE')
bars[-1].set_edgecolor('#0064BC')

for bar, val, add in zip(bars, inv_totals, annual_add):
    h = bar.get_height()
    ax1.text(bar.get_x() + bar.get_width()/2., h + 0.15, f"${val:.2f}B",
             ha='center', va='bottom', fontsize=7.0, fontweight='bold', color='#00295F')
    if val > 2.0:
        ax1.text(bar.get_x() + bar.get_width()/2., h/2., f"+${add:.2f}B\nyear",
                 ha='center', va='center', fontsize=6.0, color='#FFFFFF', fontweight='bold')

ax1.set_ylim(0, 9.8)

# Funding by Architecture
tech_categories = ['MCF (Tokamak/Stell.)', 'MIF / FRC (Helion/TAE)', 'ICF Laser (Target)', 'Other (Z-pinch)']
tech_shares = [62.5, 23.8, 10.5, 3.2]
colors_pie = ['#00295F', '#0064BC', '#F17E00', '#94A3B8']

bars2 = ax2.barh(tech_categories[::-1], tech_shares[::-1], color=colors_pie[::-1], height=0.52, edgecolor='#E2E8F0')
for bar, val in zip(bars2, tech_shares[::-1]):
    w = bar.get_width()
    if val > 40:
        ax2.text(w - 2.0, bar.get_y() + bar.get_height()/2., f"{val:.1f}%",
                 ha='right', va='center', fontsize=7.0, fontweight='bold', color='#FFFFFF')
    else:
        ax2.text(w + 1.5, bar.get_y() + bar.get_height()/2., f"{val:.1f}%",
                 ha='left', va='center', fontsize=7.0, fontweight='bold', color='#1E293B')

ax2.set_xlim(0, 75)
ax2.set_xlabel('% Share of Capital', fontsize=7.2, color='#334155', fontweight='bold', labelpad=5)
ax2.set_title('Capital by Architecture', fontsize=8.0, color='#00295F', fontweight='bold', pad=8)
ax2.grid(True, axis='x', alpha=0.5)
ax2.tick_params(labelsize=6.8)

fig.text(0.18, 0.02, 'Source: Fusion Industry Association (FIA) Reports (2021–2024); 2025 estimate based on Q3 2025 deals.',
         fontsize=5.8, color='#64748B', style='italic')

plt.subplots_adjust(left=0.18, right=0.92, top=0.90, bottom=0.16, wspace=0.48)
plt.savefig('open-pptd-skill/tests/decks/09-kimi-fusion-industry/assets/chart_fusion_investment.png', dpi=300)
plt.close()

# -------------------------------------------------------------------------
# Chart 3: Fusion LCOE Sensitivity (Slide 12)
# -------------------------------------------------------------------------
fig, ax = plt.subplots(figsize=(7.6, 4.3), dpi=300)
fig.patch.set_facecolor('#FFFFFF')
ax.set_facecolor('#F8FAFC')

occ = np.linspace(2000, 15000, 100)
crf = 0.0806
fixed_om = 120.0
var_om = 4.0

def calc_lcoe(occ_array, cf):
    annual_gen = 8760 * cf
    capital_cost_per_mwh = (occ_array * crf + fixed_om) / annual_gen * 1000
    return capital_cost_per_mwh + var_om

lcoe_90 = calc_lcoe(occ, 0.90)
lcoe_75 = calc_lcoe(occ, 0.75)
lcoe_60 = calc_lcoe(occ, 0.60)

ax.plot(occ, lcoe_90, color='#0064BC', linewidth=2.0, label='Capacity Factor 90% (Baseload)')
ax.plot(occ, lcoe_75, color='#F17E00', linewidth=1.8, linestyle='--', label='Capacity Factor 75% (Scheduled Swaps)')
ax.plot(occ, lcoe_60, color='#64748B', linewidth=1.5, linestyle=':', label='Capacity Factor 60% (Frequent Outages)')

ax.axhspan(45, 75, color='#10B981', alpha=0.12, label='Target Clean Baseload Market ($45–75/MWh)')
ax.axhline(y=75, color='#10B981', linestyle='-', linewidth=0.8, alpha=0.7)
ax.axhline(y=45, color='#10B981', linestyle='-', linewidth=0.8, alpha=0.7)

ax.axhspan(85, 120, color='#F59E0B', alpha=0.10, label='Gen-IV Fission SMR Benchmark ($85–120/MWh)')

# Target zones
ax.axvspan(3000, 4800, color='#0064BC', alpha=0.08)
ax.text(3900, 195, 'NOAK Target Zone\n($3,000–$4,800/kW)\nLCOE: $55–$78/MWh',
        ha='center', va='center', fontsize=6.6, color='#00295F', fontweight='bold',
        bbox=dict(boxstyle='round,pad=0.3', facecolor='#FFFFFF', edgecolor='#0064BC', alpha=0.9))

ax.axvspan(11500, 13800, color='#DC2626', alpha=0.08)
ax.text(12600, 195, 'FOAK Pilot Zone\n($12k–$15k/kW)\nLCOE: >$160/MWh',
        ha='center', va='center', fontsize=6.6, color='#DC2626', fontweight='bold',
        bbox=dict(boxstyle='round,pad=0.3', facecolor='#FFFFFF', edgecolor='#DC2626', alpha=0.9))

ax.set_xlim(2000, 14200)
ax.set_ylim(20, 220)
ax.set_xlabel('Overnight Capital Cost ($/kW)', fontsize=7.6, color='#334155', fontweight='bold')
ax.set_ylabel('Levelized Cost of Electricity ($/MWh)', fontsize=7.6, color='#334155', fontweight='bold', labelpad=6)
ax.grid(True, alpha=0.45)
ax.legend(loc='upper left', fontsize=6.3, framealpha=0.9, edgecolor='#CBD5E1')
ax.tick_params(labelsize=7.0)

fig.text(0.18, 0.02, 'Source: Sheffield et al. (Fusion Tech.) & ARPA-E ALPHA/BETHE reports (2024). Assumes WACC=7%, 30-yr life.',
         fontsize=5.8, color='#64748B', style='italic')

plt.subplots_adjust(left=0.18, right=0.94, top=0.95, bottom=0.15)
plt.savefig('open-pptd-skill/tests/decks/09-kimi-fusion-industry/assets/chart_fusion_lcoe.png', dpi=300)
plt.close()

print("All charts updated with perfect padding!")
