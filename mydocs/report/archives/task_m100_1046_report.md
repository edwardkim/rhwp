# 최종 결과보고서 — #1046: 본문 하단 overflow 정합 (측정 통일 B)

- 타스크: #1046 (M100), 브랜치 `local/task1046`
- 대상: 비공개 185p "재정통합 제안요청서" hwpx 본문 하단 LAYOUT_OVERFLOW
- 검증 권위: `pdf/2. 인공지능(AI) … 제안요청서-2022.pdf` (한컴 2022)
- 작성일: 2026-05-21

## 1. 개요 / 전환 경과

당초 사후 reflow(A, overflow 항목 다음 페이지 이월)는 측정 드리프트로 overflow 가
이동·증식해 **폐기**(Stage 3 findings, 커밋 15d6d2bb). 작업지시자 결정으로 **측정 통일(B)**
로 전환(수행계획서 `task_m100_1046_v2.md`).

## 2. 결과 요약

| 지표 | baseline | 최종 |
|------|----------|------|
| LAYOUT_OVERFLOW 총건 | 18 | **5** |
| in-scope (page-larger 2 제외) | 14 | **1** |
| `cargo test --release` | — | **1516 passed / 0 failed** |
| 대상 페이지 수 | 185 | 185 (불변) |
| aift.hwp 페이지 수 | 74 | 74 (불변, 회귀 0) |

해소 13건: 242·256(Stage 2), 290(A), 266·308·354·357(B), 218·600(C), 361·429·268·406(D).

## 3. 근본 원인 — 두 축

### 3-1. 배치 판정 overhead 누락 (Stage 2 + Class A)
분할 진입부 가드의 `remaining_on_page` 가 첫(비연속) fragment 의 렌더러 y_start 점프
(host_spacing.before + TopAndBottom·vert=Para 표의 vertical_offset)를 차감하지 않아, 안
들어가는 표를 강제 배치 → 초과. 가드에 동일 overhead 차감 + 다행 표 비분할 첫행 조건부
이월(`multirow_clean_defer`) 추가. genuine page-larger·1×1 셀(#874)은 제외해 무회귀.
- 해소: pi=242(19.2px), 256, 290(8.7px). 한컴 PDF 정합(SIR-002/COR-003 표 통째 배치).

### 3-2. overflow 검출의 trailing 간격 오검출 (Class B/C/D)
표/문단 콘텐츠는 본문 안에 들어가는데, **표 뒤/문단 끝의 trailing 간격**(줄간격/
spacing_after/outer_margin_bottom)이 더해진 y_offset 으로 초과를 판정해 false-positive.
페이지네이터는 이미 마지막 줄 trailing_ls 허용 배치(#359/#404)인데 검출만 어긋남.
- 수정: `last_item_content_bottom`(Cell) — 표/문단 렌더가 trailing 가산 직전 콘텐츠 하단
  기록(통째표=table_y_end, 분할표=layout_partial_table 반환직후, 문단=매 줄 y+line_height),
  검출이 표/문단 항목에서 이 값으로 비교. **렌더링 출력 불변 — 검출만 정정**(골든 무영향).
- 해소: 266/308/354/357(B), 218/600(C), 361/429/268/406(D).

## 4. 잔여 (범위 외 / 별도 이슈)

- **pi=781 (4.6px, in-scope)** — 별도 이슈 보류(작업지시자 결정). 진단 완료
  (`task_m100_1046_v2_stage3_781diag.md`): trailing/페이지네이터 아님. 렌더러가 특정 헤더
  문단(pi=760, char 15pt·line 110% → 올바른 22.0px 인데 34.8px 과대 렌더) 줄높이를 HWP
  vpos 피치보다 과대 계산해 헤더에서 21.7px 누적. 줄높이는 전 문서 렌더 핵심 경로라
  회귀 위험 큼 → 별도 이슈에서 정밀 격리 후 수정 권고.
- **page-larger 2건 (pi=323 단독 표, pi=567 nested)** — 단일 항목이 본문보다 큼, 범위 외.

## 5. 검증
- `cargo test --release`: 1516 passed / 0 failed (골든 SVG 회귀 0).
- 한컴 2022 PDF 대조: SIR-002(idx48 부근)·COR-003(idx48)·TER-003 요구사항 표 모두 한
  페이지 통째 배치 — 수정 동작과 정합. 분할 미발생이 정답.
- 대상 185p / aift 74p 불변 — 페이지 수 회귀 0.

## 6. 상주 인프라 (env 게이트, 동작 불변)
- `RHWP_TABLE_DRIFT`: TABLE_DRIFT / TABLE_CUT_DRIFT / LAYOUT_Y / TABLE_SPLIT_AVAIL /
  TABLE_SPLIT_RESULT / WHOLE_TABLE_Y — 표 측정·배치·렌더 y 대조 도구.

## 7. 커밋 (측정 통일 B 라인)
799fb7fc(B Stage1 진단) · bb72c1f9(Stage2 진단) · 9d755101(Stage2 수정) ·
7fa24f4d(Stage3 분류) · 45d7e262(Class A) · 0e722702(Class B+C) · c3a1de5d(Class D) ·
ca94f363(781 진단).
