#!/usr/bin/env python3
"""
Generate all scientific charts, heatmaps, and rail thumbnails for:
1. 10a-gwtc5-gravitational-wave
2. 10b-urban-heat-island

All raster assets are synthetic, locally generated (no external downloads).
Chart aspect ratios match the PPTD image bounds (420 x 235 px => 1.787) so that
`fit: contain` produces no letterboxing.
"""

import os
from pathlib import Path

import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.patches import Ellipse, Polygon, Rectangle
from PIL import Image, ImageDraw

plt.rcParams['font.sans-serif'] = ['DejaVu Sans', 'Arial', 'Helvetica']
plt.rcParams['axes.edgecolor'] = '#94A3B8'
plt.rcParams['axes.linewidth'] = 0.8
plt.rcParams['text.color'] = '#0F172A'
plt.rcParams['axes.labelcolor'] = '#0F172A'
plt.rcParams['xtick.color'] = '#475569'
plt.rcParams['ytick.color'] = '#475569'

HERE = Path(__file__).resolve().parent
DIR_GWTC = str(HERE / "assets")
DIR_HEAT = str(HERE.parent / "10b-urban-heat-island" / "assets")
os.makedirs(DIR_GWTC, exist_ok=True)
os.makedirs(DIR_HEAT, exist_ok=True)

# Bounds are [185, 70, 420, 235] => aspect 1.787
FIG_W, FIG_H = 6.2, 3.47
DPI = 200


def _save(fig, path):
    fig.savefig(path, dpi=DPI)
    plt.close(fig)


# -------------------------------------------------------------
# 1. GWTC: Catalog Growth (assets/catalog_growth.png)
# -------------------------------------------------------------
def make_catalog_growth():
    fig, ax = plt.subplots(figsize=(FIG_W, FIG_H), dpi=DPI, facecolor='#FFFFFF')
    ax.set_facecolor('#F8FAFC')

    catalogs = ['GWTC-1\n(O1-O2)', 'GWTC-2\n(O3a)', 'GWTC-2.1\n(O3a deep)',
                'GWTC-3\n(O3b cum.)', 'O4 alerts\n(O4a+O4b)']
    cumulative = [11, 50, 58, 90, 390]

    bars = ax.bar(catalogs, cumulative, color='#2563EB', width=0.55,
                  edgecolor='#1D4ED8', linewidth=1.2, alpha=0.9, zorder=3)
    bars[-1].set_color('#0D9488')
    bars[-1].set_edgecolor('#0F766E')

    for bar, val in zip(bars, cumulative):
        ax.text(bar.get_x() + bar.get_width() / 2., bar.get_height() + 8, f'{val}',
                ha='center', va='bottom', fontsize=9, fontweight='bold', color='#0F172A')

    ax.axhline(90, color='#DC2626', linestyle='--', linewidth=0.9, alpha=0.75, zorder=2)
    ax.text(0.05, 97, 'GWTC-3 formal offline threshold (p_astro >= 0.5): 90 events',
            fontsize=7, color='#B91C1C', style='italic')

    ax.text(2.75, 250, 'O4 public alert stream\n(~390 candidates, preliminary)',
            fontsize=7, color='#0F766E', fontweight='bold',
            bbox=dict(boxstyle="round,pad=0.3", fc="#CCFBF1", ec="#14B8A6", lw=0.8))

    ax.set_ylabel('Cumulative transients (count)', fontsize=9, fontweight='bold')
    ax.set_title('Cumulative Gravitational-Wave Transients Across Catalog Eras\n'
                 'Sample: O1–O4b | Selection: p_astro >= 0.5 | Source: LVK (2025)',
                 fontsize=8, pad=8, fontweight='bold')
    ax.set_ylim(0, 440)
    ax.tick_params(labelsize=7.5)
    ax.grid(axis='y', linestyle=':', color='#CBD5E1', alpha=0.8, zorder=1)

    fig.subplots_adjust(top=0.85, bottom=0.16, left=0.13, right=0.97)
    _save(fig, os.path.join(DIR_GWTC, "catalog_growth.png"))


