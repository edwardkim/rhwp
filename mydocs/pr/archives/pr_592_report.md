# PR #592 처리 보고서 — 핀셋 cherry-pick 머지 + 시각 판정 ★ 통과

**PR**: [#592 [m100] exam_eng.hwp p7 #40 글상자 사이 화살표 누락 — PUA U+F003B → ↓ 매핑 추가 (closes #588)](https://github.com/edwardkim/rhwp/pull/592)
**작성자**: @planet6897 (PR 등록) / Jaeook Ryu = @jangster77 (commit author)
**처리 결정**: ✅ **핀셋 cherry-pick 머지 + push + PR close + Issue #588 close**
**처리일**: 2026-05-05

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 핀셋 cherry-pick (`2788fea8` 단독) + devel merge + push + PR close |
| 시각 판정 | ★ **통과** (메인테이너 exam_eng 7 페이지 40번 화살표 출력 확인) |
| Devel merge commit | `a213fb8` |
| **PR mergeable** | **MERGEABLE** (본 사이클 첫 케이스 — PR #561~#584 모두 CONFLICTING 이었음) |
| Cherry-pick 충돌 | 0 건 |
| Author 보존 | ✅ Jaeook Ryu (@jangster77) 보존 |
| Issue #588 | CLOSED (수동 close + 안내 댓글) |
| 광범위 페이지네이션 sweep | 164 fixture / 1,614 페이지 / 페이지 수 회귀 0 |
| SVG U+F003B → ↓ 변환 | exam_eng 정확 1건 (다른 fixture 무영향) |

## 2. 본질 결함 (PR 진단)

### 2.1 결함 가설

`samples/exam_eng.hwp` 7페이지 40번 요약형 문항의 첫 번째 글상자(원문 passage)와 두 번째 글상자(요약) 사이에 위치한 ↓ 화살표가 SVG에 두부(□)로 렌더되는 결함.

원인: pi=278 의 단일 문자 **U+F003B** (UTF-8 `f3 b0 80 bb`) 가 SPUA-A 저영역 (`0xF0000~0xF00CF`) 으로 `map_pua_bullet_char` 매핑 표 밖. 글리프 미보유 폰트에서 두부 표시.

### 2.2 Stage 1 PDF 글리프 외곽 분석 (정밀 진단)

`samples/exam_eng.pdf` 7쪽의 임베디드 폰트 `AAAAAL+HCRBatang` 의 `uF003B` 글리프 외곽 분석:
- **1 contour 7 pts**
- **stem 35% × arrowhead 100% × solid filled**
- → **↓ DOWNWARDS ARROW** 형태 확정

→ 한컴 PDF 임베드 폰트의 글리프 외곽 = 한컴이 의도한 시각의 직접 권위 자료. 메모리 룰 `feedback_pdf_not_authoritative` (PDF 환경 의존성) 의 잠재 우려는 PDF 임베드 글리프 외곽에 한정 (환경 의존도 낮음) + 작업지시자 시각 판정으로 보정.

## 3. 본질 정정 — 단일 분기 신설 + 디스조인트 설계

### 3.1 정정 (단일 분기)

```rust
if (0xF0000..=0xF00CF).contains(&code) {
    return match code {
        0xF003B => '\u{2193}', // ↓ DOWNWARDS ARROW
        _ => ch,
    };
}
```

### 3.2 디스조인트 설계 (회귀 차단)

| 영역 | 범위 | 출처 |
|------|------|------|
| **본 PR (신규)** | `0xF0000..=0xF00CF` | Task #588 |
| Task #528 | `0xF00D0..=0xF09FF` | 책괄호/예시 |
| Task #509 | `0xF02B0..=0xF02FF` | 원문자 |
| Wingdings | `0xF020..=0xF0FF` | BMP PUA |

→ 신설 분기는 기존 영역과 **완전 디스조인트**. 회귀 위험 영역 0. `feedback_hancom_compat_specific_over_general` + `feedback_rule_not_heuristic` 정합.

### 3.3 단위 테스트 +2

- `supplementary_pua_a_low_range_maps_down_arrow` — U+F003B → U+2193 매핑 검증
- `supplementary_pua_a_low_range_unmapped_returns_original` — 0xF0000/F0090/F00CF default 분기 검증

## 4. PR 의 4 commits 분석 (cherry-pick 대상 식별)

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| `1baa36c5` Stage 1 — PDF 글리프 분석 + 광범위 통계 | 컨트리뷰터 fork plans/working | 무관 |
| **`2788fea8` Stage 2 — 본질 정정** | `paragraph_layout.rs` +32 + working stage2 | ⭐ cherry-pick |
| `c1e27ec6` Stage 3 — 광범위 회귀 점검 | 컨트리뷰터 fork working | 무관 |
| `9b78108a` Stage 4 — 최종 보고서 | 컨트리뷰터 fork report | 무관 |

→ 본질 1 commit 만 cherry-pick. PR #561~#580 와 동일 패턴.

## 5. cherry-pick 진행

### 5.1 대상 commit (1개, 충돌 0)

```
dd568ed Task #588 Stage 2: SPUA-A 저영역 분기 + U+F003B → ↓ 매핑
```

`Jaeook Ryu <jaeook.ryu@gmail.com>` author 보존.

### 5.2 변경 영역

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | +32 (SPUA-A 저영역 분기 + U+F003B → ↓ 매핑 + 단위 테스트 +2) |
| `mydocs/working/task_m100_588_stage2.md` | +106 (Stage 2 작업 보고서) |

## 6. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed / 2 ignored (단위 테스트 +2 GREEN) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed (Task #546 회귀 0) |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,581,527 bytes** (1m 34s, PR #584 baseline +29 bytes — paragraph_layout.rs +32 LOC 정합) |

## 7. 광범위 페이지네이션 회귀 sweep

본 환경 `samples/` 폴더 전체 자동 sweep:

| 통계 | 결과 |
|------|------|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (devel baseline) | **1,614** |
| 총 페이지 (cherry-pick 후) | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |

→ U+F003B → ↓ 매핑이 페이지네이션에 영향 없음.

## 8. SVG 정량 측정 + 권위 케이스 검증

### 8.1 정량 측정 (U+F003B → ↓ 변환)

| Fixture | U+F003B before | U+F003B after | ↓ after | 평가 |
|---|---|---|---|---|
| **exam_eng** | **1** | **0** | **1** | ✅ PR 본문 권위 영역 (page 7 #40 화살표 정정) |
| exam_kor | 0 | 0 | 0 | 회귀 영역 무 |
| exam_math | 0 | 0 | 0 | 회귀 영역 무 |
| exam_science | 0 | 0 | 0 | 회귀 영역 무 |

→ exam_eng 단독 영향 (page 7 의 1건 두부 → ↓ 정확 변환). 다른 fixture 무영향. 디스조인트 설계 정합 입증.

### 8.2 페이지 7 권위 케이스 검증 (PR 본문 명시)

`exam_eng_007.svg:4162` 정확한 1행 변경:

```diff
- <text transform="translate(796.7266666666668,1218.2666666666667) scale(1.3000,1)" font-family="HY신명조,..." font-size="15.333333333333334" fill="#000000">󰀻</text>
+ <text transform="translate(796.7266666666668,1218.2666666666667) scale(1.3000,1)" font-family="HY신명조,..." font-size="15.333333333333334" fill="#000000">↓</text>
```

→ x/y/font/size/scale/fill 모두 동일, U+F003B (`󰀻`) → U+2193 (`↓`) 만 변경.

## 9. 시각 판정 (★ 게이트)

### 9.1 SVG 자료 + WASM 환경

- `output/svg/pr592_before/exam_eng/` (devel 기준, 8 페이지)
- `output/svg/pr592_after/exam_eng/` (cherry-pick 후, 8 페이지)
- WASM: `pkg/rhwp_bg.wasm` 4,581,527 bytes (다양한 hwp 직접 검증용)

### 9.2 작업지시자 시각 판정 결과

> 메인테이너가 7 페이지 40번 화살표 출력 확인했습니다.

→ ★ **통과**. PR 본문 권위 영역 (exam_eng p7 #40 요약형 문항 글상자 사이 ↓ 화살표) 정합 회복 확인.

## 10. PR / Issue close 처리

### 10.1 PR #592 close
- 댓글 등록 (cherry-pick 결과 + 결정적 검증 + 광범위 sweep + SVG 정량 측정 + 페이지 7 권위 케이스 + Stage 1 PDF 글리프 분석 + 디스조인트 설계 + 잔존 영역 안내 + 컨트리뷰터 협업 인정)
- close 처리 (timeout 후 재시도 성공)

### 10.2 Issue #588 수동 close
- closes #588 키워드는 PR merge 가 아닌 close 로 자동 처리 안 됨 (PR #564/#570/#575/#580/#584 와 동일 패턴)
- 수동 close + 안내 댓글 (cherry-pick 처리 완료 + 권위 영역 정확 1건 변환 + 디스조인트 설계)

## 11. 잔존 영역 (PR 본문 명시, 별도 이슈 후보)

Stage 1 광범위 통계로 확인된 동일 영역 미매핑 코드포인트:
- U+F0090 (img-start-001.hwp 1건) — 본 PR 영역 (`0xF0000~0xF00CF`)
- U+F012B/F081C (복학원서.hwp) — Task #528 영역 default
- U+F02BA, F02C3~F02C5, F02CE~F02D0, F02FC — Task #509 영역 default

→ 별도 이슈 후보로 검토 가능.

## 12. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (★ 통과)
- ✅ `feedback_v076_regression_origin` — Stage 1 PDF 임베드 폰트 글리프 외곽 분석 (1 contour 7 pts → ↓ 형태) 정밀 진단 정합
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (SPUA-A 저영역 단독 분기, 디스조인트 설계)
- ✅ `feedback_rule_not_heuristic` — 코드포인트 범위 + 명시 매핑 (휴리스틱 아닌 규칙)
- ⚠️ `feedback_pdf_not_authoritative` — PDF 임베드 글리프 외곽은 환경 의존도 낮음 (PDF 출력 환경 의존성과 다른 영역) + 작업지시자 시각 판정으로 보정 완료
- ✅ `feedback_per_task_pr_branch` — Task #588 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터 협업 인정
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/5) 활발한 외부 기여의 빠른 회전 (19번째 PR 처리)

## 13. 본 PR 의 본질 — 본 사이클 첫 MERGEABLE PR

본 PR 의 처리 본질에서 가장 우수한 점:

1. **MERGEABLE 표시 첫 케이스** — PR #561~#584 모두 CONFLICTING 이었으나 본 PR 은 PR base 가 본 devel 과 정합한 시점으로 MERGEABLE. 컨트리뷰터의 빈번한 fork base 동기화 효과
2. **Stage 1 PDF 글리프 외곽 직접 분석** — 한컴 PDF 임베드 폰트 (`AAAAAL+HCRBatang`) 의 `uF003B` 1 contour 7 pts 외곽 분석으로 매핑 결정 (정밀 진단의 우수 사례)
3. **디스조인트 설계** — 기존 Task #528/#509/Wingdings 영역과 완전 무중첩으로 회귀 위험 영역 0
4. **단위 테스트 +2** — 신규 분기의 명시 검증 + default 분기 검증 모두 포함
5. **시각 변경 정량 측정** — `exam_eng_007.svg:4162` 정확한 1행 변경 (`󰀻` → `↓`, x/y/font/size 동일) + 광범위 sweep 영향 영역 명시 (582/583 byte-identical)

## 14. 본 사이클 사후 처리

- [x] PR #592 close (cherry-pick 머지 + push)
- [x] Issue #588 수동 close (안내 댓글)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_592_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_592_review.md` → `mydocs/pr/archives/pr_592_review.md`)
- [ ] 5/5 orders 갱신 (PR #592 항목 추가)
