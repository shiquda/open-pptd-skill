import os
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

asset_dir = "/Users/jim/multica_workspaces_desktop-multica.shiquda.link/dev-b47e50fa6b19/dev-583-f6ebf1094e86/workdir/open-pptd-skill/tests/decks/07-brand-magazine/assets"
os.makedirs(asset_dir, exist_ok=True)

# 1. cover_texture.jpg: High-contrast monochrome organic texture (macro cellular / grain)
np.random.seed(30)
w, h = 1280, 720
x = np.linspace(-3, 3, w)
y = np.linspace(-3, 3, h)
X, Y = np.meshgrid(x, y)
Z = np.sin(X * 2.5 + Y * 1.5) * np.cos(X * 1.2 - Y * 2.8) + 0.3 * np.sin(X * 10) * np.cos(Y * 10)
turb = np.sin(np.sqrt(X**2 + Y**2) * 4) + 0.5 * np.random.randn(h, w)
Z_total = Z + 0.25 * turb
norm = (Z_total - Z_total.min()) / (Z_total.max() - Z_total.min())
contrast = 1 / (1 + np.exp(-10 * (norm - 0.48)))
img_arr = (contrast * 255).astype(np.uint8)

img_cover = Image.fromarray(img_arr, mode='L').convert('RGB')
grain = np.random.normal(0, 10, (h, w, 3)).astype(np.int16)
img_cover_arr = np.clip(np.array(img_cover, dtype=np.int16) + grain, 0, 255).astype(np.uint8)
img_cover = Image.fromarray(img_cover_arr)
img_cover.save(os.path.join(asset_dir, "cover_texture.jpg"), quality=95)

# 2. beans_macro.jpg: High-contrast macro of green/roasted coffee beans
img_beans = Image.new('RGB', (1000, 1000), color=(247, 245, 239))
draw = ImageDraw.Draw(img_beans)
np.random.seed(42)
for _ in range(50):
    bx = np.random.randint(100, 900)
    by = np.random.randint(100, 900)
    br = np.random.randint(45, 95)
    draw.ellipse([bx - br, by - int(br * 0.65), bx + br, by + int(br * 0.65)], fill=(24, 24, 24), outline=(40, 40, 40), width=3)
    draw.arc([bx - br*0.8, by - br*0.3, bx + br*0.8, by + br*0.3], start=20, end=160, fill=(247, 245, 239), width=5)

beans_arr = np.array(img_beans, dtype=np.int16)
beans_arr += np.random.normal(0, 8, beans_arr.shape).astype(np.int16)
beans_arr = np.clip(beans_arr, 0, 255).astype(np.uint8)
Image.fromarray(beans_arr).save(os.path.join(asset_dir, "beans_macro.jpg"), quality=95)

# 3. fermentation_tank.jpg: High-contrast industrial stainless steel & valve macro
img_tank = Image.new('RGB', (1000, 800), color=(18, 18, 20))
draw_t = ImageDraw.Draw(img_tank)
for i in range(0, 1000, 5):
    v = int(120 + 110 * np.sin(i / 70.0) * np.cos(i / 150.0))
    draw_t.line([(i, 0), (i, 800)], fill=(v, v, v), width=5)
draw_t.ellipse([340, 240, 660, 560], fill=(247, 245, 239), outline=(16, 16, 16), width=14)
draw_t.ellipse([365, 265, 635, 535], outline=(140, 140, 140), width=2)
# Klein blue indicator needle
draw_t.line([(500, 400), (600, 330)], fill=(0, 56, 184), width=8)
draw_t.ellipse([488, 388, 512, 412], fill=(16, 16, 16))
img_tank.save(os.path.join(asset_dir, "fermentation_tank.jpg"), quality=95)

# 4. harvest_landscape.jpg: High-altitude terraced plantation landscape
img_land = Image.new('RGB', (1280, 720), color=(247, 245, 239))
draw_l = ImageDraw.Draw(img_land)
for y_base in range(120, 750, 40):
    points = []
    for x_pt in range(0, 1281, 40):
        y_val = y_base + int(35 * np.sin(x_pt / 110.0 + y_base / 70.0) + 20 * np.cos(x_pt / 50.0))
        points.append((x_pt, y_val))
    points.append((1280, 720))
    points.append((0, 720))
    gray = int(25 + (y_base / 750.0) * 175)
    draw_l.polygon(points, fill=(gray, gray, gray), outline=(20, 20, 20))