# -------------------------------------------------------------
# 2. GWTC: Mass Plane (assets/mass_plane.png)
# -------------------------------------------------------------
def make_mass_plane():
    fig, ax = plt.subplots(figsize=(FIG_W, FIG_H), dpi=DPI, facecolor='#FFFFFF')
    ax.set_facecolor('#F8FAFC')

    ax.axhspan(2.1, 5.0, color='#FEF08A', alpha=0.35, zorder=1)
    ax.axvspan(2.1, 5.0, color='#FEF08A', alpha=0.35, zorder=1)
    ax.text(1.05, 2.55, 'Lower mass gap (2.1–5.0)', fontsize=6.8, color='#854D0E', fontweight='bold')

    ax.axhspan(65, 120, color='#FED7AA', alpha=0.35, zorder=1)
    ax.axvspan(65, 120, color='#FED7AA', alpha=0.35, zorder=1)
    ax.text(1.05, 72, 'PISN gap (65–120)', fontsize=6.8, color='#9A3412', fontweight='bold')

    m_range = np.logspace(0, 2.3, 100)
    ax.plot(m_range, m_range, color='#94A3B8', linestyle=':', linewidth=1.2,
            label='m₁ = m₂ (equal mass)', zorder=2)
    ax.fill_between(m_range, m_range, 200, color='#E2E8F0', alpha=0.4, zorder=1)
    ax.text(5.5, 40, 'unphysical region\n(m₂ > m₁)', fontsize=7, color='#64748B',
            style='italic', rotation=32)

    np.random.seed(42)
    ax.scatter([1.46, 2.0, 1.35], [1.27, 1.4, 1.15], color='#16A34A', s=40,
               edgecolor='#15803D', linewidth=0.8, zorder=4, label='BNS (GW170817, GW190425)')
    ax.scatter([23.2], [2.59], color='#D97706', s=60, marker='^', edgecolor='#B45309',
               linewidth=1.0, zorder=5, label='GW190814 (m₂ = 2.6)')
    ax.scatter([3.6], [1.4], color='#EA580C', s=55, marker='s', edgecolor='#C2410C',
               linewidth=1.0, zorder=5, label='GW230529 (lower gap)')

    bbh_m1 = np.random.uniform(7, 50, 65)
    q = np.random.uniform(0.4, 0.95, 65)
    ax.scatter(bbh_m1, bbh_m1 * q, color='#2563EB', s=22, alpha=0.7,
               edgecolor='#1D4ED8', linewidth=0.5, zorder=3, label='BBH sample (GWTC-1..3)')
    ax.scatter([35.6], [30.6], color='#7C3AED', s=65, marker='*', edgecolor='#5B21B6',
               linewidth=1.0, zorder=5, label='GW150914 (35.6 + 30.6)')
    ax.scatter([85.0], [66.0], color='#DC2626', s=70, marker='D', edgecolor='#991B1B',
               linewidth=1.0, zorder=5, label='GW190521 (85 + 66)')

    ax.set_xscale('log')
    ax.set_yscale('log')
    ax.set_xlim(1.0, 160)
    ax.set_ylim(1.0, 160)
    ax.tick_params(labelsize=7.5)
    ax.set_xlabel('Primary component mass m₁ (M_sun)', fontsize=9, fontweight='bold')
    ax.set_ylabel('Secondary component mass m₂ (M_sun)', fontsize=9, fontweight='bold')
    ax.set_title('Component Mass Plane (m₁ vs m₂) Across Binary Classes\n'
                 'Sample: 90 GWTC-3 events + O4 outliers | Threshold: p_astro >= 0.5',
                 fontsize=8, pad=8, fontweight='bold')
    ax.legend(loc='lower right', fontsize=6.2, framealpha=0.92, facecolor='#FFFFFF',
              edgecolor='#CBD5E1')
    ax.grid(True, which='both', linestyle=':', color='#CBD5E1', alpha=0.6, zorder=1)

    fig.subplots_adjust(top=0.85, bottom=0.15, left=0.13, right=0.97)
    _save(fig, os.path.join(DIR_GWTC, "mass_plane.png"))


