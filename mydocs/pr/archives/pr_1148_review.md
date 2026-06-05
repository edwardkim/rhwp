# PR #1148 검토 — HWPX TopAndBottom 표 host_spacing 보정

## 1. PR 메타

| 항목 | 값 |
|---|---|
| PR | #1148 |
| 제목 | Task #1147: HWPX TopAndBottom 표 host_spacing 보정 |
| 작성자 | planet6897 (Jaeuk Ryu) |
| base ← head | `devel` ← `feature/task_m100_1147` |
| 상태 | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | BEHIND |
| 변경 규모 | 6 files, +372 / -1 |
| 연결 이슈 | closes #1147 |

## 2. 문제 요약

이슈 #1147은 HWPX 원본에서 `wrap=TopAndBottom` 비-TAC 표가 빈 앵커 문단에 있을 때,
표 뒤 한 줄 문단이 권위 PDF/한컴 출력과 달리 다음 페이지로 넘어가는 문제다.

PR의 핵심 주장은 다음과 같다.

```text
HWPX LINE_SEG 시멘틱에서는 빈 앵커 문단 vpos가 직전 문단 종료 vpos와 같다.
따라서 HWPX 빈 앵커 + TopAndBottom 비-TAC 표에서는 spacing_before와 host_line_spacing을
별도 가산하면 페이지 used가 과대 계산된다.
```

## 3. 변경 내용

코드 변경:

```text
src/renderer/typeset.rs
  - TypesetState.is_hwpx_source 추가
  - typeset_section_with_variant() 인자로 HWPX source 여부 전달
  - format_table()에서 HWPX + 비-TAC + TopAndBottom + 빈 앵커 문단이면
    host_line_spacing=0, before=outer_top으로 계산

src/document_core/queries/rendering.rs
  - DocumentCore.source_format == Hwpx 여부를 TypesetEngine에 전달
```

문서 변경:

```text
mydocs/plans/task_m100_1147.md
mydocs/plans/task_m100_1147_impl.md
mydocs/working/task_m100_1147_stage1.md
mydocs/report/task_m100_1147_report.md
```

비공개 샘플명/식별 가능한 내용은 문서에 직접 노출되어 있지 않다.

## 4. 로컬 검증

검토 브랜치:

```text
local/pr1148-review
```

최신 `local/devel` 병합 시뮬레이션:

```text
git checkout -b local/pr1148-merge-test
git merge local/devel --no-commit --no-ff
```

결과:

```text
conflict: none
```

검증:

```text
cargo fmt --all -- --check
cargo test --lib
cargo test --test svg_snapshot
docker compose --env-file .env.docker run --rm wasm
cargo test --tests
```

결과:

```text
fmt: pass
cargo test --lib: 1411 passed, 0 failed, 6 ignored
svg_snapshot: 8 passed
wasm build: pass (pkg/rhwp_bg.wasm 4.9M)
cargo test --tests: pass
```

`cargo test --tests`는 lib 테스트와 통합 테스트 전체를 포함해 실행했다. 특히 다음 계열의
페이지네이션/표 분할 회귀 테스트가 함께 통과했다.

```text
issue_1070_tac_table_post_text_overflow
issue_1073_nested_table_split
issue_1079_picture_pushdown_vpos
issue_1082_endnote_multicolumn_drift
issue_1086
issue_1100_exam_social_hwpx_header
issue_1105
issue_1116
issue_1145
issue_546 / issue_554 / issue_643 / issue_703 / issue_775 / issue_986
svg_snapshot
```

메인테이너 시각 판정:

```text
2026-05-27 통과
```

GitHub CI:

```text
Build & Test: SUCCESS
Render Diff / Canvas visual diff: SUCCESS
CodeQL: SUCCESS
WASM Build: SKIPPED
```

## 5. 검토 의견

### 긍정 요소

1. 변경 범위가 `TypesetEngine`의 HWPX source 경로로 한정되어 있다.
2. HWP/HWP3 기존 경로는 `is_hwpx_source=false`라 PR 분기가 발동하지 않는다.
3. `hwpspec.hwp` 178쪽 정합 유지 검증을 PR 본문에 포함했다.
4. 로컬 `local/devel` 병합 시뮬레이션과 `svg_snapshot`가 통과했다.

### 주의 요소

1. `format_table()` 인자가 늘면서 `#[allow(clippy::too_many_arguments)]`가 추가된다.
   현재 함수가 이미 측정/배치 context를 많이 받는 구조라 단기 수용은 가능하지만,
   #1146 코드베이스 정리 관점에서는 후속 리팩터링 후보다.
2. 문제 재현 샘플이 비공개라 메인테이너가 직접 동일 샘플을 재현 검증하기 어렵다.
   대신 공개 SVG 스냅샷과 기존 HWP/HWPX 회귀 테스트를 게이트로 삼는다.
3. PR 본문에 TAC 1x1 표 +42.9px 드리프트가 후속 latent 이슈로 남아 있다고 명시되어 있다.

## 6. 권장안

**조건부 수용 권장.**

조건:

```text
1. WASM 빌드 후 웹 캔버스 시각 판정 수행
2. devel push 전 페이지네이션 회귀 테스트 수행
3. PR merge 후 이슈 #1147 close 확인
4. TAC 1x1 표 드리프트는 별도 이슈로 분리 검토
```

현재 로컬 기준으로는 코드 충돌, 시각 회귀, 페이지네이션 자동 회귀가 발견되지 않았다.
