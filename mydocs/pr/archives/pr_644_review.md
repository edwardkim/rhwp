---
PR: #644
제목: Task #643: 페이지 분할 드리프트 정정 (5축 정합) — closes #643
컨트리뷰터: @planet6897 (Jaeuk Ryu)
사이클: 12번째
base: devel (BEHIND 2 commits)
mergeable: MERGEABLE
CI: ALL SUCCESS
변경: +639/-237, 11 files
---

# PR #644 1차 검토 보고서

## 1. 메타 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | #644 |
| 제목 | Task #643: 페이지 분할 드리프트 정정 (5축 정합) |
| 컨트리뷰터 | @planet6897 (12번째 사이클) |
| base / head | devel / task643-pagination-drift-fix |
| mergeStateStatus | BEHIND (devel 2 commits 앞) |
| mergeable | MERGEABLE |
| CI | Build & Test / CodeQL(JS-TS, Python, Rust) / Canvas visual diff — ALL SUCCESS |
| 변경 규모 | +639 / -237, 11 files |
| 커밋 수 | 5 (Stage 0/1, Stage 2-4, Stage 5, 후속, rebase) |
| closes | #643 |

## 2. Issue #643 본질

`samples/2022년 국립국어원 업무계획.hwp` 두 케이스에서 누적 드리프트로 부당 페이지 분리:

- **케이스 1 (page 6)**: pi=80 (' 및 점자 해당 분야 전문인력 확보 어려움') 마지막 줄이 page 6 → 7 부당 분리
- **케이스 2 (page 3)**: pi=39 ('국어사전 정보보완심의회 운영을 통한 국어사전 정보 수정 및 보완') 가 page 3 → 4 부당 분리

devel 환경 측정: `2022년 국립국어원 업무계획.hwp` 페이지 수 **40** (HWP/PDF 원본 35 페이지 대비 5 페이지 과다).

## 3. PR 의 수정 — 5축 누적 드리프트 분해

### 통찰 (PR 본문 인용)
- **HWP**: `vpos_(N+1) - vpos_N = lh_total + ls_total + sa_N + sb_(N+1)`
- **Layout**: y_advance per pi = `sb_N + lh_total + ls_total`
- → `sb_N ≠ sb_(N+1)` (예: 빈 문단 sb=0 인접) 시 차이 누적 → LAYOUT_OVERFLOW

### 5 축 정정

| 축 | 위치 | 정정 |
|----|------|------|
| 1 | `pagination/engine.rs:846-852` fit 루프 | 마지막 줄은 line_height 만 (트레일링 ls 제외) |
| 2 | `typeset.rs:907-914` LAYOUT_DRIFT_SAFETY_PX | 10 → 4px (축 1 정정 후 보수적 마진 축소) |
| 3 | `layout.rs:1521` VPOS_CORR backward | 1.0 → 8.0px (trailing ls 영역 내 안전 백워드) |
| 4 | `layout.rs:1504` VPOS_CORR end_y | sb_N 사전 차감 (layout 의 sb 추가와 정합) |
| 5 | `typeset.rs:566-606` Task #404 vpos_end | `para.line_segs.last().vpos + line_height` 직접 사용 (트레일링 ls 자연 제외) |

## 4. 본 환경 cherry-pick simulation