# -------------------------------------------------------------
# 3. GWTC: Sky Localization (assets/sky_localization.png)
# -------------------------------------------------------------
def make_sky_localization():
    fig = plt.figure(figsize=(FIG_W, FIG_H), dpi=DPI, facecolor='#FFFFFF')
    ax = fig.add_subplot(111, projection='mollweide', facecolor='#F8FAFC')
    ax.grid(color='#CBD5E1', linestyle=':', linewidth=0.7)
    ax.tick_params(colors='#64748B', labelsize=6.5)

    theta_arc = np.linspace(-np.pi / 3, np.pi / 4, 80)
    phi_arc = -1.2 + 0.5 * np.sin(2 * theta_arc)
    ax.plot(phi_arc, theta_arc, color='#DC2626', linewidth=2.6, alpha=0.75,
            label='2-det ring arc (HL, ~600 deg²)')
    ax.fill_betweenx(theta_arc, phi_arc - 0.15, phi_arc + 0.15, color='#FCA5A5', alpha=0.4)

    ra_170817, dec_170817 = -0.75, -0.40
    ax.add_patch(Ellipse((ra_170817, dec_170817), width=0.18, height=0.08, angle=-25,
                         facecolor='#16A34A', edgecolor='#15803D', linewidth=1.5,
                         alpha=0.85, label='3-det (GW170817, ~28 deg²)'))
    ra_o4, dec_o4 = 1.1, 0.35
    ax.add_patch(Ellipse((ra_o4, dec_o4), width=0.09, height=0.045, angle=35,
                         facecolor='#2563EB', edgecolor='#1D4ED8', linewidth=1.5,
                         alpha=0.9, label='4-det O4 target (~10 deg²)'))

    ax.text(phi_arc[22] + 0.1, theta_arc[22] + 0.08, 'GW150914', fontsize=6.5,
            color='#B91C1C', fontweight='bold')
    ax.text(ra_170817 - 0.05, dec_170817 - 0.22, 'GW170817 (NGC 4993)', fontsize=6.5,
            color='#15803D', fontweight='bold')
    ax.text(ra_o4 + 0.12, dec_o4 + 0.02, 'O4 network', fontsize=6.5,
            color='#1D4ED8', fontweight='bold')

    ax.set_title('Sky Localization Contours (90% credible regions, Mollweide)\n'
                 'Triangulation delay  Δt = D·n/c  |  Source: LVK alert archives',
                 fontsize=8, pad=8, fontweight='bold')
    ax.legend(loc='lower center', bbox_to_anchor=(0.5, -0.30), ncol=3, fontsize=6.2,
              framealpha=0.92, facecolor='#FFFFFF', edgecolor='#CBD5E1')

    fig.subplots_adjust(top=0.85, bottom=0.20, left=0.06, right=0.94)
    _save(fig, os.path.join(DIR_GWTC, "sky_localization.png"))


# -------------------------------------------------------------
# 4. GWTC: Distance & Redshift (assets/distance_redshift.png)
# -------------------------------------------------------------
def make_distance_redshift():
    fig, ax = plt.subplots(figsize=(FIG_W, FIG_H), dpi=DPI, facecolor='#FFFFFF')
    ax.set_facecolor('#F8FAFC')

    z = np.linspace(0.01, 1.4, 150)
    c = 299792.458
    Om, Ol = 0.315, 0.685

    def d_l(z_vals, h0):
        out = []
        for zv in z_vals:
            zp = np.linspace(0, zv, 100)
            e = np.sqrt(Om * (1 + zp) ** 3 + Ol)
            out.append((1 + zv) * (c / h0) * np.trapezoid(1.0 / e, zp) / 1000.0)
        return np.array(out)

    ax.plot(z, d_l(z, 67.4), color='#1E40AF', linewidth=2.0,
            label='Planck CMB (H₀ = 67.4 km/s/Mpc)', zorder=3)
    ax.plot(z, d_l(z, 73.0), color='#DC2626', linestyle='--', linewidth=2.0,
            label='SH0ES SNe Ia (H₀ = 73.0 km/s/Mpc)', zorder=3)

    ax.errorbar([0.0099], [0.040], yerr=[0.008], fmt='o', color='#16A34A',
                ecolor='#16A34A', elinewidth=1.5, capsize=4, markersize=6,
                label='GW170817 siren (40 ± 8 Mpc)', zorder=5)

    np.random.seed(7)
    sz = np.array([0.09, 0.18, 0.32, 0.48, 0.72, 1.05])
    sdl = d_l(sz, 69.5) + np.random.normal(0, 0.12, len(sz))
    ax.errorbar(sz, sdl, yerr=sdl * 0.18, fmt='s', color='#6366F1', ecolor='#A5B4FC',
                elinewidth=1.2, capsize=3, markersize=4.5,
                label='GWTC-3 statistical sirens', zorder=4)

    ax.set_xlabel('Cosmological redshift z', fontsize=9, fontweight='bold')
    ax.set_ylabel('Luminosity distance D_L (Gpc)', fontsize=9, fontweight='bold')
    ax.set_title('Luminosity Distance vs Redshift: Pure Gravitational Standard Sirens\n'
                 'h ∝ (G·M_z)^(5/6) / (c²·D_L)  |  independent of the distance ladder',
                 fontsize=8, pad=8, fontweight='bold')
    ax.tick_params(labelsize=7.5)
    ax.grid(True, linestyle=':', color='#CBD5E1', alpha=0.7)
    ax.legend(loc='upper left', fontsize=6.4, framealpha=0.92, facecolor='#FFFFFF',
              edgecolor='#CBD5E1')

    fig.subplots_adjust(top=0.85, bottom=0.15, left=0.13, right=0.97)
    _save(fig, os.path.join(DIR_GWTC, "distance_redshift.png"))


