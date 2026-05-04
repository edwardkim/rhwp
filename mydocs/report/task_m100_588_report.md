# Task #588 최종 결과 보고서 — exam_eng.hwp p7 #40 글상자 사이 화살표 누락 (PUA U+F003B 매핑 추가)

## 요약

- **이슈**: [#588](https://github.com/edwardkim/rhwp/issues/588) (M100, v1.0.0)
- **본질**: HWP SPUA-A 저영역 (`0xF0000~0xF00CF`) 코드포인트 U+F003B 가 `map_pua_bullet_char` 매핑 표 밖이어서 SVG 에 두부(□) 표시
- **정정**: SPUA-A 저영역 분기 신설 + U+F003B → ↓ U+2193 (DOWNWARDS ARROW) 매핑 추가
- **결과**: exam_eng.hwp p7 #40 요약형 문항 글상자 사이 화살표 정합 표시. 광범위 회귀 0 (582/583 byte-identical).

## 변경 영역

### 1. `src/renderer/layout/paragraph_layout.rs`

#### 1.1 `map_pua_bullet_char` SPUA-A 저영역 분기 신설

```rust
// Supplementary PUA-A 저영역 — 한컴 자체 영역 (Task #588 한컴 정답지 정합)
if (0xF0000..=0xF00CF).contains(&code) {
    return match code {
        // exam_eng.hwp p7 #40 요약형 문항 글상자 사이 화살표.
        // 한컴 PDF (HCRBatang 임베디드 폰트) 글리프 외곽 분석:
        //   stem 35% × arrowhead 100% × solid filled (1 contour, 7 pts) → ↓
        0xF003B => '\u{2193}', // ↓ DOWNWARDS ARROW
        _ => ch,
    };
}
```

#### 1.2 docstring 갱신

기존 "두 영역 분기" → 세 영역 (Basic / SPUA-A 저영역 / SPUA-A 일반) 으로 정합 갱신.

#### 1.3 단위 테스트 +2

- `supplementary_pua_a_low_range_maps_down_arrow` (U+F003B → U+2193)
- `supplementary_pua_a_low_range_unmapped_returns_original` (F0000/F0090/F00CF default)

## 단계 요약

| 단계 | 산출물 | 핵심 결과 |
|------|--------|----------|
| Stage 1 | `mydocs/working/task_m100_588_stage1.md` | PDF 임베디드 폰트 글리프 외곽 분석 (↓ 형태 확정) + 159 샘플 광범위 통계 (target 영역 U+F003B 1건 + U+F0090 1건) |
| Stage 2 | `paragraph_layout.rs` + `_stage2.md` | 분기 신설 + 매핑 + 단위 테스트 +2 (7/7 GREEN) |
| Stage 3 | `_stage3.md` | 광범위 회귀 점검: 583 SVG → 582 byte-identical, 1 의도된 변경 (exam_eng/p7), 회귀 0 |
| Stage 4 | 본 보고서 | 작업지시자 시각 판정 + 이슈 close |

## 결정적 검증 게이트 (Stage 3 통과)

| 게이트 | 결과 |
|--------|------|
| cargo test --lib | **1126 passed** (3 ignored, 0 failed; baseline 1124 + 신규 2) |
| cargo test --test svg_snapshot | **6/6 GREEN** |
| 통합 테스트 (issue_*, exam_eng_multicolumn) | **모두 통과** (issue_418/501/505/514/516/530/546 + exam_eng_multicolumn) |
| cargo clippy --lib -- -D warnings | **0 신규** |
| WASM 빌드 | **4,529,640 bytes** 정합 |
| 14 fixture 광범위 byte sweep | **582/583 byte-identical** (회귀 0) |

## 시각 변경 (작업지시자 판정 게이트)

`exam_eng.hwp` 7페이지 → 단 1 → 40번 문제:

**변경 전**:
```
[원문 passage 글상자 (pi=277)]
       □                       ← 두부 (U+F003B 글리프 미보유)
[요약 (A) … (B) … 글상자 (pi=279)]
```

**변경 후**:
```
[원문 passage 글상자 (pi=277)]
       ↓                       ← U+2193 DOWNWARDS ARROW (모든 폰트에서 글리프 보유)
[요약 (A) … (B) … 글상자 (pi=279)]
```

**작업지시자 시각 판정 권위** (메모리 룰 `reference_authoritative_hancom`).

## 회귀 차단 설계

신설 분기 (`0xF0000..=0xF00CF`) 는 기존 분기 영역과 **완전 디스조인트**:

| 영역 | 범위 | 차이 |
|------|------|------|
| 본 사이클 (Task #588) | `0xF0000..=0xF00CF` | (신규) |
| Task #528 (책괄호/예시) | `0xF00D0..=0xF09FF` | 0xF00D0 직전에서 종료 |
| Task #509 (원문자) | `0xF02B0..=0xF02FF` | 영역 분리 |
| Wingdings (Task #509) | `0xF020..=0xF0FF` | BMP PUA, SPUA-A 와 무중첩 |

설계 검증: 583 SVG sweep 에서 582 byte-identical → 디스조인트 설계 결정적 확인.

## 잔존 영역 (별도 task 후보)

Stage 1 광범위 통계로 확인된 동일 영역 미매핑 코드포인트:

| 코드포인트 | 분포 | 비고 |
|---|---|---|
| U+F0090 | img-start-001.hwp 1건 | 본 task 영역 (`0xF0000~0xF00CF`) — 시각 판정 후 별도 이슈 |
| U+F012B | 복학원서.hwp 1건 | Task #528 영역 default |
| U+F081C | 복학원서.hwp 2건 | Task #528 영역 default |
| U+F02BA, F02C3~F02C5 | mel-001.hwp | Task #509 영역 default |
| U+F02CE~F02D0 | k-water-rfp.hwp | Task #509 영역 default |
| U+F02FC | pic-in-head/table 22건 | Task #509 영역 default |

작업지시자 결정 대기 — 일괄 이슈 등록 후 별도 task 사이클로 처리 가능.

## 다음 단계

1. 작업지시자 시각 판정:
   - `output/svg/exam_eng_p7_after/exam_eng_007.svg` 7쪽 40번 문제 글상자 사이 ↓ 표시 정합 확인
   - (선택) `samples/exam_eng.pdf` 7쪽과 시각 비교
2. 시각 판정 통과 시 이슈 #588 close + `local/task588` → `local/devel` merge
3. 오늘할일 (`mydocs/orders/20260504.md`) 갱신

## 메모리 룰 정합

- `feedback_hancom_compat_specific_over_general` — 단일 코드포인트 한정 매핑 (광범위 영역 일괄 매핑 회피)
- `reference_authoritative_hancom` — 한컴 PDF 시각 정답지 권위 (Stage 1 분석)
- `feedback_visual_regression_grows` — 광범위 byte sweep + 시각 판정 게이트
- `feedback_essential_fix_regression_risk` — 분기 디스조인트 설계로 회귀 위험 차단
- `feedback_no_pr_accumulation` — 같은 영역 다른 코드포인트는 별도 task

## 산출물 목록

```
mydocs/plans/task_m100_588.md                  — 수행 계획서
mydocs/working/task_m100_588_stage1.md         — Stage 1 (PDF 분석 + 통계)
mydocs/working/task_m100_588_stage2.md         — Stage 2 (구현)
mydocs/working/task_m100_588_stage3.md         — Stage 3 (회귀 점검)
mydocs/report/task_m100_588_report.md          — 최종 보고서 (본 문서)

src/renderer/layout/paragraph_layout.rs        — 분기 신설 + 단위 테스트 +2 (32 lines added)

output/svg/exam_eng_p7_after/exam_eng_007.svg  — Stage 2 후 SVG (시각 판정 대상)
```
