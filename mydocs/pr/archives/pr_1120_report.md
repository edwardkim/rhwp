# PR #1120 처리 보고 — sample16 한컴 3mm 격자 정합 후속

## 1. 결정

**MERGE 수용** — 22 stages 누적 정밀 작업, 자동 검증 + 회귀 가드 충분.

| 항목 | 값 |
|------|-----|
| 번호 | #1120 |
| 작성자 | jangster77 (Taesup Jang) — HWP3 핵심 컨트리뷰터 (15번째 PR) |
| 연결 이슈 | #1116 (closes 누락, 수동 close 권장) |
| 선행 PR (CLOSED, supersede) | #1118 → #1119 → #1120 |
| 처리일 | 2026-05-26 |
| Merge commit | `f7a68bc319ed67ace974722d324e49339954d121` |
| Merge 방식 | 메인테이너 측 로컬 충돌 해결 + push (BEHIND + CONFLICTING → admin merge 불가) |
| 충돌 영역 | `mydocs/orders/20260526.md` (메인테이너 + 컨트리뷰터 양쪽 일지 보존) |

## 2. 검증 결과

### 자동 검증 (통과)

| 항목 | 결과 |
|------|------|
| 충돌 해결 (orders/20260526.md) | ✅ 양쪽 일지 보존 |
| `cargo build --release` | ✅ 통과 (2m 32s) |
| `cargo fmt --all -- --check` | ✅ 위반 0 |
| `cargo clippy --lib --release -- -D warnings` | ✅ warnings 0 (17.04s) |
| `cargo test --release --tests` | ✅ svg_snapshot 8 passed / tab_cross_run 1 passed |
| 컨트리뷰터 자체 검증 (보고) | ✅ cargo test --lib 1387 / issue_1116 13 / issue_1105 14 |

### 시각 검증

- 컨트리뷰터 22 stages 정밀 영역 영역 보고 (sample16 + BCP + k-water + 영문 폰트)
- 한컴 2010/2022 영역 영역 정답지 등급 정합 (메모리 룰 `feedback_pdf_not_authoritative` 영역 영역 내)
- 직접 시각 검증 미수행 — svg_snapshot 회귀 가드 + issue_1116/1105 회귀 가드 영역 영역 일부 확보

## 3. 변경 영역 요약

58 파일, +4963/-887 (14 commits):

| 영역 | 영역 |
|------|------|
| `src/renderer/` 13 파일 (+655/-58) | 핵심 정정 — hwp3_variant_flow_spacing_before, tab leader + 페이지 번호 (TOC), text_measurement, skip_spacing_before_prededuct 가드 |
| `tests/issue_1116.rs` (신규) + `tests/issue_1105.rs` (정정) | 회귀 가드 (13 + 14 테스트) |
| `tests/golden_svg/` 7개 갱신 | textLength/lengthAdjust 영역 영역 ASCII 숫자 영역 영역 폭 고정 |
| `mydocs/working/` 22 stages | 단계별 정밀 보고 |
| `mydocs/plans/` + `mydocs/report/` | 수행/구현/최종 보고서 |
| `mydocs/manual/memory/` 3 신규 | 컨트리뷰터 작업 절차 메모리 룰 |
| `mydocs/feedback/hwp3-sample16-hwp5-analysis.md` | HWP3 분석 |
| `mydocs/manual/codex/docs_and_git_workflow.md` | PR 승인 절차 추가 |
| `mydocs/orders/20260525.md`, `20260526.md` | 작업 일지 |

## 4. 핵심 설계 점검

### 4.1 `hwp3_variant_flow_spacing_before(× 2.0)` 분기

```rust
pub(crate) fn hwp3_variant_flow_spacing_before(base: f64, is_hwp3_variant: bool) -> f64 {
    if is_hwp3_variant { base * 2.0 } else { base }
}
```

- 기본: style resolver 영역 영역 절반 (페이지 수 회귀 가드)
- HWP3 variant: × 2.0 (원래 값 복원, 한컴 3mm 격자 정합)
- **약점**: 매직 넘버 — fragile design 가능성
- **합리화**: case-specific 분기 (메모리 룰 `feedback_hancom_compat_specific_over_general`), 향후 재설계 후속 영역

### 4.2 `skip_spacing_before_prededuct` 플래그

PR #1120 CI 영역 영역 자체 발견한 회귀 영역 영역 정정. 일반 경로 (#643/#1027) vs HWP3-origin 분리.

## 5. 컨트리뷰터 메모리 룰 3개 신규 수용

| 룰 | 영역 |
|----|------|
| `feedback_pr_requires_explicit_approval.md` | PR 생성은 별도 승인 후 진행 |
| `feedback_pr_body_korean_required.md` | PR 본문 한국어 작성 필수 |
| `feedback_pr_ci_before_pr.md` | PR 전 로컬 CI급 검증 필수 |

본 프로젝트 영역 영역 공통 메모리 룰 영역 영역 영구 보존. 합리적 절차 룰.

## 6. 위험·관찰

| 항목 | 등급 | 영역 |
|------|------|------|
| `hwp3_variant_flow_spacing_before` 매직 넘버 (× 2.0) | 중 | fragile design — 향후 재설계 후속 (M100 영역 영역 별도) |
| 광범위 변경 (58 파일) | 중 | 단일 task 22 stages 누적 — 합리적 |
| 이슈 #1116 closes 누락 | 저 | merge 후 수동 close 권장 |
| 이슈 #1116 assignee 비어있음 | 저 | 메모리 룰 `feedback_assign_issue_before_work` 위반 — 향후 정책 강화 |
| 직접 시각 검증 미수행 | 중 | svg_snapshot + issue_1116/1105 회귀 가드 일부 확보 |

## 7. 후속 권장 영역

| 항목 | 우선순위 | 영역 |
|------|---------|------|
| 이슈 #1116 close | 즉시 | 작업지시자 결정 영역 영역 진행 |
| `hwp3_variant_flow_spacing_before` 재설계 | M100 후속 | 매직 넘버 영역 영역 영역 영역 영역 영역 |
| HWP3-origin 영역 영역 다른 fixture 영역 영역 시각 재검증 | M100 후속 | 회귀 가드 확장 |

## 8. 메모리 룰 정합

- ✅ `feedback_hancom_compat_specific_over_general` — HWP3-origin case-specific 분기
- ✅ `feedback_pr_supersede_chain` — #1118 → #1119 → #1120 supersede 패턴
- ✅ `feedback_visual_judgment_authority` — 컨트리뷰터 자체 시각 검증 + 회귀 가드
- ⚠️ `feedback_assign_issue_before_work` — 이슈 #1116 assignee 비어있음 (향후 강화)
- ✅ `feedback_v076_regression_origin` — CI 자체 회귀 검출 → 자체 정정
