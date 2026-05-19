# 최종 결과보고서 — Task #987

## 이슈

**#987** HWP3 쪽 테두리: `border_type=4` 이중선 매핑 + 위치 본문 기준 정합
+ 쪽 번호 위치 정합 (sample16)

- 브랜치: `local/task987` (← `local/devel`)
- 마일스톤: M100 (v1.0.0)
- 대상: `samples/hwp3-sample16.hwp` (HWP 3.0)
- 정답지: `pdf/hwp3-sample16-hwp5-2022.pdf` (한글 2022)

## 배경

HWP3 sample16 쪽 테두리/쪽 번호 시각 판정에서 3개 결함 발견:

1. 선 종류: 한컴 = 이중 실선, rhwp = 실선 (`border_type=4` 미매핑)
2. 위치 기준: 한컴 = 본문(body) 기준, rhwp = 종이(paper) 기준
3. 쪽 번호 위치: 한컴 = 쪽 테두리 하변 근처, rhwp = footer 중앙(어긋남)

## 변경 요약 (Stage 1~5)

### Stage 1 — 이중선 매핑 + off-by-one 수정

- `src/parser/hwp3/mod.rs`: `border_type=4 → BorderLineType::Double`
  (스펙 `한글문서파일구조3.0.md:850` 근거)
- **bfid off-by-one 수정**: `(len-1)` 0-based → `len` 1-based.
  렌더러 인덱싱 정합 — Double 매핑이 화면에 미반영되던 차단 버그.
  (#952 의 "sample16 paper 정합" 판정이 이 버그로 인한 착시였음)

### Stage 2-3 — 위치 body 기준 정합

- `src/renderer/layout.rs:944`: `paper_based` 전역 `true`
  → `(pbf.attr & 0x01) != 0` (attr 존중 복원)
- `src/parser/hwp3/mod.rs`: HWP3 page_border `attr: 1 → 0` (body)
- CLAUDE.md "HWP3 전용 로직은 파서 안에서만" 준수: layout.rs 는
  포맷 중립 attr 해석, HWP3 고유 판단은 파서가 attr 로 표현

### Stage 5 — 쪽 번호 위치 정합

- `page_number_baseline_y` 헬퍼: **body 기준 테두리일 때만** Some,
  꼬리말 영역(footer_area) 세로 중앙 baseline 반환
- `build_page_number` 에 `page_border_fill` 전달, footer 위치 +
  body 기준 테두리 + `!hide_border` 시 적용
- 회귀 2건 해소: hide_border 조건 + paper 기준 제외 (aift Task #634)

> Stage 4 (회귀 검증) 는 각 Stage 검증에 통합 수행.

## 정답지 대조 (한글 2022 PDF, pdftotext -bbox)

| 항목 | y (px, rhwp 좌표) |
|------|-------------------|
| 정답지 쪽 번호 "- 1 -" | 1072.7 ~ 1087.3 |
| rhwp 쪽 테두리 하변 | 1086.1 ~ 1088.2 |
| rhwp 쪽 번호 출력 | 1089.8 |

→ 한컴 정답지 자체가 쪽 번호를 쪽 테두리 하변에 걸치게 출력.
  rhwp 출력(1089.8)과 정답지(1087.3) **0.7mm 차이 — 정답지 일치**.

## 시각 판정 (작업지시자)

| 항목 | 결과 |
|------|------|
| 쪽 테두리 선 종류 (이중선) | ✅ 통과 |
| 쪽 테두리 위/아래 위치 (body 기준) | ✅ 통과 |
| 쪽 번호 위치 | ✅ 통과 (정답지 PDF 정밀 측정 대조, 0.7mm 일치) |

## 검증

| 항목 | 결과 |
|------|------|
| `cargo test` | ✅ 1476 passed, 0 failed |
| `cargo test --lib test_634` (aift 쪽번호 회귀) | ✅ 8 passed |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| WASM 빌드 (Docker) | ✅ Stage 1~3 반영 확인 (16:02 갱신) |
| HWP5 시험지 회귀 | ✅ 없음 (paper 기준 유지) |
| aift.hwp 쪽 번호 | ✅ Task #634 정합 유지 |

## 커밋

| 커밋 | 내용 |
|------|------|
| e356627d | Stage 1: border_type=4→Double + bfid off-by-one |
| 63ead19c | Stage 2-3: 쪽 테두리 위치 body 기준 |
| 343be659 | Stage 5: 쪽 번호 테두리 하변 아래 (초안) |
| 6ba0ea10 | Stage 5 정정: 꼬리말 영역 세로 중앙 |

## 후속 과제 (범위 외, 기록)

`border_type=2` → 현재 `BorderLineType::Dash`, 스펙
(`한글문서파일구조3.0.md:850`)은 "굵은 실선". 시각 판정 게이트
필요 — 별도 이슈 검토 권고.

## 결론

Stage 1~5 전 항목 시각 판정 + 정답지 PDF 정밀 측정 대조 통과.
HWP 3.0 한정 변경으로 HWP5/HWPX 회귀 없음 확인.
이슈 #987 완료 — 작업지시자 승인 후 close.
