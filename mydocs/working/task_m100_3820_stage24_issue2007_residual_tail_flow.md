---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 24 — issue2007 p10 잔여 표 뒤 흐름 높이

## 범위와 기준

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준 PDF: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 대상 경계: PDF/rhwp 물리 p9 → p10 (`export-svg --page 8`, `--page 9`)

Stage 23의 p13 owner 조사와 분리한다. 이번 단계는 p10 첫 제목
`< 조사기능 관련 타기관 입법례 >`의 물리 수직 위치만 다룬다.

## 재현과 현재 관찰

`4f558d4cc`는 p10 상단에 나타나던 8×4 표의 4.95px 하단 테두리를 제거했다.
하지만 192dpi PDF/SVG 직접 raster 대조에서 p10의 제목·이후 표 전체는 기준 PDF보다
여전히 아래에 있다.

현재 render tree에서 같은 1×1 continuation cell의 좌표는 다음과 같다.

| 항목 | rhwp y (px) |
| --- | ---: |
| p10 inner continuation cell top | 117.147 |
| 이전 8×4 표의 보이는 꼬리 bottom | 122.093 |
| 그 뒤 `vpos=0` 빈 문단 top | 135.960 |
| p10 제목 top | 144.493 |
| p9의 동등한 새-block 제목 top | 132.493 |

즉, border paint만 숨겨서는 충분하지 않다. p10의 새 block은 이전 표가 끝난 뒤의
빈 문단·여백을 다시 예약해 p9 기준보다 12px 늦게 시작한다. 전체 높이 상수나 글꼴
보정으로 덮지 말고, **잔여 표가 현재 viewport에 콘텐츠를 갖지 않을 때에만** 그 뒤
source block의 flow origin을 정상화해야 한다.

## 다음 검증 계약

1. 8×4 표의 마지막 row fragment와 `vpos=0` spacer가 `CellUnit`에 만드는 예약을
   source 단위로 대응한다.
2. p10의 residual-tail 경로만 정상화하고 p9 제목, p11/p15 continuation frame,
   p13 residual-frame 회귀를 함께 재검증한다.
3. 변경 후 p9→p10 192dpi raster 경계와 p10 render-tree title top을 PDF와 직접
   대조한다. 페이지 수나 자동 픽셀 점수만으로 완료를 선언하지 않는다.

## 원인과 보정

p10의 이전 8×4 표 조각은 현재 쪽에 실제 표 콘텐츠 없이 4.95px의 terminal tail만
남는다. 그러나 그 바로 뒤에 저장된 빈 `vpos=0` line과 이어지는 line advance는 이미
p9에서 소비됐음에도 p10의 clipped cell에 다시 남아 있었다. 따라서 residual border만
숨기면 p10의 실제 제목과 이후 source block이 두 line advance(17.067px)만큼 낮게
배치됐다.

`repair_clipped_nested_table_fragment_frame`에서 다음의 매우 좁은 경우에만 이를
정상화했다.

1. direct nested table의 current-page fragment가 6px 미만이고,
2. 그 뒤의 direct sibling이 빈 `vpos=0` TextLine이며,
3. 다음 TextLine까지의 advance가 0.5px 초과 16px 이하인 경우.

그 경우 terminal table과 소비된 빈 spacer를 paint 대상에서 제외하고, **spacer 뒤**의
source siblings만 두 advance만큼 위로 이동한다. spacer 자체를 이동하지 않는 것이
중요하다. 그것을 옮기면 clipped ancestor 안에서 이전 logical line과 bbox가 겹쳐
정상 continuation의 text-overlap 계약을 깨뜨린다. p9처럼 residual table이 없는 실제
새 block에는 이 경로가 적용되지 않는다.

focused regression은 p10 제목의 top을 126.5–128.5px 범위로 고정하고, p10의 false
full-width border가 없으며 p11/p15 continuation frame과 p10–p16 nested-cell
text-overlap 계약이 모두 유지되는지 함께 확인한다.

## PDF 직접 재검증

최신 native `release-test` binary로 `export-svg --profile print`를 다시 실행한 뒤,
SVG는 1588×2246px, 기준 PDF는 `pdftoppm -r 192`로 같은 물리 해상도에 raster했다.
아래 비교 PNG의 왼쪽은 rhwp, 오른쪽은 Hancom 2020 기준 PDF다.

- [p9 하단 → p10 상단 경계](../pr/assets/task_m100_3820_stage24_issue2007_residual_tail_flow/compare_p009_p010_boundary_after.png): p10 상단의 false table border가 없고, `< 조사기능 관련 타기관 입법례 >`의 시작 높이와 다음 dotted table의 시작이 PDF와 같은 물리 경계에 있다.
- [p10 전체 직접 대조](../pr/assets/task_m100_3820_stage24_issue2007_residual_tail_flow/compare_p010_after.png): 흐름과 표 frame은 기준과 맞고, 남는 글자 모양 차이는 PDF/SVG 글꼴 raster 차이다.
- [p11 직접 대조](../pr/assets/task_m100_3820_stage24_issue2007_residual_tail_flow/compare_p011_after.png): continuation frame의 첫 줄과 뒤 `국세청` block이 새 보정으로 이동하거나 누락되지 않았다.
- [p13 직접 대조](../pr/assets/task_m100_3820_stage24_issue2007_residual_tail_flow/compare_p013_after.png): `국가인권위원회`의 p13 시작과 다음 `감사원` block의 물리 순서가 유지된다.

이 판정은 p9→p10 terminal-tail seam에 한정한다. 이 네 쪽이 17쪽 문서 전체의
글꼴·paint fidelity까지 종료됐다는 주장은 아니다.

## 검증

```bash
CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_2007_nested_cell_pagination
# 9 passed

CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 \
  cargo build --profile release-test --bin rhwp
# 최신 native binary로 p9, p10, p11, p13 export-svg 및 192dpi PDF 대조
```

`git diff --check`도 통과했다. WASM build는 사용자가 이미 별도로 수행한 수동
검증 범위이므로 이 단계에서 다시 실행하지 않았다.

또한 `python3 scripts/check_markdown_links.py
mydocs/working/task_m100_3820_stage24_issue2007_residual_tail_flow.md`로 이 기록의
네 asset 상대 링크가 모두 유효함을 확인했다.
