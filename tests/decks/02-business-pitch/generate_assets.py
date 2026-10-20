import os
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
from PIL import Image, ImageDraw

asset_dir = "open-pptd-skill/tests/decks/02-business-pitch/assets"
os.makedirs(asset_dir, exist_ok=True)

# 1. Abstract background for Slide 5 (full-bleed divider)
w, h = 1920, 1080
bg_img = Image.new("RGBA", (w, h), "#032817")
draw = ImageDraw.Draw(bg_img)

for i in range(12):
    alpha = int(25 + i * 15)
    color = (41, 185, 116, alpha)  # bright green accent
    x1 = -200 + i * 140
    y1 = h + 100
    x2 = w // 2 + i * 80
    y2 = -100
    draw.polygon([(x1, y1), (x1 + 180, y1), (x2 + 240, y2), (x2, y2)], fill=color)

overlay = Image.new("RGBA", (w, h), (3, 40, 23, 140))
bg_img = Image.alpha_composite(bg_img, overlay)

draw_grid = ImageDraw.Draw(bg_img)
for y in range(0, h, 60):
    draw_grid.line([(0, y), (w, y)], fill=(209, 245, 226, 18), width=1)
for x in range(0, w, 120):
    draw_grid.line([(x, 0), (x, h)], fill=(209, 245, 226, 12), width=1)

bg_img.convert("RGB").save(f"{asset_dir}/abstract_bg.png", "PNG", quality=95)
print("Generated abstract_bg.png")

# 2. Market Perishable Loss & Cold Storage Capacity Chart
plt.rcParams['font.sans-serif'] = ['DejaVu Sans', 'Arial']
fig, ax1 = plt.subplots(figsize=(8.2, 5.4), dpi=220)

regions = ['SEA (ASEAN-6)', 'Vietnam', 'Indonesia', 'Thailand', 'China', 'N. America']
loss_rates = [32.0, 34.5, 33.2, 28.0, 15.2, 9.8] # % perishable loss
storage_cap = [0.028, 0.025, 0.021, 0.042, 0.145, 0.360] # m3 per capita

x = np.arange(len(regions))
width = 0.42

colors = ['#E71C56', '#E71C56', '#E71C56', '#E71C56', '#29B974', '#03522C']
bars = ax1.bar(x - width/2, loss_rates, width, label='Perishable Food Loss Rate (%)', color=colors, alpha=0.92, edgecolor='none')

ax2 = ax1.twinx()
line = ax2.plot(x + width/2, storage_cap, color='#D3DF33', marker='o', linewidth=2.6, markersize=7, label='Cold Storage Capacity (m³/capita)')

# Title with generous top margin
ax1.set_title('Southeast Asia Cold Chain Deficit: Loss Rate vs Storage Capacity', fontsize=12, weight='bold', color='#03522C', pad=22)
ax1.set_ylabel('Perishable Loss Rate (%)', fontsize=9.5, color='#565656', weight='bold')
ax2.set_ylabel('Cold Storage Cap (m³/capita)', fontsize=9.5, color='#565656', weight='bold')
ax1.set_xticks(x)
ax1.set_xticklabels(regions, fontsize=9, color='#333333', weight='bold')
ax1.set_ylim(0, 42)
ax2.set_ylim(0, 0.42)

for bar in bars:
    height = bar.get_height()
    ax1.annotate(f'{height:.1f}%',
                xy=(bar.get_x() + bar.get_width() / 2, height),
                xytext=(0, 4), textcoords="offset points",
                ha='center', va='bottom', fontsize=8.5, weight='bold', color='#222222')

for i, txt in enumerate(storage_cap):
    ax2.annotate(f'{txt:.3f}',
                xy=(x[i] + width/2, txt),
                xytext=(0, 6), textcoords="offset points",
                ha='center', va='bottom', fontsize=8, weight='bold', color='#4A6000')

ax1.grid(axis='y', linestyle='--', alpha=0.35, color='#CCCCCC')
ax1.spines['top'].set_visible(False)
ax2.spines['top'].set_visible(False)
ax1.spines['left'].set_color('#999999')
ax2.spines['right'].set_color('#999999')
ax1.spines['bottom'].set_color('#999999')

