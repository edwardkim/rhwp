# Stage 1 완료보고서 — Task #987

## 단계 목표

HWP3 `border_type=4` → `BorderLineType::Double` 매핑하여 쪽 테두리 이중선 렌더.

## 변경 내용

### 1. `border_type` → `BorderLineType` 매핑 (`src/parser/hwp3/mod.rs:2795-2806`)

스펙 근거 확보: `mydocs/tech/한글문서파일구조3.0.md:850` —
HWP3 선 종류 체계 `0=없음, 1=실선, 2=굵은 실선, 3=점선, 4=2중 실선`.

- `4 => BorderLineType::Double` 추가
- `_ =>` fallback 주석 `5 이상`으로 정정
- 스펙 대조로 `2 => Dash` 가 오매핑(스펙=굵은 실선)임을 발견 →
  범위 외라 미수정, **후속 과제로 기록** (아래 §후속)

### 2. off-by-one 버그 수정 (`src/parser/hwp3/mod.rs:2815-2820`)

**Stage 1 진행 중 발견한 차단 버그.**

- 기존: `let bfid = (doc_border_fills.len() - 1) as u16; // 0-based`
- 수정: `let bfid = doc_border_fills.len() as u16; // 1-based`
- 근거: 렌더러 `layout.rs:934` 가 `border_fill_id - 1` 로 인덱싱(1-based 가정).
  같은 파일 `mod.rs:310/1043` 도 `len()` 1-based 규칙 사용.
  page_border 만 0-based(`len-1`) 라 렌더러가 -1 더 빼서
  인접 빈 border(`fill_type=None`) 를 읽음 → Double 매핑해도 화면 미반영.

## 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 성공 |
| 쪽 테두리 line 수 (sample16 p0) | ✅ 4 → **8** (변당 2줄 × 4변 = Double) |
| 상단 변 평행선 | ✅ y=17.88 / y=19.98 (간격 ≈2.1px), width=0.9 |
| `cargo test` | ✅ **1476 passed, 0 failed** (회귀 없음) |

산출물: `output/poc/task987_stage1/hwp3-sample16_001.svg`

## 시각 판정 요청

sample16 페이지 0 쪽 테두리가 **이중 실선**으로 렌더됨.
한컴 정답지(이중 실선)와 정합 여부 작업지시자 시각 판정 요청.

> 단 위치 기준(paper/body)은 Stage 2-3 에서 처리 예정.
> 현재는 여전히 paper 기준(`attr=1`, `paper_based=true`) 이라
> 위치는 한컴과 다를 수 있음 — 본 Stage 는 **선 종류(이중선)만** 판정 대상.

## 후속 과제 (범위 외, 보고서 기록)

`border_type=2` → 현재 `BorderLineType::Dash`, 스펙은 "굵은 실선".
시각 판정 게이트 없이 변경 시 회귀 위험 → 별도 이슈 검토 권고.

## 다음 단계

Stage 2 — `layout.rs:944` `paper_based` attr 존중 복원 (회귀 민감 지점).
승인 후 진행.
