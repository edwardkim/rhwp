---
PR: #737
제목: Task #615 — pua_oldhangul U+F53A 매핑 제거 (hwpspec '매핑 표 외' 정합)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 6번째 PR)
처리: 옵션 A — 2 commits cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: 6ea6f851
---

# PR #737 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge `6ea6f851`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `6ea6f851` (--no-ff merge) |
| Cherry-pick commits | `c1d5e8e5` (Task #615) + `efe1867e` (Copilot 리뷰) |
| closes | #615 |
| 시각 판정 | ✅ 통과 (작업지시자 SVG 시각 판정) + WASM 빌드 4.66 MB |
| 자기 검증 | cargo test ALL GREEN + pua tests 13 PASS + sweep 170/170 same |

## 2. 정정 본질 — 1 file, +24/-4

### 2.1 `src/renderer/pua_oldhangul.rs` (+24/-4)

- **U+F53A 매핑 항목 제거** (line 5022)
  - 기존: `(0xF53A, &['\u{1112}', '\u{119E}', '\u{11AB}'])` (자모 시퀀스 ᄒᆞᆫ)
  - 정정: 매핑 제거 (한컴 정답지 영역 영역 빈 공백 정합)
- **매핑 수 동기화**: 5660 → 5659 (헤더 doc + `test_map_size`)
- **자동 생성 안전성 코멘트**: `scripts/gen_pua_oldhangul_rs.py` 재생성 시 주의 + 회귀 가드 명시
- **회귀 가드 신규**: `test_hwpspec_unmapped_codepoints_not_in_table` — 재삽입 차단

## 3. 결함 본질 (Issue #615)

### 3.1 한컴 정답지 vs 본 환경 (정정 전)

| 영역 | U+F53A 처리 |
|------|-------------|
| **한컴 PDF 정답지** | hwpspec "Basic-out, (매핑 표 외)" — 매핑 미지원, **빈 공백 표시** |
| 본 환경 SVG (정정 전) | 자모 시퀀스 `ᄒᆞᆫ` 영역 영역 임의 변환 → 옛한글 자모 글리프 미렌더 → 박스 표시 |

### 3.2 출처 영역
KTUG HanyangPuaTableProject (자동 생성 영역 영역 의 원본 데이터) 영역 영역 hypua2jamo 영역 영역 의 의도 영역 영역 hwpspec 영역 영역 차이 — 한컴 영역 영역 의 spec 정합 우선.

## 4. 본 환경 cherry-pick + 검증

### 4.1 cherry-pick (2 commits)
충돌 0건.

### 4.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (32.25s) |
| pua tests (`cargo test --lib pua`) | ✅ **13 PASS** (신규 1건 + 기존 12건, map_size 5659) |
| `cargo test --release` (전체) | ✅ ALL GREEN, failed 0 |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** (U+F53A 영역 영역 7 fixture 영역 영역 사용 부재) |
| WASM 빌드 (Docker) | ✅ 4.66 MB |

### 4.3 작업지시자 시각 판정 ✅ 통과

`samples/pua-test.hwp` 영역 영역 SVG 출력 영역 영역 빈 공백 정합 (한컴 PDF 정답지 영역 영역 정합) 영역 영역 작업지시자 점검 ✅.

## 5. PR supersede 체인 — 별 패턴

PR #600 (closes #513, @oksure) Supplementary PUA-A 시각 판정 중 발견 영역 영역 별개 영역 영역 결함 영역 영역 → 본 PR 영역 영역 별 PR 영역 영역 분리 후속. `feedback_pr_supersede_chain` 영역 영역 다른 패턴 (별 영역 영역 회귀 발견 → 별 PR 영역 영역 후속) — 동일 컨트리뷰터.

## 6. 영향 범위

### 6.1 변경 영역
- pua_oldhangul 영역 영역 U+F53A 영역 영역 매핑 미지원 (한컴 정답지 정합)
- 자동 재생성 영역 영역 회귀 가드 (재삽입 차단)

### 6.2 무변경 영역 (sweep 170/170 same 영역 영역 입증)
- 다른 PUA 매핑 5658개 영역 영역 보존
- HWP3 한자 영역 (`johab_map.rs`) 영역 영역 무관 (별 영역)
- 다른 layout/render 경로 영역 영역 무영향
- HWP3/HWPX 변환본 영역 영역 시각 정합 (광범위 sweep 영역 영역 입증)

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735/#737 영역 6번째 PR) |
| `feedback_pr_supersede_chain` | 별 패턴 — PR #600 시각 판정 중 발견된 별 영역 영역 결함 영역 영역 별 PR 영역 영역 분리 후속 |
| `feedback_image_renderer_paths_separate` | pua_oldhangul.rs 영역 영역 격리 — 다른 layout/render 경로 영역 영역 무영향 |
| `feedback_hancom_compat_specific_over_general` | U+F53A 영역 영역 단일 코드포인트 영역 영역 case 가드 — 일반화 매핑 영역 영역 영역 회귀 본질 |
| `feedback_visual_judgment_authority` | 한컴 PDF 정답지 영역 영역 정합 영역 영역 권위 — 작업지시자 SVG 시각 판정 ✅ 통과 |
| `feedback_process_must_follow` | 자동 생성 영역 영역 안전성 코멘트 + 회귀 가드 영역 영역 재삽입 차단 — 위험 좁힘 |

## 8. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- HWP3 한자 영역 (`johab_map.rs:4969` U+F53A → 慽) 영역 영역 별 포맷 영역 영역 별 의미 — 본 PR 영역 영역 무관

---

작성: 2026-05-10