img_land.save(os.path.join(asset_dir, "harvest_landscape.jpg"), quality=95)

# 5. altitude_chart.png: Swiss modernist chart for Page 4
fig, ax1 = plt.subplots(figsize=(7.2, 4.4), dpi=200)
fig.patch.set_facecolor('#F7F5EF')
ax1.set_facecolor('#F7F5EF')

altitudes = np.array([1200, 1400, 1600, 1800, 2000, 2200])
scores = np.array([82.4, 83.8, 85.2, 86.9, 88.6, 89.4])
sucrose = np.array([5.1, 5.8, 6.7, 7.8, 9.2, 9.8]) # Sucrose % dry weight

# Swiss bar styling (sucrose bars)
bar_width = 100
bars = ax1.bar(altitudes - 25, sucrose, width=65, color='#D8D7D2', edgecolor='#101010', linewidth=1.2, label='Sucrose Density (%)')

ax2 = ax1.twinx()
line = ax2.plot(altitudes + 25, scores, color='#0038B8', linewidth=3.5, marker='s', markersize=7, markerfacecolor='#0038B8', markeredgecolor='#101010', markeredgewidth=1.5, label='SCA Cupping Score (pts)')

ax1.set_xlabel('Altitude Elevation (MASL / 米)', fontsize=11, fontweight='bold', color='#101010', labelpad=8)
ax1.set_ylabel('Sucrose Content (% dry matter)', fontsize=11, fontweight='bold', color='#55514A', labelpad=8)
ax2.set_ylabel('SCA Cup Score (80-100)', fontsize=11, fontweight='bold', color='#0038B8', labelpad=8)

ax1.set_ylim(4, 12)
ax1.set_xlabel('Altitude Elevation (MASL / Meters)', fontsize=10, fontweight='bold', color='#101010', labelpad=8)
ax1.set_xticks(altitudes)
ax1.set_xticklabels([f'{a}m' for a in altitudes], fontsize=10, fontweight='bold', color='#101010')

ax1.spines['top'].set_visible(False)
ax2.spines['top'].set_visible(False)
ax1.spines['left'].set_color('#101010')
ax1.spines['bottom'].set_color('#101010')
ax2.spines['right'].set_color('#0038B8')
ax1.spines['left'].set_linewidth(1.5)
ax1.spines['bottom'].set_linewidth(1.5)
ax2.spines['right'].set_linewidth(1.5)

ax1.grid(axis='y', linestyle='--', alpha=0.5, color='#D8D7D2')

# Annotate 88.5+ threshold
ax2.axhline(88.0, color='#0038B8', linestyle=':', linewidth=1.2)
ax2.text(1250, 88.2, 'SPECIALTY BENCHMARK > 88.0 PTS', color='#0038B8', fontsize=8.5, fontweight='heavy')

# Combined legend in top-left
lines1, labels1 = ax1.get_legend_handles_labels()
lines2, labels2 = ax2.get_legend_handles_labels()
ax1.legend(lines1 + lines2, labels1 + labels2, loc='upper left', frameon=True, facecolor='#F7F5EF', edgecolor='#101010', fontsize=9)

plt.title('HIGH-ALTITUDE SUCROSE ACCUMULATION VS CUP SCORE', loc='left', fontsize=12, fontweight='heavy', color='#101010', pad=12)
fig.tight_layout()
fig.savefig(os.path.join(asset_dir, "altitude_chart.png"), facecolor=fig.get_facecolor(), edgecolor='none')
plt.close(fig)

# 6. roast_profile.png: Thermodynamic roast curve for Page 6
fig, ax1 = plt.subplots(figsize=(8.0, 4.6), dpi=200)
fig.patch.set_facecolor('#0A194E')
ax1.set_facecolor('#0A194E')

time_sec = np.linspace(0, 690, 200) # 0 to 11.5 minutes
# Bean Temp curve (starts 200C charge, dips to 92C at 75s, then rises to 208C)
bt = 200 - 108 * np.exp(-time_sec / 35.0) + (time_sec / 690.0) * 116
bt[time_sec > 75] = 92 + 116 * ((time_sec[time_sec > 75] - 75) / (690 - 75))**0.85

# RoR curve (Rate of Rise, C/min)
ror = np.clip(32 * np.exp(-time_sec / 240.0) + 4, 4, 30)