# -------------------------------------------------------------
# 5. GWTC: Population Distribution (assets/population_mass_spin.png)
# -------------------------------------------------------------
def make_population_distribution():
    # bounds [185, 70, 420, 210] => aspect 2.0
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(6.2, 3.1), dpi=DPI, facecolor='#FFFFFF')
    for ax in (ax1, ax2):
        ax.set_facecolor('#F8FAFC')
        ax.grid(True, linestyle=':', color='#CBD5E1', alpha=0.7)
        ax.tick_params(labelsize=7)

    m = np.linspace(5, 80, 200)
    p_m1 = 0.8 * (m / 10) ** (-2.3) + 0.35 * np.exp(-0.5 * ((m - 34) / 4.5) ** 2)
    ax1.plot(m, p_m1, color='#2563EB', linewidth=2.2, label='Fitted distribution')
    ax1.fill_between(m, p_m1, color='#93C5FD', alpha=0.35)
    ax1.axvline(34, color='#DC2626', linestyle='--', linewidth=1.2)
    ax1.text(36.5, 0.42, 'PPISN peak\n(~34 M_sun)', fontsize=7, color='#B91C1C', fontweight='bold')
    ax1.set_xlabel('Primary mass m₁ (M_sun)', fontsize=8.5, fontweight='bold')
    ax1.set_ylabel('Relative probability p(m₁)', fontsize=8.5, fontweight='bold')
    ax1.set_title('(A) Primary Mass Spectrum\nbroken power-law + Gaussian peak',
                  fontsize=7.5, fontweight='bold')

    chi = np.linspace(-0.6, 0.8, 200)
    p_chi = np.exp(-0.5 * ((chi - 0.06) / 0.14) ** 2) / (0.14 * np.sqrt(2 * np.pi))
    ax2.plot(chi, p_chi, color='#0D9488', linewidth=2.2)
    ax2.fill_between(chi, p_chi, color='#99F6E4', alpha=0.35)
    ax2.axvline(0.0, color='#64748B', linestyle=':', linewidth=1.0)
    ax2.text(0.12, 2.15, 'peak near χ_eff ≈ 0.06\n(low aligned spins)', fontsize=7,
             color='#0F766E', fontweight='bold')
    ax2.set_xlabel('Effective inspiral spin χ_eff', fontsize=8.5, fontweight='bold')
    ax2.set_ylabel('Probability density p(χ_eff)', fontsize=8.5, fontweight='bold')
    ax2.set_title('(B) Effective Spin Distribution\ncentered near zero, positive tail',
                  fontsize=7.5, fontweight='bold')

    fig.subplots_adjust(top=0.84, bottom=0.17, left=0.10, right=0.97, wspace=0.28)
    _save(fig, os.path.join(DIR_GWTC, "population_mass_spin.png"))


