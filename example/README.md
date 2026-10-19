# PPTD Examples

This directory contains PPTD example projects and test decks.

The `.pptx` presentations are regenerable build outputs compiled from the `.pptd` source files and `assets/`. They are not committed to the repository to keep clone sizes minimal.

## Rebuilding Examples

You can compile each example using the `pptd` CLI.

### From repository root:

```bash
# 1. Starter template
pptd build example/starter/deck.pptd -o example/starter/starter.pptx

# 2. DJI Osmo Pocket 4
pptd build example/dji-pocket4/dji-pocket4.pptd -o "example/dji-pocket4/DJI Osmo Pocket 4 产品深度解读.pptx"

# 3. Xiaomi YU7 (with animation)
pptd build example/xiaomi-yu7-ppt-animation/xiaomi-yu7.pptd -o example/xiaomi-yu7-ppt-animation/xiaomi-yu7.pptx

# 4. Xiaomi YU7 (comprehensive deck)
pptd build example/yu7-ppt/yu7.pptd -o example/yu7-ppt/yu7.pptx
```

### From within each example directory:

```bash
# Starter template
cd example/starter && pptd build deck.pptd -o starter.pptx

# DJI Osmo Pocket 4
cd example/dji-pocket4 && pptd build dji-pocket4.pptd -o "DJI Osmo Pocket 4 产品深度解读.pptx"

# Xiaomi YU7 (with animation)
cd example/xiaomi-yu7-ppt-animation && pptd build xiaomi-yu7.pptd

# Xiaomi YU7 (comprehensive deck)
cd example/yu7-ppt && pptd build yu7.pptd
```

> **Note**: If an output `.pptx` already exists and was modified or compiled without a matching sync hash, pass `--force` (or `-f`) to overwrite.
