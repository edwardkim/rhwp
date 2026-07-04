# Task M100-1352 Visual Comparison

Sample: `samples/hwpx/hy-001.hwpx`, page 1.

- `before/`: rendered from `upstream/devel` (`d8b40eff`).
- `after/`: rendered from current task commit (`829a6a3e`).
- `compare/before_after_header_cell_compare.png`: enlarged crop of the affected first table cell.
- `compare/before_after_page1_compare.png`: full-page context comparison.

Key SVG coordinate:

- Before image y: `102.56`
- After image y: `81.23`

The after image aligns with the text y coordinate and no longer drops below the cell clip.