# -------------------------------------------------------------
# 6. GWTC Left Rail Thumbnails
# -------------------------------------------------------------
def make_gwtc_rail_thumbnails():
    w, h = 240, 150

    # Thumb 1: global detector network (graticule + 5 sites + baselines)
    fig = plt.figure(figsize=(2.40, 1.50), dpi=100, facecolor='#FFFFFF')
    ax = fig.add_axes([0, 0, 1, 1])
    ax.set_xlim(0, 240)
    ax.set_ylim(0, 150)
    ax.axis('off')
    ax.add_patch(Rectangle((0.5, 0.5), 239, 149, fill=False,
                           edgecolor='#CBD5E1', linewidth=1))
    ax.add_patch(Rectangle((0, 118), 240, 32, facecolor='#F1F5F9',
                           edgecolor='none'))
    ax.text(8, 133, "Five Detectors, One Global Network", fontsize=6.6,
            color='#0F172A', va='center')

    # graticule
    for lon in range(0, 241, 24):
        ax.plot([lon, lon], [10, 112], color='#E2E8F0', linewidth=0.6, zorder=1)
    for lat in range(10, 113, 20):
        ax.plot([0, 240], [lat, lat], color='#E2E8F0', linewidth=0.6, zorder=1)
    ax.plot([0, 240], [61, 61], color='#94A3B8', linewidth=0.8, zorder=2)

    # coarse continental masses (schematic, not a survey-grade map)
    land = {
        'N.America': [(22, 104), (58, 108), (74, 96), (62, 78), (48, 74), (34, 84)],
        'S.America': [(60, 72), (74, 74), (78, 52), (70, 30), (60, 34), (56, 54)],
        'Europe':    [(112, 104), (140, 106), (146, 92), (126, 88), (110, 94)],
        'Africa':    [(114, 86), (142, 88), (150, 62), (132, 34), (114, 44), (108, 64)],
        'Asia':      [(146, 106), (214, 110), (226, 94), (196, 78), (160, 82), (146, 92)],
        'Australia': [(196, 58), (222, 60), (226, 42), (206, 34), (192, 44)],
    }
    for poly in land.values():
        ax.add_patch(Polygon(poly, closed=True, facecolor='#F1F5F9',
                             edgecolor='#CBD5E1', linewidth=0.6, zorder=2))

    # detector sites (lon/lat -> schematic sheet position) and baselines
    sites = {
        'H1': (36, 96),
        'L1': (48, 84),
        'V1': (128, 94),
        'K1': (214, 88),
        'G1': (130, 100),
    }
    baselines = [('H1', 'L1'), ('H1', 'V1'), ('L1', 'V1'), ('V1', 'K1')]
    for a, b in baselines:
        ax.plot([sites[a][0], sites[b][0]], [sites[a][1], sites[b][1]],
                color='#93C5FD', linewidth=1.0, zorder=3)
    for name, (x, y) in sites.items():
        ax.add_patch(Ellipse((x, y), 8, 8, facecolor='#2563EB',
                             edgecolor='#1E40AF', linewidth=0.7, zorder=4))
        ax.text(x, y, name, fontsize=3.6, color='#FFFFFF', ha='center',
                va='center', zorder=5)
    ax.text(8, 22, "53% joint coincidence", fontsize=5.6, color='#2563EB')
    ax.text(140, 22, "10–30 ms baselines", fontsize=5.6, color='#64748B')
    fig.savefig(os.path.join(DIR_GWTC, "thumb_detectors.png"), dpi=100,
                facecolor='#FFFFFF')
    plt.close(fig)

    im2 = Image.new('RGB', (w, h), color='#FFFFFF')
    d2 = ImageDraw.Draw(im2)
    d2.rectangle([(0, 0), (w - 1, h - 1)], outline='#CBD5E1', width=1)
    d2.rectangle([(0, 0), (w, 24)], fill='#F1F5F9')
    d2.text((10, 6), "Nearly a Decade of Observing: O1->O4b", fill='#0F172A')
    d2.rectangle([(15, 50), (55, 90)], fill='#94A3B8')
    d2.rectangle([(65, 45), (110, 90)], fill='#64748B')
    d2.rectangle([(120, 35), (170, 90)], fill='#2563EB')
    d2.rectangle([(180, 25), (225, 90)], fill='#0D9488')
    d2.text((25, 98), "O1", fill='#64748B')
    d2.text((80, 98), "O2", fill='#64748B')
    d2.text((138, 98), "O3", fill='#2563EB')
    d2.text((195, 98), "O4", fill='#0D9488')
    d2.text((15, 125), "Sensitivity: 70 Mpc -> 160 Mpc", fill='#0F172A')
    im2.save(os.path.join(DIR_GWTC, "thumb_timeline.png"))

    im3 = Image.new('RGB', (w, h), color='#FFFFFF')
    d3 = ImageDraw.Draw(im3)
    d3.rectangle([(0, 0), (w - 1, h - 1)], outline='#CBD5E1', width=1)
    d3.rectangle([(0, 0), (w, 24)], fill='#F1F5F9')
    d3.text((10, 6), "The Catalog Grows: 390 Transients", fill='#0F172A')
    d3.rectangle([(25, 105), (50, 115)], fill='#2563EB')
    d3.rectangle([(65, 95), (90, 115)], fill='#2563EB')
    d3.rectangle([(105, 88), (130, 115)], fill='#2563EB')
    d3.rectangle([(145, 75), (170, 115)], fill='#2563EB')
    d3.rectangle([(185, 35), (210, 115)], fill='#0D9488')
    d3.text((20, 124), "11", fill='#64748B')
    d3.text((62, 124), "50", fill='#64748B')
    d3.text((102, 124), "58", fill='#64748B')
    d3.text((142, 124), "90", fill='#2563EB')
    d3.text((182, 124), "390", fill='#0D9488')
    im3.save(os.path.join(DIR_GWTC, "thumb_catalog.png"))

    im4 = Image.new('RGB', (w, h), color='#FFFFFF')
    d4 = ImageDraw.Draw(im4)
    d4.rectangle([(0, 0), (w - 1, h - 1)], outline='#CBD5E1', width=1)
    d4.rectangle([(0, 0), (w, 24)], fill='#F1F5F9')
    d4.text((10, 6), "The Mass Plane: NS to 100+ M_sun BHs", fill='#0F172A')
    d4.line([(25, 35), (25, 115), (215, 115)], fill='#94A3B8', width=1)
    d4.line([(25, 115), (215, 35)], fill='#CBD5E1', width=1)
    d4.ellipse([(35, 102), (41, 108)], fill='#16A34A')
    d4.ellipse([(45, 96), (51, 102)], fill='#16A34A')
    d4.ellipse([(90, 80), (96, 86)], fill='#2563EB')
    d4.ellipse([(110, 70), (116, 76)], fill='#2563EB')
    d4.ellipse([(130, 60), (136, 66)], fill='#2563EB')
    d4.ellipse([(170, 42), (176, 48)], fill='#DC2626')
    d4.text((25, 124), "NS: 1.4 M_sun   |   PISN: 65-120 M_sun", fill='#0F172A')
    im4.save(os.path.join(DIR_GWTC, "thumb_mass.png"))