### 4.1 깨끗한 적용
- `local/pr644-sim` 브랜치 생성, 5 commits cherry-pick
- **충돌 0건** (devel 의 ff47aefa Task #634 + 1185eb98 PR close 영역과 본 PR 코드 영역 비충돌)

### 4.2 결정적 검증 (전체 1221 테스트)
- `cargo test --release` → **ALL PASS**, failed 0건
- `tests/issue_643.rs` (신규): `page6_pi80_last_line_stays_on_page6` ✅
- `tests/issue_554.rs`:
  - `task554_no_regression_2022_kuglip` ✅
  - `task554_no_regression_aift` ✅
  - `task554_no_regression_exam_kor` ✅
  - `task554_no_regression_2025_donations_hwpx` ✅
  - `task554_no_regression_exam_science` ✅
  - 12 / 12 ✅
- `tests/svg_snapshot.rs`: `issue_147_aift_page3` ✅ (golden SVG 갱신 적용)

### 4.3 페이지 수 변화 확증
| 환경 | `2022년 국립국어원 업무계획.hwp` 페이지 수 |
|------|-----|
| devel (현재) | 40 |
| local/pr644-sim | **35** |

→ HWP/PDF 원본 정합 회복 (5 페이지 감소, 의도한 정정).

## 5. 검토 관점 — 본질 정합

### 5.1 통찰의 타당성

PR 본문의 **`sb_N ≠ sb_(N+1)` 누적 드리프트** 분해는 본 프로젝트 layout/typeset 산식의 정확한 진단입니다:
- HWP IR 의 `LineSeg.vertical_pos` 는 `sb_(N+1)` 을 미리 포함 (다음 문단 line 0 의 vpos 가 곧 그 시작점)
- 본 프로젝트 `LayoutEngine` 은 `paragraph_layout` 시작 시 `sb_N` 을 다시 추가 → 순방향 추가 + HWP 인코딩 차이로 누적

### 5.2 Stage 별 본질

| Stage | 본질 등급 | 비고 |
|-------|----------|------|
| Stage 0/1 (계획서 + RED 테스트) | ★★★ | TDD RED 영역 정확 캡처 |
| Stage 2-4 (4축 정정) | ★★★ | 본질 정정 — 축 1 이 핵심, 나머지는 정합 보강 |
| Stage 5 (보고서) | ★★★ | 산출물 정확 |
| 후속 (Task #404 vpos_end) | ★★★ | 축 5 — heading-orphan 가드 vpos_end 도 동일 산식 적용 |
| 후속 (CI rebase 회귀) | ★★ | golden SVG 정합 (재계산 결과) |

### 5.3 잔존 LAYOUT_OVERFLOW 진단 메시지 (~9px)

PR 본문 명시:
> LAYOUT_OVERFLOW 진단 메시지 ~9px 잔존 (paragraph y_out 기준, visible content fits). `record_overflow` 는 진단 기록만 수행, 렌더링 액션 없음.

→ **렌더링 부작용 없음**, 진단 임계값만 잔존. 추후 `LAYOUT_OVERFLOW` 임계 정책 통합 시 재검토 권장 (별도 task scope).

## 6. 메모리 룰 관점

### `feedback_visual_regression_grows`
> 페이지 총 수 byte 비교만으로는 셀 안 그림 클램프 같은 시각 결함 검출 불가. 작업지시자 시각 판정이 절차의 핵심 게이트

→ 본 PR 은 페이지 수 변화 (40 → 35) 외에도 **시각 회귀 위험**이 있는 영역 (typeset/layout/pagination 3 모듈 동시 정정). 작업지시자 시각 판정 게이트 필수.

### `feedback_hancom_compat_specific_over_general`
> 한컴 호환은 일반화보다 케이스별 명시 가드

→ 본 PR 은 **5 축 동시 정합** (일반화 알고리즘) 으로 다른 fixture 회귀 위험. 그러나 `task554` 광범위 sweep 12 케이스 ALL PASS 로 회귀 미검출.

### `feedback_pdf_not_authoritative` (갱신 후)
> 한컴 2020 PDF 는 정답지 역할 가능 (Linux/macOS 컨트리뷰터 환경)

→ 컨트리뷰터의 PDF 기준 (35 페이지) 정합. 단, 작업지시자 한컴 2010/2022 편집기 출력으로 최종 판정 필요.

### `feedback_v076_regression_origin`
> 외부 PR 컨트리뷰터들이 자기 환경 PDF 를 정답지로 사용 → 작업지시자 환경에서 회귀

→ **본 PR 의 핵심 위험점**. 컨트리뷰터가 PDF 35 페이지 기준 5 축 정합 했으나, 작업지시자 한컴 편집기 환경에서 다른 fixture 회귀 가능성.

## 7. 결정 옵션

| 옵션 | 내용 | 비고 |
|------|------|------|
| **A** | 5 commits 그대로 squash merge + WASM 빌드 + 작업지시자 시각 판정 | 본질 정정 + 회귀 0건 + CI 통과. 시각 게이트 통과 시 안전 |
| **B** | 5 commits 단계별 보존 merge + WASM 빌드 + 시각 판정 | TDD 5 단계 보존, 후속 디버깅 시 commit 단위 추적 가능 |
| **C** | merge 보류 — 광범위 fixture sweep (메인테이너 환경) 후 결정 | PR 본문 Test plan `[ ] 광범위 fixture sweep (메인테이너 환경)` 미체크 영역 처리 |
| **D** | 일부 축만 cherry-pick (예: 축 1 + 축 5 만) | 5 축 동시 정합의 시너지를 분해하면 효과 약화 위험 — 비권장 |

## 8. 잠정 결정

**옵션 B + WASM 빌드 + 작업지시자 시각 판정** 잠정 권장.

이유:
1. 결정적 검증 (1221 테스트) ALL PASS, 회귀 0건
2. 페이지 수 정합 회복 (40 → 35) 확증
3. 5 축 분해의 본질 정합 — `sb_N ≠ sb_(N+1)` 통찰이 정확
4. 12번째 사이클 컨트리뷰터의 TDD 절차 (Stage 0~5 + 후속) 준수
5. **단, `feedback_v076_regression_origin` 룰에 의해 작업지시자 시각 판정이 핵심 게이트** — 컨트리뷰터 환경 (Linux/Mac PDF) 정합과 작업지시자 한컴 편집기 환경 정합이 일치하는지 시각 확인 필요

## 9. 작업지시자 결정 요청

1. **옵션 선택**: A / B / C / D 중?
2. **WASM 빌드 + 시각 판정 시점**: cherry-pick simulation 으로 충분한가? 또는 별도 WASM 빌드 후 rhwp-studio 에서 페이지 3, 6 시각 확인?
3. **잔존 LAYOUT_OVERFLOW 진단 ~9px**: 별도 후속 task 등록? 아니면 본 PR 범위 내 처리?
4. **광범위 fixture sweep 영역**: 작업지시자 환경에서 추가 sweep 필요?

---

작성: 2026-05-08
