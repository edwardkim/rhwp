---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 27 — issue2007 p1–p7 native PDF 직접 대조

## 범위

Stage 21은 p2의 table-cell text overlap과 p4의 우측 border를 focused 확인했다.
Stage 24–26은 p8–p17의 nested-table continuation과 physical page boundary를 확인했다.
17쪽 전체의 native PDF fidelity에 대해 부분 판정만 묶어 전체 정상이라고 말하지 않기 위해,
남은 p1–p7을 한컴 기준 PDF와 각각 직접 대조한다.

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 매핑: PDF p1–p7 ↔ `rhwp export-svg --page 0..6`

## 판정 계약

각 SVG/PDF를 같은 192dpi physical raster로 비교한다. 페이지 수가 같다는 사실은 보조
정보일 뿐이고, table frame, 중첩 cell text overlap, 그림/본문의 source owner, page
top/bottom을 개별로 확인한다. 실제 구조 차이가 발견되면 이 stage에는 원인 기록까지만
남기고 수정은 다음 stage로 분리한다.

## 직접 대조 결과

최신 `release-test/rhwp export-svg --profile print`가 생성한 17개 SVG 중 p1–p7을
선택해, Hancom 2020 기준 PDF의 같은 physical page와 192dpi/1588×2246px에서 직접
대조했다. 각 pair의 왼쪽은 rhwp, 오른쪽은 기준 PDF다.

- p1: 요약 표의 row/column frame과 셀 내용의 시작·끝이 같은 위치다.
- p2: 조문 대비표의 두 nested cell에서 문단이 겹치지 않으며, Stage 21의 overlap
  회귀가 유지된다.
- p3: 표의 계속된 row, 줄 경계 및 하단 source owner가 같은 physical page에 있다.
- p4: 규제영향분석서 표의 우측 outer border를 포함한 frame이 완전하게 paint되며,
  Stage 21의 right-border 회귀가 유지된다.
- p5–p7: 일반 문단, 통계 표 및 heading의 physical top/bottom과 page owner가 일치한다.

글꼴 glyph/획과 일부 문자 폭은 native SVG와 Hancom PDF raster가 다르지만, page
분할, table geometry, text overlap, source owner 차이는 보이지 않았다. Stage 24–26의
p8–p17 직접 대조와 합쳐 이 fixture의 **native** renderer physical page 17/17을
구조적으로 확인했다. 이 결론은 WASM/브라우저 최종 paint를 대체하지 않는다.

## 증적

PNG는 `git check-attr filter`에서 LFS 대상이 아님을 먼저 확인했다.

- [p1–p7 contact](../pr/assets/task_m100_3820_stage27_issue2007_p1_p7_native_fidelity_scan/review_p001_p007_contact.png)
- 페이지별 대조: [p1](../pr/assets/task_m100_3820_stage27_issue2007_p1_p7_native_fidelity_scan/review_p001_pair.png), [p2](../pr/assets/task_m100_3820_stage27_issue2007_p1_p7_native_fidelity_scan/review_p002_pair.png), [p3](../pr/assets/task_m100_3820_stage27_issue2007_p1_p7_native_fidelity_scan/review_p003_pair.png), [p4](../pr/assets/task_m100_3820_stage27_issue2007_p1_p7_native_fidelity_scan/review_p004_pair.png), [p5](../pr/assets/task_m100_3820_stage27_issue2007_p1_p7_native_fidelity_scan/review_p005_pair.png), [p6](../pr/assets/task_m100_3820_stage27_issue2007_p1_p7_native_fidelity_scan/review_p006_pair.png), [p7](../pr/assets/task_m100_3820_stage27_issue2007_p1_p7_native_fidelity_scan/review_p007_pair.png)

## 검증 요약

- 최신 native CLI의 `export-svg`는 17개 SVG를 생성했다.
- Stage 24 source 보정의 focused integration
  `cargo test --profile release-test --test issue_2007_nested_cell_pagination`은 9 passed다.
- 이 stage에서는 새 renderer 결함을 확인하지 못했으므로 코드를 추가로 변경하지 않았다.
