# 요청 → 명령 매핑

사용자 말을 기존 CLI 이름 하나로 접는다. 새 이름을 만들지 않는다.

| 사용자 요청 | 명령 | 페이지 | 레퍼런스 |
|---|---|---|---|
| svg로 빼줘 | `export-svg` | 0-based -p | 02_export_svg.md |
| 3쪽을 svg | `export-svg` | 0-based -p | 02_export_svg.md |
| debug overlay | `export-svg` | 0-based -p | 02_export_svg.md |
| 겹침 보이게 | `export-svg` | 0-based -p | 02_export_svg.md |
| png로 비전 모델에 | `export-png` | 0-based -p | 03_export_png.md |
| 인쇄용 pdf | `export-pdf` | 0-based -p | 04_export_pdf.md |
| 본문만 텍스트 | `export-text` | 0-based -p | 05_export_text.md |
| 마크다운으로 | `export-markdown` | 0-based -p | 06_export_markdown.md |
| 이 페이지 배치 | `dump-pages` | 0-based -p | 07_dump_pages.md |
| 문단 속성 | `dump` | n/a | 08_dump.md |
| raw record | `dump-records` | n/a | 09_dump_records.md |
| 번호가 이상해 | `diag` | n/a | 10_diag.md |
| 몇 쪽이야 | `info` | n/a | 11_info.md |
| bbox 좌표 | `export-render-tree` | 0-based -p | 12_export_render_tree.md |
| hwpx랑 hwp 비교 | `ir-diff` | n/a | 13_ir_diff.md |
| 썸네일만 | `thumbnail` | n/a | 14_thumbnail.md |
| 배포용 풀고 편집 | `convert` | n/a | 15_convert.md |
| 한컴 저장이랑 달라 | `hwp5-inventory-diff` | n/a | 16_hwp5_family.md |
| 표 저장 계약 | `hwp5-table-probe` | n/a | 16_hwp5_family.md |
| 특정 글자 주변 record | `hwp5-anchor-trace` | n/a | 16_hwp5_family.md |
| CHAR_SHAPE 차이 | `hwp5-char-shape-audit` | n/a | 16_hwp5_family.md |
| 자기 라운드트립 | `hwp5-roundtrip` | n/a | 16_hwp5_family.md |
| 레이아웃 버그 | `export-svg` | 0-based -p | 02_export_svg.md |
| 간격이 이상해 | `dump-pages` | 0-based -p | 07_dump_pages.md |
| 셀이 잘려 | `export-svg` | 0-based -p | 02_export_svg.md |

## 레이아웃 요청은 명령 하나가 아니다

"간격/겹침/잘림 디버깅" 은 6단 사다리다. 첫 명령은 항상 `export-svg --debug-overlay`.

## 매핑 규칙

1. 요청에 쪽번호가 있으면 한컴 표기로 가정하고 1을 뺀다. 사용자가 0 기준이라고 밝히면 그대로.
2. "비교" 가 두 파일이면 ir-diff. 한 파일+한컴 저장본이면 hwp5-inventory-diff.
3. "고쳐서 저장" 은 이 스킬이 아니다. rhwp-safe-edit.
