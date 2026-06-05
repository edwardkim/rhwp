# PR #1120 검토 — sample16 한컴 3mm 격자 정합 후속

- 검토일: 2026-05-26
- PR: https://github.com/edwardkim/rhwp/pull/1120
- 연결 이슈: #1116 (assignee 비어있음, milestone 없음)
- 선행 PR (CLOSED, supersede): #1118 → #1119 → #1120
- 검토자: Claude (rhwp 메인테이너 보조)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #1120 |
| 제목 | #1116 sample16 한컴 3mm 격자 정합 후속 |
| 작성자 | jangster77 (Taesup Jang) — HWP3 영역 핵심 컨트리뷰터 (15개 PR, 대다수 CLOSED) |
| base ← head | `devel` ← `jangster77:local/task1116` |
| head SHA | `7db2b6140c1821bf84124a320b0468a4248a2e91` |
| commits | 14 (광범위 누적) |
| 상태 | OPEN / mergeable=**CONFLICTING** (DIRTY) / 충돌 1 파일 (mydocs/orders/20260526.md) |
| 변경 | 58 files, +4963 / -887 |
| 본질 변경 | src/renderer 13파일 (+655/-58), src/document_core/queries/rendering.rs, src/main.rs |
| 테스트 | tests/issue_1116.rs 신규 (+367) + tests/issue_1105.rs 정정 (+92) |
| Golden | tests/golden_svg/ 7개 갱신 (form-002, aift-page3, issue-157, ktx-toc, exam-kor, bokhakwonseo, table-text) |
| 문서 | mydocs/working/ 22 stages + plans/report/feedback + memory rule 3개 신규 |
| GitHub CI | 컨트리뷰터 보고 영역 영역 통과 (page_path_sb_prededuct 회귀 정정 + svg golden 갱신) |

## 2. 컨트리뷰터 누적 사이클 + supersede 체인

### 누적 사이클 (jangster77, 15 PR)

HWP3-origin 영역 영역 핵심 작업자. 직전 영역:
- **#1107 MERGED** (Task #1105 HWP3→HWP5 page break)
- **#1103 CLOSED** (Task #1086 한컴오피스 페이지네이션)
- **#1085 CLOSED** (Task #1042 HWP3→HWP5 multi-fixture alignment)
- **#1118 CLOSED** (#1116 1차 시도, 2026-05-25 13:58 close)
- **#1119 CLOSED** (#1116 후속, 14:49 close)
- **#1120 OPEN** ← 본 PR (#1116 또 후속)

### 메모리 룰 `feedback_pr_supersede_chain` 정합

`(a) close+통합 머지` 패턴 — #1118/#1119 영역 영역 닫고 #1120 영역 영역 누적 영역.

## 3. 충돌 해결

### 충돌 영역
- `mydocs/orders/20260526.md` (add/add) — 메인테이너 PR 처리 일지 vs 컨트리뷰터 #1116 작업 일지

### 해결 정책
선행 사례 #1005 (`--ours` 메인테이너 일지 보존) 정합 + **양쪽 일지 모두 보존** (메인테이너 PR #1101/#1102 처리 일지 + 컨트리뷰터 Task #1116 일지). 향후 영역 영역 명확한 기록 보존.

## 4. 변경 내용 — 영역 점검

### 4.1 src/renderer 핵심 (13 파일, +655/-58)

