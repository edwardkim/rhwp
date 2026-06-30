#1658 centered-cell over-count 회귀 fixture (합성, 실문서 비포함)
마커 CENTERME 의 y0(PDF pt) — BUG=clean devel 30931679 / FIX=참고 수정본

- centered_cell_nested_table.hwpx        Center  over(중첩표1)  BUG 107.7(상단) / FIX 185.7(중앙)
- cell_vcenter_multi_nested_overcount.hwpx Center over(중첩표2)  BUG 105.6(상단) / FIX 197.7(중앙)
- cell_vbottom_nested_overcount.hwpx      Bottom  over          BUG 109.8(상단) / FIX 265.8(하단)
- cell_vcenter_nested_undercount.hwpx     Center  under 가드     BUG 212.2 / FIX 212.2 (양쪽 정상 중앙; #44 회귀 방지)

셀 텍스트는 HOSTLINE/CENTERME/INNER 합성만.
