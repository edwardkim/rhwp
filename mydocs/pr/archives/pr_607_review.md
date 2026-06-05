# PR #607 검토 보고서

**PR**: [#607 fix: HWPX ColorRef 상위 바이트 보존 (closes #606)](https://github.com/edwardkim/rhwp/pull/607)
**작성자**: @dicebattle (Sunyong Lim) — **첫 PR 컨트리뷰터** / commit author: dice-liner (dice@linercorp.com)
**상태**: OPEN, **mergeable=MERGEABLE**, **mergeStateStatus=BEHIND** (PR base 90 commits 뒤 — 5/5 등록 후 본 사이클 5/5+5/6 처리분 누적)
**관련**: closes #606
**처리 결정**: ⏳ **검토 중** (1차 검토 — 컨트리뷰터에게 샘플 파일 + 한컴 버전 정보 보완 요청 댓글 등록)
**검토 시작일**: 2026-05-06

## 1. 검토 핵심 질문

1. **본질 결함 식별 정합성** — HWPX `parse_color_str` 의 8자리 `#AARRGGBB` 포맷 처리에서 상위 바이트 손실 → `style_resolver.rs:700` 의 상위 바이트 의존 분기 (특수값/확장값 → 채우기 없음) 미발동 → `#FF000000` + `alpha="0"` 조합이 검정 채우기로 렌더링되는 결함이 본 환경에서도 재현되는가?
2. **회귀 위험** — 단일 함수 수정 (+9/-5) 이지만 호출처 광범위 (textColor/shadeColor/borders/grad 등 모든 색상 영역). 다른 색상이 `#AARRGGBB` 8자리 포맷으로 들어왔을 때 상위 바이트 보존이 의도 정합인가?
3. **PR base skew (5/5 등록 → 본 사이클 5/5+5/6 처리분 90 commits 뒤)** — 본 환경 cherry-pick 충돌 0?
4. **첫 PR 컨트리뷰터 영역** — `feedback_pr_comment_tone` (차분/사실 중심) + `feedback_check_open_prs_first` (Issue assignee 점검) 정합 운영

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | fix: HWPX ColorRef 상위 바이트 보존 | 정합 (작은 fix, 명료한 본질) |
| author | @dicebattle (Sunyong Lim) — **첫 PR 컨트리뷰터** | 신규 컨트리뷰터, AI 도구 (Codex) 사용 명시 |
| commit author | dice-liner (dice@linercorp.com) | GitHub @dicebattle 와 commit author dice-liner 매핑 (linercorp 직장 이메일) |
| changedFiles | **1** / +9 / -5 | **단일 파일** (가장 작은 fix 패턴) |
| 본질 변경 | `src/parser/hwpx/utils.rs::parse_color_str` (단일 함수 +4/-2) + 단위 테스트 (+5/-3) | 단일 함수 |
| **mergeable** | MERGEABLE (UI), **mergeStateStatus=BEHIND** (PR base 90 commits 뒤) | 본 환경 cherry-pick 충돌 0 확인 |
| Issue | closes #606 (본인 등록 + 본인 정정) | ✅ |
| Issue assignee | 미지정 (memory `feedback_assign_issue_before_work` 영역) | 본 환경 처리 시 assignee 영역 점검 |

## 3. PR 의 1 commit 분석

| commit | 설명 | 본 환경 처리 |
|--------|------|-------------|
| **`de0250ee`** fix: HWPX ColorRef 상위 바이트 보존 | `parse_color_str` +9/-5 (단일 파일) | ⭐ cherry-pick |

→ **단일 본질 commit**. PR #561~#629 와 다른 패턴 (fork plans/working/report 부재 — 컨트리뷰터가 본 프로젝트의 hyper-waterfall 절차를 따르지 않은 첫 PR, 신규 컨트리뷰터 영역 정합).

## 4. 본질 변경 영역

### 4.1 결함 가설 (PR 본문 + Issue #606 인용)

HWPX `faceColor="#AARRGGBB"` 8자리 색상 파서 (`parse_color_str`) 가 상위 바이트 (alpha) 를 버리고 `0x00BBGGRR` 로만 변환:

```rust
// 결함 코드 (devel HEAD)
} else if hex.len() == 8 {
    // AARRGGBB → 0x00BBGGRR (alpha 무시)
    if let Ok(v) = u32::from_str_radix(hex, 16) {
        let r = (v >> 16) & 0xFF;
        let g = (v >> 8) & 0xFF;
        let b = v & 0xFF;
        return b << 16 | g << 8 | r;
    }
}
```

`#FF000000` (alpha=0xFF, RGB=0x000000) → `0x00000000` (alpha 손실 + RGB 검정).

`style_resolver.rs:700` 의 분기:
```rust
if (s.background_color >> 24) != 0 {
    None  // 채우기 없음 (특수값/확장값)
} else {
    Some(s.background_color)
}
```

→ alpha 손실로 상위 바이트 0 → 일반 검정 채우기 fallback → 표 셀 배경이 검정으로 렌더링 (Issue #606 의 발현).

### 4.2 정정 (`parse_color_str` 단일 함수 +4/-2)

```rust
// 신규
} else if hex.len() == 8 {
    // AARRGGBB → 0xAABBGGRR
    // 상위 바이트는 HWP ColorRef의 확장/특수값 판별에 쓰이므로 보존한다.
    if let Ok(v) = u32::from_str_radix(hex, 16) {
        let a = (v >> 24) & 0xFF;
        let r = (v >> 16) & 0xFF;
        let g = (v >> 8) & 0xFF;
        let b = v & 0xFF;
        return a << 24 | b << 16 | g << 8 | r;
    }
}
```

→ alpha 보존 (`a << 24` 추가) + 주석 갱신.

**HWP5 vs HWPX 정합 회복**: HWP5 `byte_reader.rs::read_color_ref` 는 4바이트 그대로 (`0x00BBGGRR` 또는 상위 바이트 비0 특수값) 반환하지만, HWPX `parse_color_str` 만 상위 바이트 손실 → 비정합 → 본 PR fix 가 **HWP5/HWPX 정합 회복** 본질.

### 4.3 단위 테스트 갱신 (+5/-3)

```rust
fn test_parse_color_str_with_alpha() {
    // AARRGGBB — RGB는 BGR로 바꾸되 상위 바이트는 보존
    assert_eq!(parse_color_str("#80FF0000"), 0x800000FF);
    assert_eq!(parse_color_str("#FF000000"), 0xFF000000);
    assert_eq!(parse_color_str("#FFFFFFFF"), 0xFFFFFFFF);
}
```

→ `#FF000000` (Issue #606 권위 케이스) + `#FFFFFFFF` (CLR_INVALID) + 반투명 예시 모두 검증.

### 4.4 회귀 위험 영역 점검

`parse_color` / `parse_color_str` 호출처 (모든 색상 영역):
- `header.rs`: textColor / shadeColor / underline_color / strike_color / shadow_color / borders / diagonal / faceColor / hatchColor / grad.colors
- `section.rs`: faceColor / hatchColor

**호출처 광범위하지만**:
- 6자리 `#RRGGBB` 포맷은 무영향 (분기 분리, 변경 없음)
- 8자리 `#AARRGGBB` 포맷은 상위 바이트 보존만 변경 — 다른 색상 영역에서 8자리 + 비0 alpha 가 들어오는 경우는 현재 다른 처리 코드에서 무시됐던 영역이므로, 보존이 의도 정합 (HWP5 와 동일 영역)

`style_resolver.rs:700` + `layout/utils.rs:126` 두 곳만 `(s.background_color >> 24) != 0` 패턴 — **`background_color` (= `faceColor`) 만 상위 바이트 의존**, 다른 색상은 무영향.

## 5. 본 환경 직접 검증 (임시 브랜치 `pr607-cherry-test`)

| 단계 | 결과 |
|------|------|
| `de0250ee` cherry-pick | ✅ 단일 파일 충돌 0 (orders 충돌 없음 — 컨트리뷰터 fork plans/working/report/orders 부재) |
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1140 passed** / 0 failed (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --release --test issue_546 --test issue_554` | ✅ 모두 통과 |
| `cargo clippy --release --lib` | ✅ 0건 |
| Docker WASM 빌드 | ✅ **4,588,198 bytes** (PR #629 baseline 4,590,307 -2,109 — alpha 보존 코드는 작지만 dead code elimination 등 LLVM 최적화 영향) |

→ **본 환경 base skew 90 commits 영향 0** — 단일 파일 충돌 0 + 결정적 검증 모두 통과.

## 6. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (BEFORE) | **1,684** |
| 총 페이지 (AFTER) | **1,684** |
| **fixture 별 페이지 수 차이** | **0** |

→ ColorRef 상위 바이트 보존이 페이지네이션에 영향 없음 (회귀 0).

## 7. SVG 시각 영역 측정 (PR 본문 정량 영역)

PR 본문 명시 정량:
- `123kb.hwp` 2페이지: `rect fill="#000000"` 65 → 0
- `74kb.hwp` 1페이지: `rect fill="#000000"` 31 → 0

본 환경 11 fixture (HWPX 6 + exam_* 5) 의 `rect fill="#000000"` 측정:

| Fixture | BEFORE | AFTER | 평가 |
|---|---|---|---|
| 2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 0 | 0 | 본 환경 미발현 |
| hwp3-sample-hwpx.hwpx | 0 | 0 | 본 환경 미발현 |
| hwp3-sample5-hwpx.hwpx | 0 | 0 | 본 환경 미발현 |
| table-vpos-01.hwpx | 0 | 0 | 본 환경 미발현 |
| tac-img-02.hwpx | 0 | 0 | 본 환경 미발현 |
| 표-텍스트.hwpx | 0 | 0 | 본 환경 미발현 |
| exam_eng/kor/math/science/social.hwp | 0 | 0 | 본 환경 미발현 |

→ **본 환경 fixture 들은 이 결함이 발현되지 않음** — `faceColor="#FF000000"` + `alpha="0"` 조합이 흔치 않은 **엣지 케이스**.

**중요**: PR 본문 권위 영역 샘플 (`123kb.hwp` / `74kb.hwp`) **본 환경 미존재** → 메인테이너 직접 시각 판정 불가.

## 8. 메인테이너 정합성 평가

### 정합 영역 — 우수
- ✅ **본질 진단 정확** — HWP5 `read_color_ref` 와 HWPX `parse_color_str` 비정합 영역 정확 식별
- ✅ **HWP5/HWPX 정합 회복** — 본 fix 가 HWP5 표준 영역 (`0xAABBGGRR` 보존) 으로 정합 회복
- ✅ **`style_resolver.rs:700` 분기 활용** — 기존 코드 변경 없이 의도된 fix 영역 발동
- ✅ **단일 함수 +9/-5** — 가장 작은 fix 영역, 회귀 위험 영역 좁힘
- ✅ **단위 테스트 갱신** — `#FF000000` (권위 케이스) + `#FFFFFFFF` (CLR_INVALID) + 반투명 모두 검증
- ✅ **결정적 검증 정합** — 1140 passed / clippy 0 / svg_snapshot 6/6 / 광범위 sweep 회귀 0
- ✅ **자기 진단 정합** — Issue #606 + PR 본문 모두 한글 + 결함 가설 + 정정 후보 검증 + 환경 정보 명시
- ✅ **AI 도구 (Codex) 사용 명시** — 투명한 협업 영역
- ✅ **신규 컨트리뷰터 첫 PR** — 깔끔한 영역, hyper-waterfall 부재 영역도 본질 fix 의 작음으로 인해 절차 부담 없이 정합

### 우려 영역
- ⚠️ **샘플 파일 본 환경 미존재** — `123kb.hwp` / `74kb.hwp` 본 환경 `samples/` 에 없어 메인테이너 시각 판정 자료 부재. **컨트리뷰터에게 샘플 동봉 요청 + 한컴 버전 정보 보완 요청** (작업지시자 안내 정합)
- ⚠️ **PR base 90 commits 뒤** — UI MERGEABLE 표시지만 본 환경 cherry-pick 충돌 0 확인 (저위험 영역)
- ⚠️ **Issue #606 / PR #607 모두 컨트리뷰터 본인 작성** — 자기 진단 영역 (외부 리뷰 영역 부재) 이지만 본 PR 의 본질 (작은 단일 함수) 로 위험 영역 충분히 좁음

## 9. 컨트리뷰터에게 보낸 보완 요청 댓글 (작업지시자 안내 정합)

작업지시자 안내:
1. "샘플 파일들도 함께 PR에 포함시켜야 메인테이너가 시각 판정이 가능하다고 커멘트를 해주세요."
2. "가능하면 해당 hwp 파일이 한컴의 어떤 버전으로 작성했는지도 제공하면 좋겠습니다."

**한글 댓글 등록 완료** — `comment-4389114416`:
- 샘플 파일 (`123kb.hwp` / `74kb.hwp`) PR 동봉 요청 (`samples/` 또는 `samples/issue-606/` 영역)
- 한컴 버전 정보 (작성/저장 환경) 요청
- 본 환경 결정적 검증 요약 (참고 영역)
- 본 PR 의 본질 진단 + 정정 영역 정합 인정 (본 PR 평가 톤 — 차분/사실 중심, `feedback_pr_comment_tone` 정합)

## 10. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능** — `de0250ee` 단일 파일 충돌 0
- ✅ **결정적 검증** — 1140 passed / clippy 0 / svg_snapshot 6/6 / 광범위 sweep 회귀 0
- ✅ **HWP5/HWPX 정합 회복** — 본 PR 의 본질 영역
- ✅ **단일 함수 영역 좁힘** — 회귀 위험 좁음
- ⏳ **샘플 파일 + 한컴 버전 정보 보완 대기** — 컨트리뷰터 응답 영역

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 컨트리뷰터 응답 후 머지 (권장, 작업지시자 안내 정합)
- 샘플 파일 동봉 + 한컴 버전 정보 응답 대기
- 응답 도착 시 본 환경 샘플 도입 + 시각 판정 진행 + 머지

#### 옵션 B — 즉시 머지 (응답 없이)
- 본질 정합성 + 결정적 검증 모두 통과로 즉시 cherry-pick + merge 가능
- 단, 시각 판정 게이트 부재 → 본 사이클 운영 패턴 (`feedback_visual_regression_grows`) 비정합

#### 옵션 C — close + 후속 영역
- 샘플 파일 부재 + 본 환경 미발현 으로 close + 별도 task 영역으로 분리

→ **작업지시자 결정 대기**. 옵션 A 권장 (컨트리뷰터 응답 후 머지).

## 11. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 페이지네이션 sweep (164 fixture / 1,684 페이지) + 1140 passed 회귀 0
- ✅ `feedback_hancom_compat_specific_over_general` — 단일 함수 영역, 8자리 포맷만 보존 (case-specific)
- ✅ `feedback_rule_not_heuristic` — HWP ColorRef 표준 (HWP5 4바이트 보존) 와 정합 회복, 휴리스틱 미도입
- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (샘플 파일 보완 요청 — 시각 판정 자료 영역 보장)
- ✅ `feedback_pdf_not_authoritative` — PDF 미사용, SVG byte 측정 + HWP5/HWPX 정합 영역
- ✅ `feedback_per_task_pr_branch` — Task #606 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 첫 PR 컨트리뷰터 환영 + 차분/사실 중심 톤 (과도한 표현 자제, `feedback_pr_comment_tone` 정합)
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + 본 사이클 처리분 점검 후 진행
- ✅ `feedback_assign_issue_before_work` — Issue #606 assignee 미지정 영역 점검 (외부 컨트리뷰터 본인 작성 + 본인 정정으로 영역 정합 — 외부 기여 자율 영역)
- ✅ `feedback_small_batch_release_strategy` — v0.7.10 후 세 번째 PR 처리 영역
- ✅ **신규 컨트리뷰터 첫 PR 영역** — 본 사이클 (`feedback_pr_comment_tone` 의 과도한 표현 자제) + 정중한 보완 요청 패턴 정합

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**1차 검토 단계 — 컨트리뷰터 응답 + 작업지시자 결정 대기**.