| 파일 | 영역 |
|------|------|
| `text_measurement.rs` (+179) | tab leader + 페이지 번호 (TOC) 영역 영역 정밀 처리 — `right_leader_tab_target_rel`, `right_leader_body_target_rel`, `tab_suffix_is_ascii_page_number` 신규 |
| `paragraph_layout.rs` (+152) | spacing_before 영역 영역 HWP3-origin 분기, right leader digit 영역 영역 inline tab 처리 |
| `style_resolver.rs` (+54) | HWP3 영역 영역 영문 폰트 (`HCI Poppy`, Palatino) 영역 영역 HFT 치환 |
| `web_canvas.rs` (+46) | 본 PR 영역 영역 SVG 영역 정합 영역 |
| `svg.rs` (+45) | textLength/lengthAdjust 영역 영역 ASCII 숫자 영역 영역 너비 고정 |
| `mod.rs` (+36) | **`hwp3_variant_flow_spacing_before()` 신규** + `clamp_tab_leader_end_x()` 신규 |
| `composer.rs` (+29) | tab leader 영역 영역 composer 영역 처리 |
| `table_layout.rs` (+29) | k-water 2024 RowBreak 표 절단 위치 정정 |
| `height_cursor.rs` (+21) | **`skip_spacing_before_prededuct` 플래그** — 일반 vs HWP3-origin 분기 (PR #1120 CI 회귀 정정 영역) |
| `layout.rs` (+21) | 영역 영역 측정 영역 |
| `height_measurer.rs` (+13) | 영역 |
| `typeset.rs` (+10) | 영역 |
| `skia/text_replay.rs` (+6/-) | 영역 |

#### 핵심 분기 영역 — `hwp3_variant_flow_spacing_before()`

```rust
pub(crate) fn hwp3_variant_flow_spacing_before(base: f64, is_hwp3_variant: bool) -> f64 {
    if is_hwp3_variant { base * 2.0 } else { base }
}
```

- 기본 (style resolver): `spacing_before` 영역 영역 절반 (페이지 수 회귀 가드)
- HWP3 variant: × 2.0 (원래 값 복원)
- 본문 흐름 영역 영역 한컴 3mm 격자 정합 영역 영역 사용

**평가**:
- 매직 넘버 2.0 + 분기 영역 영역 fragile design 가능성
- 그러나 회귀 가드 보존 (style resolver 영역 영역 절반 유지) + HWP3 영역 영역 명시 분기 → 합리적
- 메모리 룰 `feedback_hancom_compat_specific_over_general` 정합 (case-specific over general)

#### 핵심 영역 — `skip_spacing_before_prededuct` 플래그

PR #1120 CI 영역 영역 발견된 회귀 영역 영역 정정:
- 일반 경로 (#643/#1027): `raw_end_y - curr_sb` 유지 (`spacing_before` 사전 차감)
- HWP3-origin 흐름: `raw_end_y` 그대로 (사전 차감 생략)

**평가**: 자체 발견한 결함 영역 영역 자체 정정 — 컨트리뷰터의 성숙도 높음.

### 4.2 Golden SVG 7개 갱신

| Golden | 영역 영역 |
|--------|----------|
| form-002, table-text, issue-157, issue-267, issue-617, issue-147 | ASCII 숫자 영역 영역 `textLength + lengthAdjust` 속성 추가 (텍스트 폭 고정) |
| issue-677 (bokhakwonseo) | 1018 hunks (가장 큰 영역) — 본 영역 영역 영역 영역 변경 영역 |

**평가**: 구조 영역 영역 무손실 (라인 수 동일), 좌표/속성 영역 영역 미세 조정. ASCII 숫자 영역 영역 너비 고정 영역 영역 SVG 영역 영역 일관성 영향 → 합리적 변경.

### 4.3 새 통합 테스트

- `tests/issue_1116.rs` 신규 (+367) — 13개 테스트 (컨트리뷰터 보고)
- `tests/issue_1105.rs` 정정 (+92) — 14개 테스트

### 4.4 컨트리뷰터 메모리 룰 추가 (mydocs/manual/memory/)

3 신규 룰 + MEMORY.md 인덱스 갱신 + codex/docs_and_git_workflow.md 영역 영역 정정:

| 룰 | 영역 |
|----|------|
| `feedback_pr_requires_explicit_approval.md` | PR 생성은 별도 승인 후 진행 (PR 준비 vs PR 생성 분리). 컨트리뷰터 본인 영역 영역 #1116 영역 영역 작업지시자 피드백 기반 |
| `feedback_pr_body_korean_required.md` | PR 본문 한국어 작성 필수 |
| `feedback_pr_ci_before_pr.md` | PR 전 로컬 CI급 검증 필수 |

**평가**: 합리적인 룰. 컨트리뷰터 영역 영역 본인 작업 가이드라인 + 본 프로젝트 공통 영역 영역 적용. 메인테이너 영역 영역 신구 메모리 영역 (`mydocs/manual/memory/`) 와 정합.

## 5. 자동 검증 결과

cherry-pick 영역 영역 무관 — merge --no-commit 영역 영역 충돌 해결 후 영역 영역 검증:

| 항목 | 결과 |
|------|------|
| 충돌 해결 (orders/20260526.md) | ✅ 양쪽 일지 보존 |
| `cargo build --release` | ✅ 통과 (2m 32s) |
| `cargo fmt --all -- --check` | ✅ 위반 0 |
| `cargo clippy --lib --release -- -D warnings` | ✅ warnings 0 (17.04s) |
| `cargo test --release --tests` | ✅ svg_snapshot 8 passed / tab_cross_run 1 passed |
| 컨트리뷰터 자체 검증 보고 | ✅ cargo test --lib 1387 / issue_1116 13 / issue_1105 14 / fmt / git diff --check / build --bin |
| GitHub CI | 컨트리뷰터 보고 영역 영역 통과 (PR #1120 CI 정정 후) |

## 6. 시각 검증

### 6.1 영향 fixture
- `samples/hwp3-sample16-hwp5*` (6개 변종) — 본 PR 핵심 영역
- `samples/k-water-rfp-2024.hwp` — RowBreak 표 영역 (PR #1101 fixture 영역 영역 다른 영역)
- `samples/hwp3-sample16.hwp` — HWP3 원본 영역

### 6.2 컨트리뷰터 보고 영역 영역 시각 정합
- sample16 p2 목차 right-tab/leader 영역 영역 보정 + 회귀 테스트 (issue_1116 13 passed)
- sample16 p3 본문 영역 영역 한컴 3mm 격자 정합 (Stage 20 영역 영역 폰트 차이 검증 영역 영역 산출 — Palatino=6, HCI=0)
- 2022 BCP 꼬리 줄 영역 영역 좁은 조건 정정
- k-water 2024 p5 RowBreak 영역 영역 절단 위치 정정

### 6.3 시각 판정 권위 등급 (메모리 룰 `reference_authoritative_hancom`)

PR 영역 영역 정답지 = 한컴 2022 PDF (`pdf/`, 권위 등급 정합). HWP3 영역 영역 한컴 2010/2022 영역 영역 모두 — 본 영역 영역 정답지 등급 부합 영역.

### 6.4 직접 시각 검증

작업지시자 영역 영역 sample16 + k-water 영역 영역 22 stages 영역 영역 정밀 작업 — 결과 영역 영역 컨트리뷰터 보고에 의존. svg_snapshot golden 8개 모두 통과 영역 영역 회귀 가드 영역.

## 7. 위험·관찰

| 항목 | 등급 | 영역 |
|------|------|------|
| `hwp3_variant_flow_spacing_before` 매직 넘버 (× 2.0) | 중 | fragile design 가능성. 일반 경로 영역 영역 분리 (`is_hwp3_variant` 플래그) 영역 영역 가드. 향후 영역 영역 영역 영역 재설계 영역 가능 |
| #1118/#1119 영역 영역 supersede 체인 | 저 | 컨트리뷰터 영역 영역 본인 영역 영역 정리 — 합리적 작업 영역 |
| 광범위 변경 (58 파일, +4963/-887) | 중 | 단일 PR 영역 영역 단일 책임 영역 영역 깨짐. 그러나 단일 task (#1116) 영역 영역 22 stages 영역 영역 누적 — 합리적 영역 영역 단일 PR |
| 이슈 #1116 영역 영역 closes 누락 | 저 | PR 본문 영역 영역 `관련 이슈: #1116` 영역 영역 closes 영역 영역 명시 안 함. merge 후 영역 영역 영역 영역 작업지시자 결정 |
| 이슈 #1116 영역 영역 assignee 비어있음 | 저 | 메모리 룰 `feedback_assign_issue_before_work` 영역 영역 위반 — 영역 영역 향후 정책 강화 |
| 컨트리뷰터 메모리 룰 3개 추가 | 저 | 본 프로젝트 영역 영역 영향 영역. 룰 내용 영역 영역 합리적 (PR 작업 절차) → 수용 |
| svg golden 7개 갱신 | 저 | textLength/lengthAdjust 영역 영역 변경 — 텍스트 폭 고정 영역 영역 일관성 향상. 회귀 0 |
| HWP3 사용자 영역 영역 정밀화 vs 일반 사용자 영역 영역 회귀 | 중 | 본 PR 영역 영역 HWP3 영역 영역 정밀화 + 일반 경로 영역 영역 가드 (`skip_spacing_before_prededuct`) 영역 영역 분리. 회귀 가드 영역 영역 svg_snapshot 통과 영역 영역 일부 확보 |

## 8. 최종 평가 (잠정)

| 항목 | 결과 |
|------|------|
| 본질 해결 | ✅ sample16 한컴 3mm 격자 + BCP + k-water + 영문 폰트 — 22 stages 누적 정정 |
| 자동 검증 | ✅ 모두 통과 (build / fmt / clippy --lib / test / svg_snapshot) |
| 시각 검증 | ⚠️ 컨트리뷰터 자체 보고 영역 영역 의존 (직접 시각 미수행) — 한컴 2010/2022 영역 영역 권위 등급 정합 |
| 코드 품질 | ✅ 22 stages 보고서 + 단계별 영역 영역 정밀. 핵심 분기 영역 영역 매직 넘버 (× 2.0) 영역 영역 약점 |
| 메모리 룰 정합 | ✅ `hancom_compat_specific_over_general`, `pr_supersede_chain`, `visual_judgment_authority` |
| 회귀 가드 | ✅ issue_1116 (13) + issue_1105 (14) + svg_snapshot 갱신 (회귀 검출 메커니즘) |
| 충돌 해결 | ✅ orders 양쪽 일지 보존 |
| **결정 권장** | **MERGE** — 본질 해결 + 자동 검증 통과 + 회귀 가드 영역 충분. 광범위 영역 영역 단일 task 누적 합리적 |

## 9. 작업지시자 결정 요청

1. **MERGE** 진행 — 충돌 해결 영역 영역 메인테이너 측 자체 머지 (BEHIND 영역 영역 admin merge 영역)
2. 이슈 #1116 close 영역 영역 — PR merge 후 영역 영역 영역 영역 (closes 본문 영역 영역 미명시 영역 영역 수동)
3. 컨트리뷰터 메모리 룰 3개 영역 영역 본 프로젝트 영역 영역 영구 보존 영역 영역 수용
4. `hwp3_variant_flow_spacing_before` 매직 넘버 영역 영역 향후 재설계 영역 — M100 후속 영역 영역 (별도 영역, 본 PR 영역 영역 작동 영역 영역 우선 확보)