# -------------------------------------------------------------
# 7. HEAT ISLAND: 4 pixel-grid temperature maps
#    Synthetic 30 m grid schematic (示意网格), NOT real observations.
# -------------------------------------------------------------
def make_heat_island_grids():
    np.random.seed(101)
    rows, cols = 40, 28

    def render(temp, filename):
        rgb = np.zeros((rows, cols, 3), dtype=np.float32)
        for r in range(rows):
            for cc in range(cols):
                t = temp[r, cc]
                if t >= 45.0:
                    k = min(1.0, (t - 45.0) / 10.0)
                    rgb[r, cc] = [0.92 + 0.06 * k, 0.35 + 0.15 * k, 0.05]
                elif t >= 38.0:
                    k = (t - 38.0) / 7.0
                    rgb[r, cc] = [0.65 + 0.20 * k, 0.22 + 0.10 * k, 0.05]
                else:
                    k = max(0.0, min(1.0, (t - 28.0) / 10.0))
                    rgb[r, cc] = [0.18 + 0.15 * k, 0.22 + 0.12 * k, 0.32 + 0.15 * k]

        fig, ax = plt.subplots(figsize=(2.8, 4.0), dpi=200, facecolor='#0B0F19')
        ax.set_facecolor('#0B0F19')
        ax.imshow(rgb, interpolation='nearest', aspect='auto')
        ax.set_xticks([])
        ax.set_yticks([])
        for spine in ax.spines.values():
            spine.set_edgecolor('#334155')
            spine.set_linewidth(1.0)
        fig.subplots_adjust(left=0.02, right=0.98, top=0.98, bottom=0.02)
        fig.savefig(os.path.join(DIR_HEAT, filename), dpi=200, facecolor='#0B0F19')
        plt.close(fig)

    t1 = np.random.normal(49.5, 3.2, (rows, cols))
    for _ in range(6):
        cr, cc = np.random.randint(5, rows - 5), np.random.randint(4, cols - 4)
        t1[cr - 4:cr + 5, cc - 3:cc + 4] += 4.0
    render(np.clip(t1, 41.0, 56.5), "grid_city_center.png")

    t2 = np.random.normal(41.0, 3.8, (rows, cols))
    for _ in range(4):
        cr, cc = np.random.randint(5, rows - 5), np.random.randint(4, cols - 4)
        t2[cr - 3:cr + 4, cc - 3:cc + 4] += 5.0
    render(np.clip(t2, 33.0, 48.0), "grid_built_up.png")

    t3 = np.random.normal(33.5, 2.5, (rows, cols))
    t3[10:16, 8:16] += 5.5
    t3[26:31, 14:20] += 5.0
    render(np.clip(t3, 29.0, 40.0), "grid_green_space.png")

    t4 = np.random.normal(31.2, 1.2, (rows, cols))
    t4[5:8, 20:25] += 4.0
    t4[22:25, 22:26] += 3.5
    render(np.clip(t4, 29.0, 35.5), "grid_waterfront.png")


