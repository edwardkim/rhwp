# Task M100-1156 — Stage 2 정정 보고서

- 이슈: [#1156](https://github.com/edwardkim/rhwp/issues/1156)
- 일시: 2026-05-29
- 단계: Stage 2 (차트 단 이동) — **정량 + 시각 정합, 작업지시자 최종 판정 대기**

## 1. 정정 영역

`src/renderer/typeset.rs` — `typeset_table_paragraph` 의 Shape/Picture/Equation 분기 (+43줄)

### 근본 원인 (Stage 1 정정)

- 실제 pagination 엔진 = **TypesetEngine** (typeset.rs), engine.rs 는 fallback
- `typeset_table_paragraph` 의 Shape 분기는 **TAC 객체만** 높이 가산/단 이동 처리
- **비-TAC 자리차지(TopAndBottom) 객체(차트 OLE)는 높이 0 으로 push 만** → 단 이동/back-fill 누락

### 변경 내용

비-TAC + TopAndBottom + vert=Para 인 Picture/Shape(차트) 에 대해 `non_tac_pushdown_h` (common 높이 + margin_bottom) 계산 후:
- 현재 단 잔여 영역 부족 + 단 상단 아니면 `advance_column_or_new_page` (다음 단 이동)
- `current_height += extra` (점유 높이 반영)

기존 `pushdown_h` (typeset.rs:1621, !has_table 경로) 와 동일 시멘틱을 has_table 경로에도 적용.

## 2. 정량 검증 (dump-pages)

| | 정정 전 | 정정 후 |
|--|---------|---------|
| 단 0 | items=14, used=785px | items=11, used=729px |
| 단 1 | items=6, used=345px, **diff=-396px** | items=8, used=726px, **diff=-15.5px** |
| 차트(pi=7 ci=1) | 단0 vpos=48603 (표와 겹침) | **단1 상단** |

→ 차트가 단1 상단으로 이동, 단1 diff -396px → -15.5px (거의 정합). 차트가 비운 단0 공간에 pi8/pi9 텍스트 back-fill 자동 발생.

## 3. 시각 검증 (PNG export, 폰트 폴더 사용)

`rhwp export-png --font-path /home/edward/mygithub/ttfs[/hwp,/windows]`:
- 정정본 `output/poc/issue_1156/round1.png`: **차트(막대그래프)가 단1 상단** + 텍스트
- 정답지 `pdf-large/hwpx/143E433F503322BD33.pdf` (page-1): 동일 — 차트 단1 상단

→ **정답지와 차트 배치 일치.**

## 4. 회귀 검증

- `cargo test --release --test svg_snapshot` → **8 passed, 0 failed** (다단 포함 골든 불변)
- `cargo fmt --all -- --check` → 정합
- `cargo clippy --lib --release -- -D warnings` → warnings 0

## 5. 작업지시자 시각 판정 요청

`output/poc/issue_1156/round1.png` (PNG, 폰트 적용) vs 정답지 PDF 비교:
1. 차트가 단1 상단 정상 배치 인지
2. 단0 빈 공간 텍스트 back-fill 정합 인지
3. 단1 텍스트 흐름 정합 인지

## 6. 다음 단계

- Stage 3: back-fill 미세 정합 (단1 diff -15.5px 잔차 분석, 필요 시)
- Stage 4: 회귀 가드 + 최종 보고서 + close

단1 diff 가 -15.5px 로 이미 거의 정합이므로, Stage 3 back-fill 은 잔차 확인 후 필요 시만 추가 정정.