ax1.plot(time_sec / 60.0, bt, color='#FFFFFF', linewidth=3, label='Bean Temp (BT °C)')
ax2 = ax1.twinx()
ax2.plot(time_sec / 60.0, ror, color='#0038B8', linewidth=2.5, linestyle='--', label='RoR (°C/min)')

# Vertical milestone markers
milestones = [
    (1.25, 'TURNING POINT\n92°C', '#D8D7D2'),
    (4.5, 'YELLOWING\n152°C', '#D8D7D2'),
    (9.2, 'FIRST CRACK\n196°C', '#C68A4C'),
    (11.08, 'DROP\n208°C', '#FFFFFF')
]
for t_m, label, col in milestones:
    ax1.axvline(t_m, color='#334155', linestyle=':', linewidth=1)
ax1.set_xlabel('Roast Duration (Minutes:Seconds)', fontsize=10, fontweight='bold', color='#D8D7D2', labelpad=8)

ax1.set_ylabel('Bean Temperature (°C)', fontsize=10, fontweight='bold', color='#FFFFFF', labelpad=8)
ax2.set_ylabel('Rate of Rise (°C/min)', fontsize=10, fontweight='bold', color='#0038B8', labelpad=8)

ax1.set_xlim(0, 12)
ax1.set_ylim(80, 230)
ax2.set_ylim(0, 35)

ax1.tick_params(colors='#D8D7D2', labelsize=9)
ax2.tick_params(colors='#0038B8', labelsize=9)
for sp in ax1.spines.values():
    sp.set_color('#334155')
for sp in ax2.spines.values():
    sp.set_color('#334155')

ax1.grid(True, linestyle=':', alpha=0.2, color='#FFFFFF')

l1, lb1 = ax1.get_legend_handles_labels()
l2, lb2 = ax2.get_legend_handles_labels()
ax1.legend(l1 + l2, lb1 + lb2, loc='upper left', facecolor='#0A194E', edgecolor='#334155', fontsize=8.5, labelcolor='#FFFFFF')

plt.title('THERMODYNAMIC ROAST PROFILE // 30°N BAOSHAN TYPICA', loc='left', fontsize=11, fontweight='heavy', color='#FFFFFF', pad=12)
fig.tight_layout()
fig.savefig(os.path.join(asset_dir, "roast_profile.png"), facecolor=fig.get_facecolor(), edgecolor='none')
plt.close(fig)

# 7. sensory_wheel.png: Radar flavor profile for Page 7
categories = ['Floral\n(Jasmine)', 'Citrus\n(Bergamot)', 'Stone Fruit\n(Peach)', 'Sweetness\n(Raw Sugar)', 
              'Acidity\n(Malic)', 'Body\n(Silky)', 'Finish\n(Black Tea)', 'Cleanliness\n(Crystalline)']
N = len(categories)
angles = [n / float(N) * 2 * np.pi for n in range(N)]
angles += angles[:1]

values = [9.2, 8.8, 8.9, 9.0, 8.7, 8.4, 9.1, 9.3]
values += values[:1]

fig, ax = plt.subplots(figsize=(6.2, 5.0), subplot_kw=dict(polar=True), dpi=200)
fig.patch.set_facecolor('#F7F5EF')
ax.set_facecolor('#F7F5EF')

ax.set_theta_offset(np.pi / 2)
ax.set_theta_direction(-1)

plt.xticks(angles[:-1], categories, color='#101010', size=9, fontweight='heavy')
ax.set_rscale('linear')
plt.yticks([6, 7, 8, 9, 10], ["6", "7", "8", "9", "10"], color="#888880", size=8)
plt.ylim(5, 10)

ax.spines['polar'].set_color('#101010')
ax.spines['polar'].set_linewidth(1.5)
ax.grid(color='#D8D7D2', linestyle='-', linewidth=1)

ax.plot(angles, values, color='#0038B8', linewidth=2.5, linestyle='solid')
ax.fill(angles, values, color='#0038B8', alpha=0.35)

plt.title('30°N MICRO-LOT SENSORY RADAR (SCA 89.2 PTS)', size=11, fontweight='heavy', color='#101010', y=1.12)
fig.tight_layout()
fig.savefig(os.path.join(asset_dir, "sensory_wheel.png"), facecolor=fig.get_facecolor(), edgecolor='none')
plt.close(fig)

print("All assets successfully created.")
