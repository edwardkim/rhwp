# PR #999 검토 — fix: HWP5 sample16 페이지 수 HWP3 reference 정합 (64 페이지)

- 작성일: 2026-05-19
- 컨트리뷰터: [@jangster77](https://github.com/jangster77) (Taesup Jang)
- PR: https://github.com/edwardkim/rhwp/pull/999
- base/head: `devel` ← `jangster77:local/task998` (cross-repo fork)
- 연결 이슈: closes #998 (HWP3/HWP5/HWPX sample16 페이지 수 차이 64 vs 67 vs 69)
- 규모: +921 / -19, 13 files (소스 2: `composer.rs`, `typeset.rs`, 문서 11)
- mergeable: **CONFLICTING** (#997 적층 — #997 머지 완료로 본질 증분만 cherry-pick 시 해소)

## 1. PR 정보 + 적층 관계

PR #997 (Task #994 G4 word wrap) 위에 적층. PR 본문 명시: "PR #997 머지 후 머지 권장". **#997 은 직전 사이클에서 머지 완료** (`f44fd446`, devel 반영). 본 PR 의 본질 증분 = 단일 커밋 `c850148d` (Task #998), task_994 변경은 #997 와 중복이므로 cherry-pick 에서 제외.

## 2. 컨트리뷰터 사이클 (`feedback_contributor_cycle_check`)

@jangster77 24+ 사이클. #997(closes #994, 머지됨) → **#999(closes #998, 본 PR)** → #1005(closes #1001) → #1009 → #1011 연속. sample16 동일 대상의 겹침(#997) → 페이지 수(#999) 연쇄.

## 3. 변경 내용 분석 (본질 커밋 `c850148d`, +351/-3, 소스 2)

### A. composer.rs — CHARS_PER_LINE 35 → 45

```rust
const CHARS_PER_LINE: usize = 45;  // HWP3 reference 평균 43~46
```

PR #997 의 G4 fallback 휴리스틱 35 → 45 조정. 근거: HWP3 sample16 line_segs 실측 (pi=443 avg 44, 444 avg 46, 445 avg 45, 446 avg 53 → 평균 43~46 → 45 채택). 매 paragraph +1 wrap line 발생으로 인한 페이지 inflate 완화.

### B. typeset.rs — spacing_before 0 (line_segs-missing)

```rust
let raw_spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
// [Task #998 실험] spacing_before=0 으로 강제 — 효과 측정용
let spacing_before = if para.line_segs.is_empty() && !para.text.is_empty() {
    0.0
} else {
    raw_spacing_before
};
```

근거 (PR 본문): HWP3→HWP5 변환 시 ParaShape spacing_before 2x (Hancom 변환기 데이터 차이, pi=443 1132→2264 HU). 59 paragraph × 1132 HU ≈ 1 페이지 inflate.

## 4. 검토 의견

### 강점

1. **#997 의 직접 후속, 측정 근거 제시** — HWP3 line_segs 실측치로 45 도출. 페이지 수 62→64 (HWP3 reference 64 정합).
2. **HWP3 전용 분기 아님** — `line_segs.is_empty()` 일반 조건. CLAUDE.md HWP3 규칙 위반 아님.
3. **잔존 투명 명시** — 자동보정 path(69), HWPX 변종(72)이 본 fix 미통과임을 PR 본문이 인정 + 별도 task 분리 예정.
4. cargo test 1297 / fmt / 240 sample sweep 수행.

### ⚠️ 핵심 쟁점

#### (A) typeset.rs spacing_before=0 — 커밋 주석 자체가 "실험" + 공통 경로 무차별

코드 주석: **`// [Task #998 실험] spacing_before=0 으로 강제 — 효과 측정용`**. 커밋이 스스로 "실험/효과 측정용"으로 표기. `format_paragraph` 는 **공통 typeset 경로** (HWPX/HWP5/HWP3, 본문/각주/모든 paragraph, typeset.rs:923/1250/2002 호출). `para.line_segs.is_empty() && !para.text.is_empty()` 조건이 발동되는 **모든 paragraph 의 spacing_before 를 무조건 0 으로 무시**. ParaShape 의 정당한 spacing_before(문단 위 간격) 까지 제거 — sample16 외 line_segs 비는 paragraph(빈 셀, 변환본 등) 회귀 위험. `feedback_hancom_compat_specific_over_general` 위배 소지 (측정 의존 아닌 무차별 0 강제). PR 본문도 이를 "실험"으로 인정.

#### (B) CHARS_PER_LINE 45 — 여전히 측정 무관 고정 매직넘버

35→45 는 sample16 HWP3 실측 평균이나, 컬럼 폭·폰트 크기 무관 고정. PR 본문도 "잔존 ParaShape 데이터 차이" + "+1 까지 축소"로 완전 정합 아님 인정. #997 검토 쟁점 A 의 연장 — 임시 휴리스틱 (향후 reflow_line_segs 대체 약속).

#### (C) 한컴 정합 권위 (`feedback_self_verification_not_hancom`)

rhwp 64 = **HWP3 reference 64 정합**이 본 PR 목표. 그러나 HWP3/HWP5/HWPX 가 64/67/69 로 각각 다른 것은 한컴 변환기 자체의 ParaShape 데이터 차이. "HWP3 reference 64" 가 한컴 권위 페이지 수인지(작업지시자 한컴 직접 확인) vs rhwp 자기 기준 정합인지 구분 필요. PR 은 HWP3 를 reference 로 삼으나, HWP5/HWPX 의 한컴 정답 페이지 수는 별도 권위 확인 영역.

### 확인 필요 (검증 단계)

1. cherry-pick `c850148d` devel 적용 (충돌 — #997 머지 후이므로 composer.rs 양립 예상)
2. `cargo test --release --lib` + clippy -D + fmt 0
3. **광범위 sweep** — 특히 (A) **line_segs 비는 paragraph 보유 샘플 (빈 셀/변환본) 의 spacing_before=0 회귀** 집중 + hy-001 (PR 인정 신규 1건)
4. 작업지시자 시각 판정 — sample16-hwp5 page 19/22/23 + 페이지 수 64 가 회귀 없는 정합인지

## 5. 처리 옵션

- **옵션 A (수용)**: sweep 회귀 부재 + 작업지시자 시각 판정 통과 시. 단 쟁점 A(spacing_before=0 실험 코드 공통 경로 회귀) 와 hy-001 변동 작업지시자 판정 필수.
- **옵션 B (수정 요청)**: 쟁점 A 가 다른 샘플 회귀로 확인되면 — typeset spacing_before=0 을 sample16 변환본 케이스로 좁히거나 (구조 가드), "실험" 주석 제거 + 근거 명문화 요청. composer 45 는 #997 연장이므로 유지 가능.
- **옵션 C (close)**: 본질 결함 시. 실제 페이지 수 정합 효과 있어 해당 낮음.

## 6. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @jangster77 #997→#999→#1005~ 연속 시리즈 순차
- `feedback_hancom_compat_specific_over_general` — **쟁점 A/B**: spacing_before=0 무차별 + CHARS_PER_LINE 고정. 일반화 위험 (권위 우려 사례)
- `feedback_self_verification_not_hancom` — **쟁점 C**: HWP3 reference 정합 ≠ 한컴 권위 페이지 수. 작업지시자 한컴 직접 확인 영역
- `feedback_fix_scope_check_two_paths` — 자동보정(reflow_line_segs) path 우회 (PR 인정 잔존), 본 fix 는 composer fallback path 한정
- `feedback_visual_judgment_authority` — sample16 페이지 수 64 + hy-001 변동 작업지시자 시각 판정 최종 게이트
- `feedback_v076_regression_origin` — 컨트리뷰터 자기 환경 reference(HWP3 64)로 정합 시 작업지시자 환경 회귀 가능 — 시각 판정 게이트 필수

## 7. 권고

**옵션 A 조건부** — 검증 단계에서 (1) test/clippy/fmt GREEN, (2) **쟁점 A(spacing_before=0 공통 경로) sweep + 작업지시자 시각 판정으로 회귀 부재 확인**, (3) sample16-hwp5 64 페이지 + page 19/22/23 시각 판정 통과 시 cherry-pick no-ff merge. 쟁점 A 가 회귀로 확인되면 옵션 B(케이스 한정 또는 "실험" 코드 정식화 요청). 자동보정/HWPX 변종 잔존은 PR 본문대로 후속 task 분리.
