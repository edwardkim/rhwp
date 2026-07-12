# Task M100 #2211 — 3단계 완료 보고: 중첩/TAC 표 행 성장 pad 제외 + 저장 셀 흐름 신뢰

- 이슈: #2211 / 브랜치: `local/task2211` / 작성일: 2026-07-12

## 구현 (2건 + 구조 가드, [table_layout.rs](../../src/renderer/layout/table_layout.rs))

**정정 1 — 행 성장 판정 pad 가산 제외** (2단계 승인안): `resolve_row_heights`
1-b/2-b에서 저장 LINE_SEG 보유 셀(합성 seg 제외, `para_has_no_stored_line_segs`
술어 공유)의 **줄 흐름**은 `required = line_based`(pad 미가산). 개체 기반
지오메트리(중첩 표·TopAndBottom flow·Square bottom)는 pad 가산 유지 —
콘텐츠 높이를 (줄, 개체)로 분리하는 `calc_cell_paragraphs_content_parts` 신설
(#1486 p19 Square 그림 캘리브레이션 보존).

**정정 2 — 저장 셀 흐름 압축 신뢰** (작업지시자 추가 리포트: 악보 가사 절단):
저장 LINE_SEG extent(seg vpos+lh 최댓값)가 자체 스택 합보다 작은 셀(악보 셀 —
빈 앵커 줄이 그림 높이에 흡수, 가사 vpos=2553=그림 높이)은 ①정렬 기준 콘텐츠
높이를 저장 extent로 ②문단 배치를 저장 vpos 스냅으로 강제. 비-flow 개체
(Square/중첩 표)가 저장 extent를 넘는 셀은 제외 (`non_flow_object_extent` 가드).

**구조 가드 (`relaxed_pad`)**: 두 정정 모두 **중첩 표(셀 내부) 또는 TAC 표**로
한정 — `depth > 0 || treat_as_char`. 상위 분할/앵커 생태계(RowBreak 분할·하단
앵커·걸침 rowspan)는 pad 포함 회계로 캘리브레이션되어 있어(#1748/#1858 실증)
기존 동작 유지. 케이스별 명시 가드 정책 준수. 하드코딩 없음 — 판정 근거는
저장 LINE_SEG/선언 h/pad/wrap 스펙 필드뿐.

### 시행착오 기록 (경로 캘리브레이션 충돌)

전면 적용 시도에서 상호 배타 실패를 실측: 레이아웃 단독 적용 → #1858(하단앵커
flush, 측정·렌더 두-경로 불일치 8.5px) 실패 / 측정 경로 미러 추가 → #1748(걸침
rowspan 연속쪽, 컷 예산 한 줄 오버 13.6px) 실패. 분할/앵커 기계 전반의 재캘리브
없이는 전면 규칙이 불가함을 확인하고 구조 가드로 수렴. **상위 float/분할 표의
동일 팽창(rowbreak 노트박스 등)은 후속 이슈 범위**.

## 정량 효과 (주보 p1, 한컴 대조 — 측정 기준 오프셋 ≈ +11)

| 항목 | 수정 전 | 수정 후 |
|------|--------|--------|
| 좌측 단 마지막 문단("모세야") baseline | 755.2 (본문 하한 748 초과 절단) | **730.7 (완전 포함)** |
| 스페이서 행(모세가 위 빈 행) | 7.76px | **4.00px = 선언 300HU = 한컴 편집기 1.06mm** |
| 악보 가사 baseline vs 셀 클립 552.5 | 555.1 (하반 절단) | **544.9 (한컴 예측 545 일치, 여유 7.6px)** |
| 좌측 단 델타 중앙값 | — | +11.0 (실질 ~0) |

우측 단 잔여 +29px = 별개 기제(BehindText+tac 표 호스트 줄에 빈 문단 텍스트 줄
1950HU 가산 — 저장 lh 24700 = 표 22996 + outer_margin 1704 실증) — 후속 이슈.

## 표적 테스트 + 게이트

- `tests/issue_2211_nested_table_row_growth.rs` 신설 — 수정 전 **FAILED
  (y=755.2)** 실증 / 수정 후 ok.
- fmt 통과 / clippy 0 / `--tests --no-fail-fast` **3,045 / 실패 0**
- golden svg_snapshot 8/8 — exam-kor-page5 1건 의도적 재생성 (대각선 박스 하단
  0.027px 반올림 경계, 텍스트 1,835자 전부 무이동)
- 영향권 스위트 전부 통과: #1486(6), #1748(3), #1858(6), #1994, #2189, #2207
- OVR 5샘플: 4샘플 0건 + rowbreak obj1 **h−3.8px 1건 = 의도된 정합 개선**
  (tac=true 노트박스, 한컴 154px vs 우리 162.8→159.0px) → **baseline 현행화
  요청** (전례: #1936 시각 판정 후 재생성)

## 시각 판정 자산 (4단계)

- `output/poc/issue2211/compare_left_bottom.png` — 좌측 단 하단 3-way
- `output/poc/issue2211/compare_lyrics_cell.png` — 악보 가사 셀 3-way
