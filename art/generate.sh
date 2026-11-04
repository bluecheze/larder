#!/bin/sh
# Paints art/source.png. Needs mflux (`uv tool install mflux`) and about
# 10 GB of free memory. The style reference isn't in the repo because it's
# someone else's artwork, bring your own.
set -e
cd "$(dirname "$0")"
STYLE_REF="${1:?usage: generate.sh <style-reference-image>}"

mflux-generate-flux2-edit --model flux2-klein-4b -q 4 --low-ram --width 1536 --height 512 --seed 8 \
  --image-paths mascot.png "$STYLE_REF" \
  --prompt "Wide panoramic illustration in the exact art style of the second image: crisp black ink linework, dense fine pen hatching, flat bold screen print colors, sharp edges, no blur. The same young man from the first image (messy brown hair, white button shirt, dark jeans, white sneakers, orange backpack on his back) stands on a small wooden step stool on the right side, placing a cardboard box on a tall wooden shelf in a cozy old pantry larder. The shelves are full of glass jars, wooden crates, labeled boxes and stacked folders. Warm golden sunlight comes in from a small window, dust in the light. The left third of the image is a calm plain plaster wall with soft light and nothing on it. Palette: warm wood brown, cream, golden yellow, deep cobalt blue shadows, orange accent." \
  --output source.png