lines1, labels1 = ax1.get_legend_handles_labels()
lines2, labels2 = ax2.get_legend_handles_labels()
ax1.legend(lines1 + lines2, labels1 + labels2, loc='upper right', frameon=True, facecolor='#FFFFFF', edgecolor='#E0E0E0', fontsize=8)

plt.subplots_adjust(top=0.88, bottom=0.12, left=0.10, right=0.90)
fig.savefig(f"{asset_dir}/market_perishable_loss.png", dpi=220, facecolor='#FFFFFF')
plt.close(fig)
print("Generated market_perishable_loss.png")

# 3. Telematics & IoT Dashboard Mockup for Slide 8
fig, (ax_top, ax_bot) = plt.subplots(2, 1, figsize=(8.2, 5.4), dpi=220, gridspec_kw={'height_ratios': [1.8, 1.2]})

hours = np.linspace(0, 24, 96)
np.random.seed(42)
temp_ideal = -18.0 + 0.6 * np.sin(hours * 0.5) + np.random.normal(0, 0.2, len(hours))
temp_trad = -18.0 + 0.8 * np.sin(hours * 0.4)
breach_mask = (hours >= 7.5) & (hours <= 13.5)
temp_trad[breach_mask] = -18.0 + 11.5 * np.exp(-((hours[breach_mask]-10.5)/2.0)**2)

ax_top.plot(hours, temp_ideal, color='#29B974', linewidth=2.4, label='ColdSync Managed (PT FreshMart Fleet #42)')
ax_top.plot(hours, temp_trad, color='#E71C56', linewidth=2.0, linestyle='--', label='Legacy Unmonitored Fleet (Breach Alert)')
ax_top.axhline(y=-15.0, color='#E71C56', linestyle=':', alpha=0.7, label='Critical Breach Threshold (-15°C)')

ax_top.fill_between(hours, -22, -15, color='#D1F5E2', alpha=0.35, label='Compliant Safe Zone (-22°C to -15°C)')
ax_top.set_xlim(0, 24)
ax_top.set_ylim(-24, -4)
ax_top.set_title('Real-time Cold Chain Telemetry: Reefer Temp Log (Jakarta → Bandung 180km)', fontsize=11, weight='bold', color='#03522C', pad=18)
ax_top.set_xlabel('Mission Timeline (Hours)', fontsize=8.5, color='#565656')
ax_top.set_ylabel('Chamber Temp (°C)', fontsize=8.5, color='#565656')
ax_top.grid(True, linestyle='--', alpha=0.35, color='#CCCCCC')
ax_top.legend(loc='upper right', fontsize=7.2, framealpha=0.9)

ax_top.annotate('ColdSync IoT Alert: Compressor Restarted in 6 mins\nAvoided $4,200 Salmon Batch Spoilage',
                xy=(10.5, -7.5), xytext=(12.0, -6.5),
                arrowprops=dict(facecolor='#03522C', shrink=0.08, width=1.5, headwidth=5),
                fontsize=7.2, weight='bold', color='#03522C',
                bbox=dict(boxstyle="round,pad=0.3", fc="#B8EDBC", ec="#29B974", lw=1))

speed = 45 + 15 * np.sin(hours * 0.8) + np.random.normal(0, 3, len(hours))
speed = np.clip(speed, 0, 80)
compressor_duty = 85 + 10 * np.sin(hours * 0.3) + np.random.normal(0, 2, len(hours))

ax_bot.plot(hours, speed, color='#03522C', linewidth=1.5, label='Vehicle Speed (km/h)')
ax_bot.plot(hours, compressor_duty, color='#2FC1D6', linewidth=1.5, label='Compressor Duty Cycle (%)')
ax_bot.set_xlim(0, 24)
ax_bot.set_ylim(0, 110)
ax_bot.set_xlabel('Hours', fontsize=8.5, color='#565656')
ax_bot.set_ylabel('Telemetry (%)', fontsize=8.5, color='#565656')
ax_bot.grid(True, linestyle='--', alpha=0.35, color='#CCCCCC')
ax_bot.legend(loc='lower right', fontsize=7.2, framealpha=0.9)

plt.subplots_adjust(top=0.90, bottom=0.12, left=0.10, right=0.95, hspace=0.35)
fig.savefig(f"{asset_dir}/fleet_iot_dashboard.png", dpi=220, facecolor='#FFFFFF')
plt.close(fig)
print("Generated fleet_iot_dashboard.png")
