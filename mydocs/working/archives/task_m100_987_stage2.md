# Stage 2-3 완료보고서 — Task #987

> 구현계획서 Stage 2(layout.rs attr 복원) + Stage 3(HWP3 attr=0) 을
> 단일 단계로 통합 진행. 둘은 한 쌍으로만 유효(attr 존중 + attr 주입).

## 단계 목표

HWP3 쪽 테두리 위치를 한컴 정답지대로 **본문(body) 기준**으로 정합.

## 변경 내용

### Stage 2 — `src/renderer/layout.rs:944`

- `let paper_based = true;` (전역 하드코딩, #952)
  → `let paper_based = (pbf.attr & 0x01) != 0;` (attr bit0 존중 복원)
- HWPX/HWP5 본래 의미(textBorder=PAPER) 그대로. 주석에 #952 "sample16
  paper 정합" 이 bfid off-by-one 착시였음을 기록.

### Stage 3 — `src/parser/hwp3/mod.rs:2824`

- `attr: 1` (paper) → `attr: 0` (body)
- sample16 한컴 정답지 = body 기준 (작업지시자 시각 판정).
- CLAUDE.md HWP3 격리 규칙 준수: layout.rs 는 포맷 중립 attr 해석만,
  HWP3 고유 판단(body)은 파서가 attr 로 표현.

## 측정 결과 (sample16 page 0)

| 항목 | Stage 1 (paper) | Stage 2-3 (body) | body_area 기반 검산 |
|------|-----------------|------------------|---------------------|
| 디버그 | attr=0x01 paper=true | attr=0x00 **paper=false** | — |
| 상변 이중선 y | 17.9 / 20.0 | **55.6 / 57.7** | 75.6 − 18.9 ≈ 56.7 ✅ |
| 하변 이중선 y | 1102.5 / 1104.6 | **1086.1 / 1088.2** | 1068.2 + 18.9 ≈ 1087.1 ✅ |

body_area: y=75.6, h=992.6 (하단 1068.2). spacing 1420 HU ≈ 18.9px.
본문 경계에서 바깥쪽으로 spacing 만큼 확장 — 검산 일치.

## 회귀 검증 (Stage 4 필수 게이트)

attr 존중 복원이 #920/#952 회귀 history 지점이므로 시험지 계열 점검:

| 샘플 | attr | paper_based | 판정 |
|------|------|-------------|------|
| `21_언어_기출_편집가능본.hwp` (HWP5 시험지) | 0x01 | true | 복원 후도 paper 유지 → **회귀 없음** |
| `exam_eng.hwp` (HWP5 시험지) | 0x01 | true | 복원 후도 paper 유지 → **회귀 없음** |
| `hwp3-sample16.hwp` (HWP3 native) | 0x00 | false | 파서 주입 → body (의도) |

- HWP5 시험지는 원본 attr bit0=1 → 복원 로직에서 paper 유지.
  #952 이전과 동일 동작. 게다가 spacing=0 이라 위치 차 미미.
- HWP3 native 만 파서가 attr=0 주입 → body. 타 포맷 무영향 (격리).
- `hwp3-sample-hwpx.hwpx` (HWP3→HWPX 변환본) 는 HWPX 파서 경로라
  본 수정 대상 아님 (변환기 attr 별개 사안).

| 검증 | 결과 |
|------|------|
| `cargo test` | ✅ 1476 passed, 0 failed |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| HWP5 시험지 회귀 | ✅ 없음 (paper 유지) |

산출물: `output/poc/task987_stage3/hwp3-sample16_001.svg`

## 시각 판정 요청

sample16 페이지 0 쪽 테두리:
- 선 종류: 이중 실선 (Stage 1 판정 통과)
- 위치: **본문 기준** 으로 상변 ≈56.7, 하변 ≈1087 이동

한컴 정답지(body 기준)와 윗쪽/아랫쪽 위치 정합 여부 작업지시자 시각 판정 요청.
다중 페이지 확인 필요 시 p1/p2 도 추가 export 가능.

## 다음 단계

Stage 4 — 시각 판정 통과 시 최종 보고서 작성 + PR #971 stash 복귀 처리.
