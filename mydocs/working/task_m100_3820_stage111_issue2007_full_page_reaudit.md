---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 111 — issue2007 최신 17쪽 전수 재감사

## 목적과 기준

Stage 63 이후 renderer 변경이 누적됐으므로 당시의 p7–p17 합격 기록을 그대로
재사용하지 않는다. 최신 committed source와 같은 release-test 바이너리로
`issue2007_nested_cell_pagination_42065.hwp`의 모든 물리 쪽을 독립 한컴 PDF와 다시
비교해 페이지 경계, 중첩 표 continuation, 셀 경계 침범, 표 선 clip, 상·하단 글자
절단이 회귀했는지 판정한다.

- 시작 commit: `66f129a7f`
- renderer 기준 commit: `7093985f0`
- `7093985f0..66f129a7f`의 `src/` 차이: 없음
- 입력 HWP:
  `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 입력 SHA-256:
  `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626`
- 한컴 기준 PDF:
  `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 기준 SHA-256:
  `9b0390f856bb9ad43337679babf6677209b7c7ab678b6616fcc6d6d5551ff1c4`
- PDF metadata: Creator `HANCOM OFFICE HANGUL 2010 8.0.0.466`, Producer
  `Hancom PDF 1.3.0.404`
- 검증 바이너리: `target/pr-review/release-test/rhwp`
- 바이너리 SHA-256:
  `c88b9d91254920dad1ff28805219b4540c76770110e33bcc8422eec7202e72dd`

## 실행

새 output을 사용해 과거 SVG cache를 섞지 않았다.

```text
RHWP_BIN=target/pr-review/release-test/rhwp \
venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 16 \
  --source samples/basic/issue2007_nested_cell_pagination_42065.hwp \
  --reference-pdf pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf \
  --label task3820-stage111-issue2007-full-current \
  --reference-grade '한컴오피스 2020 기준 PDF' \
  --export-all-svg --layout-ledger \
  --out-dir output/task-3820-stage111-issue2007-full-current
```

## 전수 결과

- 기준 PDF / rhwp SVG / render tree: `17/17/17`쪽
- 요청 / 완료 / 누락: `17/17/0`
- 인접 text owner-shift: 0건
- 인접 text owner-sequence: 0건
- 통합 page-boundary 후보: 0건
- visible text excess: 0건
- 같은 셀의 paint text overlap: 0건
- 실제 외곽 세로선 clip 후보: 0건
- 실제 외곽 가로선 clip 후보: 0건

17개 페이지의 PDF와 rhwp 비교 이미지를 각각 원본 크기로 직접 판독했다.

- p1–p6: p2 셀 문단 중첩과 p2–p4 우측 세로선 소실이 재발하지 않았다. 모든 문자는
  자기 셀 안에 있고 p4의 중첩 표 우측 외곽선도 보인다.
- p7–p9: p7의 조기 표 선, p8 하단 추가 선, p9 제목/표 경계 회귀가 없다. p8의
  마지막 표 행과 p9의 표는 기준 PDF와 같은 페이지를 소유한다.
- p10–p15: 중첩 표 continuation의 본문 순서와 페이지별 section 소유권이 PDF와
  같다. p12·p15 제목 앞 간격, p14의 두 block 사이 간격과 p14·p15 하단 경계도
  유지된다.
- p16–p17: 상단 문단의 첫 glyph가 잘리지 않고 기준 PDF와 같은 쪽에서 시작한다.

픽셀 diff 상위는 p11 `24.93%`, p10 `24.25%`, p14 `23.00%`, p15 `22.67%`다.
비교 이미지에서 이 차이는 휴먼명조 EBDT local face를 Chrome에서 직접 선택하지
않고 정상 outline 명조로 raster하는 fidelity 하네스의 획·antialiasing 차이가 주로
차지한다. 줄 소유권, 줄 수, 표 fragment, block 시작·끝은 위 구조 원장과 직접
판독에서 PDF와 일치한다. 따라서 픽셀 diff만으로 renderer 회귀로 승격하지 않는다.

## 자동 후보의 직접 폐기

`svg-text-band-clip-candidates.tsv`는 p14 상단에 baseline `118.1px`, clip top
`120.1px`, visible-height ratio `0.05`인 source text 35자를 올렸다. 이는 이전
fragment의 glyph 근사 band 끝 `0.7px`가 clip 계산과 만난 신호다. rhwp p14의 실제
raster 상단을 확대하면 첫 가시 줄은 PDF와 같은 `2. 증명서, 변명서...`이고, 그 위에
문자 잉크나 잘린 획이 나타나지 않는다. 근사 band가 glyph outline의 빈 하단 영역까지
포함해 발생한 candidate-only 오탐이므로 renderer를 추측 수정하지 않는다.

`table-cell-text-boundary-candidates.tsv`의 p8 1건과 p9 3건도 모두
`natural_visible_width_risk`다. 실제 raster에서는 끝 glyph가 세로선을 침범하지 않고
PDF와 같은 줄에 놓이므로 오탐으로 폐기한다.

## 결론과 다음 범위

최신 renderer에서 issue2007 17쪽의 기존 구조·페이지 경계 결함은 재현되지 않았다.
이 stage는 별도 코드 변경 없이 현재 상태를 전수 증적으로 고정한다. 전체 #3820 완료를
의미하지 않으며, 다음 stage는 기존 문서에 명시적으로 잔여로 남은 정책연구 p23–p24
그림·caption flow부터 직접 재감사한다.

## 증적

- [17쪽 contact sheet](../pr/assets/task_m100_3820_stage111_issue2007_full_page_reaudit/review_contact_sheet.png)
- [픽셀 순위](../pr/assets/task_m100_3820_stage111_issue2007_full_page_reaudit/report.tsv)
- [문자 원장](../pr/assets/task_m100_3820_stage111_issue2007_full_page_reaudit/text-report.tsv)
- [페이지 수 원장](../pr/assets/task_m100_3820_stage111_issue2007_full_page_reaudit/page-count-ledger.tsv)
- [실행 완료 원장](../pr/assets/task_m100_3820_stage111_issue2007_full_page_reaudit/run-state.tsv)
- 페이지별 원본 크기 비교:
  `mydocs/pr/assets/task_m100_3820_stage111_issue2007_full_page_reaudit/compare/`