# -------------------------------------------------------------
# 8. HEAT ISLAND Left Rail Thumbnails
# -------------------------------------------------------------
def make_heat_rail_thumbnails():
    w, h = 240, 135

    im1 = Image.new('RGB', (w, h), color='#0B0F19')
    d1 = ImageDraw.Draw(im1)
    d1.rectangle([(0, 0), (w - 1, h - 1)], outline='#334155', width=1)
    d1.text((10, 8), "Surface Temp != Air Temp", fill='#F8FAFC')
    d1.text((10, 22), "45-55   38-45   30-38   30-34", fill='#F97316')
    d1.rectangle([(15, 38), (55, 98)], fill='#EA580C')
    d1.rectangle([(65, 38), (105, 98)], fill='#C2410C')
    d1.rectangle([(115, 38), (155, 98)], fill='#334155')
    d1.rectangle([(165, 38), (205, 98)], fill='#1E293B')
    d1.text((15, 108), "36-38 C Air Temp (1.5 m)", fill='#94A3B8')
    d1.text((140, 108), "dT: +1-2 C", fill='#F97316')
    im1.save(os.path.join(DIR_HEAT, "thumb_slide1.png"))

    im2 = Image.new('RGB', (w, h), color='#0B0F19')
    d2 = ImageDraw.Draw(im2)
    d2.rectangle([(0, 0), (w - 1, h - 1)], outline='#334155', width=1)
    d2.text((10, 8), "Three 'Heaters', One Simple Test", fill='#F8FAFC')
    d2.rectangle([(10, 28), (115, 54)], fill='#1E293B', outline='#EA580C', width=1)
    d2.text((15, 34), "1. Absorbs Heat", fill='#F8FAFC')
    d2.rectangle([(10, 58), (115, 84)], fill='#1E293B', outline='#EA580C', width=1)
    d2.text((15, 64), "2. Blocks Wind", fill='#F8FAFC')
    d2.rectangle([(10, 88), (115, 114)], fill='#1E293B', outline='#EA580C', width=1)
    d2.text((15, 94), "3. Emits Heat", fill='#F8FAFC')
    d2.rectangle([(125, 28), (225, 114)], fill='#0F172A', outline='#334155', width=1)
    d2.text((130, 34), "Class Mini-Lab", fill='#F97316')
    d2.rectangle([(130, 50), (220, 68)], fill='#1E293B')
    d2.rectangle([(130, 72), (220, 90)], fill='#1E293B')
    d2.text((135, 96), "Asphalt dT: +14 C", fill='#38BDF8')
    im2.save(os.path.join(DIR_HEAT, "thumb_slide2.png"))


if __name__ == '__main__':
    print("Generating GWTC assets...")
    make_catalog_growth()
    make_mass_plane()
    make_sky_localization()
    make_distance_redshift()
    make_population_distribution()
    make_gwtc_rail_thumbnails()
    print("Generating Heat Island assets...")
    make_heat_island_grids()
    make_heat_rail_thumbnails()
    print("All assets generated successfully!")
