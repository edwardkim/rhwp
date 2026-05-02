# PR #510 검토 문서

**PR**: [#510 Task #508: PageLayerTree image brightness/contrast JSON 필드 추가](https://github.com/edwardkim/rhwp/pull/510)
**작성자**: @postmelee (Taegyu Lee) — alhangeul-macos downstream 관점의 외부 컨트리뷰터
**Base / Head**: `devel` ← `postmelee:task508` (Fork 기반)
**Linked Issue**: [#508](https://github.com/edwardkim/rhwp/issues/508) (OPEN, milestone 미지정, labels 없음, assignees 없음)
**상태**: OPEN, MERGEABLE, mergeStateStatus = **BEHIND**
**CI**: ALL SUCCESS (Build & Test + CodeQL × 3 + Canvas visual diff; WASM Build SKIPPED)
**작성일**: 2026-05-01
**검토일**: 2026-05-02

---

## 1. 개요

### 1.1 본질

`PageLayerTree` JSON 의 `PaintOp::Image` serialization 에 `brightness`, `contrast` 두 필드 추가. downstream native renderer (alhangeul-macos) 가 image paint op 만으로 core SVG renderer 와 동일한 image filter 입력값을 재현할 수 있도록 contract 보강.

### 1.2 누락 경로

`ImageNode` 와 `PageRenderTree` 생성 경로에는 `brightness`/`contrast` 가 이미 존재. 그러나 `src/paint/json.rs:289` 의 `PaintOp::Image` JSON 출력에서 `effect` 만 출력하고 두 필드를 누락 → `PageLayerTree` JSON 기반 renderer 가 이미지 보정 값을 알 수 없음.

### 1.3 정정

`effect` 출력 직후 `brightness`, `contrast` 를 항상 출력 (기본값 `0` 도 생략 없이). 4 줄 추가.

```rust
",\"effect\":{},\"brightness\":{},\"contrast\":{}"
```

기본값 출력 정책 — downstream renderer 가 필드 존재 여부 분기 없이 replay 가능.

---

## 2. 변경 정합

| 파일 | 변경 | 비고 |
|------|------|------|
| `src/paint/json.rs` | +6 / -2 | `PaintOp::Image` serialization 4 줄 추가 + test assertion 4 줄 추가 |
| `mydocs/plans/task_m100_508.md` (신규) | +125 | 수행 계획서 |
| `mydocs/plans/task_m100_508_impl.md` (신규) | +178 | 구현 계획서 |
| `mydocs/working/task_m100_508_stage{1-3}.md` (신규) | (3 파일) | 단계별 보고서 |
| `mydocs/report/task_m100_508_report.md` (신규) | — | 최종 보고서 |
| `mydocs/orders/20260501.md` | +6 / 0 | M100 #508 항목 추가 |

**소스**: 1 파일 (+6 / -2) — 매우 작은 변경.
**테스트**: 기존 `serializes_backend_replay_payload_fields` 테스트에 non-zero assertion 추가 (신규 파일 없음).

---

## 3. 검토 항목

### 3.1 코드 품질

- ✅ **변경 영역 본질** — `PaintOp::Image` 단일 분기. 다른 paint op 영향 0.
- ✅ **출력 위치** — `effect` 직후, `transform` 직전. 기존 JSON 구조와 정합.
- ✅ **포맷팅** — 기존 `write!` 호출에 두 필드 추가 (별도 `write!` 분리 안 함). 코드 간결.
- ✅ **기본값 항상 출력 정책** — `brightness=0`, `contrast=0` 도 생략 없이 출력. PR 본문/구현 계획서에 근거 명시 (downstream 분기 단순화).

### 3.2 테스트

- ✅ **non-zero assertion** — 기존 `serializes_backend_replay_payload_fields` 에 `image.brightness = -50; image.contrast = 70;` 설정 + JSON contains assertion 2건 추가.
- ✅ **회귀 테스트 적합** — 추가 fixture 불필요, 기존 통합 테스트 재사용.
- 경미: 기본값 (`0`) 출력 검증은 별도 assertion 없음. 본 변경의 정책 (항상 출력) 을 직접 검증하지는 않음. 본질 외, 해당 사항 아님.

### 3.3 schemaVersion 판단

- ✅ **`PAGE_LAYER_TREE_SCHEMA_VERSION` 유지** — additive change. 기존 필드 제거/의미 변경 없음. PR 본문/구현 계획서에 근거 명시.
- ✅ **downstream strict schema validator 위험 인지** — 구현 계획서 §위험 영역에 명시. 작업지시자가 bump 필요시 별도 결정.

### 3.4 검증 게이트 점검

| 게이트 | PR 본문 보고 | 검토 노트 |
|--------|------------|----------|
| cargo test --lib paint::json::tests::serializes_backend_replay_payload_fields | 통과 | 정합 |
| cargo test --lib paint::json | 4 passed, 0 failed | 정합 |
| cargo test --lib | 1102 passed, 0 failed, 1 ignored | 정합 |
| cargo clippy --lib -- -D warnings | 통과 | 정합 |
| cargo clippy -- -D warnings | 통과 | 정합 |
| cargo test (전체) | lib + integration + doc-tests 통과 | 정합 |
| GitHub CI | Build & Test + CodeQL × 3 + Canvas visual diff 통과 | 정합 |

### 3.5 외부 영역 정합 (PR #506 / #507 / Task #509)

- ✅ **회귀 위험 0** — 변경은 `src/paint/json.rs` 의 `PaintOp::Image` serialization 한 줄. 다른 영역 충돌 0.
- ✅ **PR #507 (수식 영역) 와 무관** — 수식 / equation 영역과 별개.
- ✅ **PR #506 (HWP3 파서) 와 무관** — HWP3 별도 파서.
- ✅ **Task #509 (PUA 글머리표) 와 무관** — paragraph_layout 별개 영역.

### 3.6 시각 검증 fixture 점검

이슈 #508 본문에 언급된 대표 샘플 `samples/복학원서.hwp` 의 워터마크 이미지 (`effect=GrayScale`, `brightness=-50`, `contrast=70`) 가 정합한 시각 검증 fixture.

- ✅ **`samples/복학원서.hwp`** — 존재 (114 KB)
- ✅ **`samples/복학원서.pdf`** — 존재 (한컴 출력 PDF, 199 KB)
- ✅ **메인테이너 환경에서 직접 시각 판정 가능** — `rhwp export-svg samples/복학원서.hwp` 후 PageLayerTree JSON 출력에 `brightness`/`contrast` 포함 확인 가능

### 3.7 외부 컨트리뷰터 첫 PR / 워크플로우 점검

- ✅ 작성자 `postmelee` 의 외부 PR.
- ✅ Fork 기반 워크플로우 정합 (`postmelee:task508` → `edwardkim/rhwp:devel`).
- ✅ 내부 워크플로우 정합 — 수행 계획서 / 구현 계획서 / 단계별 보고서 (3 stage) / 최종 보고서 작성. PR 본문에도 외부 기여자 체크리스트 + 산출물 명시.
- ✅ `mydocs/pr/` 폴더는 수정하지 않음 (외부 기여자가 내부 PR 검토 폴더를 건드리지 않는 정합).
- ✅ `AGENTS.md` 미추적 파일을 PR 범위에서 제외.
- ✅ **alhangeul-macos downstream 관점** — 메모리 `project_alhangeul_ios` 의 iPad 학습 도구 프로젝트와 별개 (alhangeul-macos 는 macOS native renderer). downstream 의 backend replay contract 보강 요청.

### 3.8 이슈 #508 메타데이터 점검

- ⚠️ Issue assignees 없음 → 메모리 `feedback_assign_issue_before_work` 적용. 메인테이너 assign 권장 (이미 PR 제출됐으므로 일차 방어선은 무의미하나 정책 일관성).
- ⚠️ Issue milestone 미지정 → PR 본문에서 "M100 — v1.0.0 조판 엔진" 으로 분류한다고 명시. issue 에도 v1.0.0 milestone 추가 권장.
- ⚠️ Issue labels 없음 → `enhancement` 라벨 추가 권장.

---

## 4. 위험 정리

| 위험 | 가능성 | 비고 |
|------|--------|------|
| `brightness`/`contrast` 항상 출력 정책으로 기존 PageLayerTree JSON consumer (strict schema) 가 schema mismatch 처리 | 🟨 작음 | additive change. PR 본문/구현 계획서에 위험 인지. 현재 알려진 strict consumer 없음. |
| 기본값 (`0`) 출력으로 JSON 크기 증가 | 🟢 매우 작음 | image paint op 당 약 30 byte. 무시 가능. |
| schemaVersion 유지 결정이 downstream strict version guard 와 충돌 | 🟢 매우 작음 | additive change 정합. 필요 시 작업지시자 별도 결정. |
| 시각 회귀 (이미지 출력 자체 변화) | 🟢 0 | core SVG renderer / Canvas / WebCanvas 출력 변경 없음 (JSON contract 한정). |

**중요 차이 (PR #507 와 비교):** PR #510 은 **데이터 contract 보강** 이고 시각 결함 정정이 아님. core SVG / Canvas 출력은 변경 없음이 전제. 다만 메모리 `feedback_visual_regression_grows` / `reference_authoritative_hancom` 정합으로 **작업지시자 시각 판정을 필수 게이트로 적용** 한다 (한컴 2010 + 2020 으로 `samples/복학원서.hwp` 출력 ↔ cherry-pick 후 SVG 비교). 회귀 부재 보장 + 의도되지 않은 부수 효과 점검.

---

## 5. 결정

**권장**: ✅ **머지 (cherry-pick)** — 코드 변경이 작고 contract additive 보강. 회귀 위험 0.

**근거:**
1. 변경 영역 작음 (소스 1 파일, +6/-2).
2. 기존 통합 테스트 재사용 + non-zero assertion 추가로 회귀 방어.
3. CI 전체 통과 (Build & Test + CodeQL × 3 + Canvas visual diff).
4. `samples/복학원서.hwp` + PDF 존재 → 메인테이너가 PageLayerTree JSON 출력 직접 점검 가능.
5. schemaVersion 판단 근거 명시 (additive, 기본값 동일 의미).
6. 외부 컨트리뷰터 (postmelee) 의 워크플로우 정합 — 수행 / 구현 / 단계별 / 최종 보고 모두 정합.

**머지 절차:**
1. `git fetch origin pull/510/head:local/pr510`
2. `git checkout local/devel && git checkout -b local/pr510-cherry`
3. cherry-pick — 소스 1 commit + 문서 commit 분리 또는 일괄
4. 검증 게이트: cargo test --lib + paint::json + cargo clippy + svg_snapshot 6/6 (회귀 0)
5. 시각 점검 (선택): `rhwp export-svg samples/복학원서.hwp` 출력 + PageLayerTree JSON 의 `brightness=-50`, `contrast=70` 확인
6. local/devel 머지 → devel push
7. PR #510 close (cherry-pick 정합) + 이슈 #508 close (`closes #508` commit 메시지)

**머지 시 추가 정합 사항:**
- 이슈 #508 milestone v1.0.0 추가
- 이슈 #508 enhancement 라벨 추가
- 이슈 #508 assignee 지정 (메인테이너)
- README 기여자 목록 갱신 (postmelee)

---

## 6. PR 본문 산출물 점검

PR 본문 보고 산출물:
- 수행 계획서: `mydocs/plans/task_m100_508.md`
- 구현 계획서: `mydocs/plans/task_m100_508_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_508_stage{1-3}.md` (3 파일)
- 최종 보고서: `mydocs/report/task_m100_508_report.md`
- 오늘할일 갱신: `mydocs/orders/20260501.md`

✅ 외부 컨트리뷰터로서 내부 워크플로우 정합. 단계 분리도 적절 (3 stage — Stage 1 진단, Stage 2 구현 + Stage 3 테스트).

---

## 7. 메모리 정합

- `feedback_check_open_prs_first` — 본 PR 처리 정합 (이슈 #508 → PR #510 연결 확인)
- `feedback_pr_comment_tone` — 차분하고 사실 중심 댓글
- `feedback_hancom_compat_specific_over_general` — 본 PR 의 변경은 contract additive (한컴 호환과 무관)
- `feedback_release_sync_check` — cherry-pick 시점 git pull --ff-only origin main 점검
- `feedback_assign_issue_before_work` — 이슈 #508 assignees 없음 점검 필요
- `feedback_visual_regression_grows` — **본 PR 은 데이터 contract 보강이므로 시각 회귀 게이트 미적용** (PR #507 과 차이)

---

## 8. PR #507 와의 비교

| 항목 | PR #507 | PR #510 |
|------|---------|---------|
| 본질 | 시각 결함 정정 (squashing 해소) | 데이터 contract 보강 (JSON 필드 추가) |
| core SVG 출력 변화 | O (`12r²` → `(1/2) x²`) | X (전제) |
| 시각 판정 게이트 | ⚠️ 필수 (메인테이너 한컴 2010+2020 직접 판정) | ⚠️ 필수 (회귀 부재 보장 + 부수 효과 점검) |
| 시각 fixture 의존성 | ⚠️ 미적분03.hwp 미존재 → 수정 요청 | ✅ 복학원서.hwp + 한컴 PDF 존재 |
| 결정 | 수정 요청 | **머지 권장 (작업지시자 시각 판정 게이트 통과 후)** |

---

## 9. 다음 단계

작업지시자 승인 후:

1. `pr_510_review_impl.md` 작성 (cherry-pick 절차 상세) → 승인 요청
2. cherry-pick + 검증 게이트 통과
3. local/devel 머지 + devel push
4. PR #510 close + 이슈 #508 close (closes #508)
5. `pr_510_report.md` 작성 (merge 결정 + 사유)
