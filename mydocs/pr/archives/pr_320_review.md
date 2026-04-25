# PR #320 검토 — Task #318 회귀 수정 + 분리 PR 통합

## PR 정보

- **PR**: [#320](https://github.com/edwardkim/rhwp/pull/320)
- **이슈**: #318 (z-table 회귀 + wrap=Square 호스트 중복) + 부가로 #313, #314, #317
- **작성자**: @planet6897
- **base/head**: `devel` ← `task318`
- **Mergeable**: ⚠️ 원래 CONFLICTING (orders 1건만 충돌, 코드 자동 merge 성공)
- **CI**: ✅ 전부 SUCCESS

## 변경 요약

본 PR 은 **Epic #309 의 4개 sub-issue 통합** PR. PR #316 의 z-table 회귀 (issue_301 FAIL) 을 수정하여 정식 통과시키는 것이 핵심.

### 포함 sub-issue

| Task | 내용 | 비고 |
|------|------|------|
| #311 | vpos-reset 강제 분리 가설 부정 | **PR #315 에서 이미 머지됨** |
| #312 | column 정확도 조사 / TypesetEngine 발견 | **PR #315 에서 이미 머지됨** |
| **#313** | TypesetEngine main 전환 | 이 PR 핵심 (1) |
| **#314** | HWPX 어댑터 normalize | 이 PR 핵심 (2) |
| **#317** | 어댑터 +1쪽 잔존 origin 보강 | 이 PR 핵심 (3) |
| **#318** | 분할 표 + wrap=Square 호스트 인라인 수식 중복 emit 회귀 수정 | 이 PR 핵심 (4) |

PR #315 가 이미 머지된 지금, **devel merge 시 #311/#312 부분은 자동으로 사라지고 #313/#314/#317/#318 만 추가됨**. 자동 merge 성공.

## 4개 핵심 변경

### Task #313 — TypesetEngine main 전환

**`src/document_core/queries/rendering.rs::paginate()`** — Paginator → TypesetEngine default 전환:
- `RHWP_USE_PAGINATOR=1` env로 fallback 가능
- `tests/hwpx_to_hwp_adapter.rs` 회귀 3건 `#[ignore]` (어댑터 측 사안, Task #317 에서 회수)
- `tests/golden_svg/issue-147/aift-page3.svg` 페이지 번호 마커 갱신

### Task #314 — HWPX 어댑터 normalize (부분 완료)

**`src/document_core/commands/document.rs::normalize_hwpx_paragraphs`** 함수 추가:
- HWPX 로드 후 빈 char_shapes 에 default `[(0, 0)]` 추가
- control_mask 를 controls 기반 재계산
- 셀 paragraphs 재귀 처리
- char_shapes_len 59건 + control_mask 27건 normalize 해소

### Task #317 — 어댑터 +1쪽 잔존 origin 보강

**`src/document_core/converters/hwpx_to_hwp.rs::adapt_table`** — typeset 의 `is_tac` 판정(`table.attr & 0x01`) 비대칭 보강:
- 어댑터 raw_ctrl_data 합성 시 attr 영역(offset 0..4) 0 강제
- `table.attr=0` 보존
- DIRECT 와 동일한 block 분기 진입
- `tests/hwpx_to_hwp_adapter.rs` `#[ignore]` 3건 제거 → **재활성화 25 / 0 / 0**

### Task #318 — issue_301 회귀 수정 (가장 중요)

**두 origin 보강**:

1. **`src/renderer/layout/table_partial.rs:766` `Control::Equation`** — `#301` 의 `already_rendered_inline` 가드 적용
   - 영향: z-table 셀 0.1915/0.3413/0.4332 각 2회 → 1회

2. **`src/renderer/layout.rs::layout_column_item` `PartialParagraph` 분기** — `is_wrap_host` 가드 추가 (FullParagraph 동일 패턴)
   - 영향: 0.4772 body 위치 1회 → 2회 (z-table 1 합쳐 3 → 2)

**`tests/issue_301.rs::z_table_equations_rendered_once` `#[ignore]` 제거** — 정식 통과.

## 메인테이너 검증 결과

### PR 브랜치 + devel merge 후

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **992 passed / 0 failed / 1 ignored** |
| `cargo test --test svg_snapshot` | ✅ 6 passed (golden 유지) |
| `cargo test --test issue_301` | ✅ **1 passed (z-table 회귀 해소!)** |
| `cargo test --test hwpx_to_hwp_adapter` | ✅ **25 passed / 0 failed / 0 ignored** (#313 격리분 회수) |
| `cargo test --test tab_cross_run` | ✅ 1 passed |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32` | ✅ clean |

### 4샘플 페이지 수 (Epic #309 핵심 목표 달성)

| 샘플 | Before (Paginator) | After (TypesetEngine) | PDF |
|------|---------------------|------------------------|-----|
| **21_언어** | 19 | **15** | 15 ✅ |
| exam_math | 20 | 20 | 20 ✅ |
| exam_kor | 25 | 24 | (미보유) |
| exam_eng | 11 | 9 | (미보유) |

### Task #291 (어제 메인테이너 핀셋) 호환성

KTX.hwp 표 좌측 x:
- pi=31: x=**518.16** ✅ (Task #291 수정 유지)
- pi=32: x=**517.95** ✅ (Task #291 수정 유지)

→ 본 PR 의 변경이 Task #291 수정과 충돌 없음. 양쪽 효과 모두 보전.

### CI (원본 브랜치)

| Check | 결과 |
|-------|------|
| CI / Build & Test | ✅ SUCCESS |
| CodeQL rust/js/python | ✅ 전부 SUCCESS |

## 주의 사항 — LAYOUT_OVERFLOW 메시지

KTX.hwp 빌드 시:
```
LAYOUT_OVERFLOW: page=0, col=1, para=32, type=Table, y=765.5, bottom=755.9, overflow=9.6px
```

pi=32 표가 단 1 영역 끝까지 길고, Task #291 의 우측 정렬로 표 우측이 단 우측 경계에 가까워졌기 때문. 시각적 깨짐 없음 (작업지시자 어제 검증 시 이미 확인됨).

## 충돌 분석

- **충돌 파일**: `mydocs/orders/20260425.md` 단 1개 (문서)
- **코드 충돌**: 없음 (`layout.rs`, `table_partial.rs` 자동 merge)
- **원인**: PR 본문에 Task #313/#314/#317/#318 섹션이 있고 devel 에는 Task #291 섹션이 있음
- **해결**: 모두 포함하는 형태로 메인테이너 직접 해결

## 평가 포인트

1. **#316 의 z-table 회귀 정식 수정** — `tests/issue_301.rs` `#[ignore]` 제거 후 통과
2. **어댑터 격리 테스트 회수** — `hwpx_to_hwp_adapter` 3건 `#[ignore]` 제거 후 재활성화 (25/0/0)
3. **Epic #309 핵심 목표 달성** — 21_언어 15쪽 PDF 정확 일치
4. **두 origin 명확 식별** — table_partial.rs (분할 표) + layout.rs (PartialParagraph 호스트)
5. **#301 의 가드 패턴 일관 적용** — `already_rendered_inline`, `is_wrap_host`
6. **CLAUDE.md 절차 완전 준수** — 두 task (#313/#314, #317/#318) 모두 계획서/보고서 완비

## 후속 사안 (작성자 명시)

> TypesetEngine 이 wrap=Square 호스트 paragraph 에 대해 PartialParagraph 를 emit 하는 동작이 정상인지 재평가. 현재 layout 측 가드로 회피하지만, 의도가 명확하면 PartialParagraph 자체를 emit 하지 않는 것이 더 깔끔.

## 리스크 평가

| 리스크 | 판정 |
|--------|------|
| 기존 골든 SVG 회귀 | ✅ svg_snapshot 6 passed |
| 어댑터 회귀 | ✅ 25/0/0 (격리 3건 회수) |
| Task #291 (TAC 표 align) 충돌 | ✅ 자동 merge 성공, 효과 보전 |
| LAYOUT_OVERFLOW 9.6px (KTX.hwp) | ⚠️ 시각적 영향 없음 (어제 작업지시자 검증) |
| `is_wrap_host` 가드의 부작용 | ✅ FullParagraph 와 동일 패턴, 동작 일관성 |

## 판정

✅ **Merge 권장**

**사유:**
1. **#316 의 z-table 회귀 정식 수정** — issue_301 ignore 제거 후 통과
2. **어댑터 격리 회수** — 3 ignore 해소
3. **Epic #309 핵심 목표 달성** — 21_언어 PDF 정확 일치
4. **CI + 로컬 검증 모두 통과** — 992 lib + 25 어댑터 + 6 svg_snapshot + 1 issue_301 + 1 tab_cross_run
5. **Task #291 (어제 핀셋) 효과 보전** — KTX.hwp 표 좌표 동일 유지
6. CLAUDE.md 절차 완전 준수

**Merge 전략:**
- orders 문서 충돌 메인테이너 직접 해결 완료
- `planet6897/task318` 에 push 후 admin merge

**WASM 시각 검증 권장 (선택)** — TypesetEngine 전환은 큰 변경이라 작업지시자 직접 확인 시 안심.

### 메인테이너 WASM 브라우저 검증 (2026-04-25)

- Docker로 WASM 재빌드: `pkg/rhwp_bg.wasm` 갱신 (13:17)
- 작업지시자 직접 시각 확인:
  - **개선 확인** — 21_언어 15쪽 PDF 정확 일치, KTX.hwp Task #291 효과 유지, issue_301 z-table 정상
  - **주요 회귀 테스트 시각 확인 통과**
- 작업지시자 최종 판정: **검증 성공**
