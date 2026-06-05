# PR #1268 검토 - legacy HWP OLE 차트 Contents 렌더링

- PR: https://github.com/edwardkim/rhwp/pull/1268
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1251
- 작성일: 2026-06-03
- PR 작성자: @postmelee
- base: `devel`
- head: `task-1251-ole-chart` / `703398b148136d2d5fcc1aed2d74f2bfe07eb287`
- 규모: 25 files, +3041 / -9
- GitHub mergeable: `CONFLICTING`
- PR 댓글: 없음

## 1. PR 요약

PR #1268은 legacy HWP OLE 차트 객체의 nested OLE `/Contents` 스트림을 파싱해 `RawSvg` 렌더링 경로로 내리는 첫 구현이다.

대상 fixture는 다음 파일이다.

- `samples/143E433F503322BD33.hwp`
- `BinData #2`
- nested OLE container 내부에 `OOXMLChartContents`, `OlePres000`, native image preview가 없고 `/Contents` 스트림만 존재

기존 렌더링은 해당 객체를 다음 placeholder로 표시했다.

```text
OLE 개체 (BinData #2)
```

이번 PR은 `/Contents`에서 title/category/series/value를 추출하고, Rust SVG renderer로 차트 형태를 만들어 `PageRenderTree`의 `RawSvg` 노드로 삽입한다.

## 2. 주요 변경 범위

| file | 변경 |
|---|---|
| `src/ole_chart/parser.rs` | legacy HWP OLE `/Contents` probe 및 최소 차트 데이터 추출 |
| `src/ole_chart/ir.rs` | renderer-neutral OLE chart IR JSON/base64 helper |
| `src/ole_chart/svg_renderer.rs` | canonical Rust SVG chart fragment/standalone renderer |
| `src/ole_chart/charming_renderer.rs` | PR 원본에는 optional native `charming` SSR adapter가 있었으나, 통합본에서는 링크 오류로 제외 |
| `src/ole_chart/mod.rs` | OLE chart module public API |
| `src/renderer/layout/shape_layout.rs` | OLE fallback 순서에 `/Contents` chart parser + RawSvg 경로 추가 |
| `Cargo.toml` | PR 원본의 `charming-renderer` feature는 통합 검증에서 제외, Rust SVG 경로 관련 기존 의존성만 유지 |
| `tests/issue_1251_ole_chart_contents.rs` | fixture 기반 `/Contents` probe/parser/render 회귀 테스트 |
| `mydocs/...task_m100_1251*` | 계획/단계/결정/분석/보고 문서 추가 |

## 3. 검토 결과

수정 방향은 조건부로 수용 가능하다.

- 이 fixture는 OOXML chart나 preview image가 없으므로 기존 fallback만으로는 차트 정보가 모호하게 손실된다.
- `charming`을 기본 browser/WASM 렌더러로 쓰지 않고, Rust SVG renderer를 canonical path로 둔 결정은 현재 `PageLayerTree`/multi-backend 방향과 맞다.
- `charming`을 `charming-renderer` feature 뒤 optional native adapter로 둔 의도는 좋지만, 실제 통합 검증에서는 `deno_core/v8`가 현재 `cdylib` crate-type과 링크 충돌을 일으켰다.
- `/Contents` 파서는 현재 완전한 legacy chart parser가 아니라 이 fixture에서 확인된 `ChartOBJ`/`VtDataGrid`/`VtChartTitle` 기반의 제한적 parser다. PR 본문과 문서가 이 한계를 명시하고 있으므로 1차 경로로는 받아들일 수 있다.

주의점:

- PR branch는 현재 `devel`보다 5 commits behind, 1 commit ahead 상태다.
- GitHub는 mergeable을 `CONFLICTING`으로 표시한다.
- 실제 충돌 가능성이 높은 파일은 `mydocs/orders/20260603.md`와 archive 정책이 적용된 `mydocs/plans`, `mydocs/report`, `mydocs/working` 문서 위치다.
- 현재 로컬 `devel`에는 `samples/[2027] 온새미로 1 본교재.hwp` 샘플 추가 커밋 `1a37bd14`가 아직 `origin/devel`에 push되지 않은 상태다. 통합 브랜치 생성 전 이 커밋을 함께 가져갈지 분리할지 결정해야 한다.
- 파서가 `EUC_KR` label heuristic과 dense/sparse `f64` run 탐지에 의존하므로, 향후 다른 legacy chart fixture에서는 오탐/미탐 가능성이 있다.
- `shape_layout.rs`에서 `/Contents`가 존재하고 chart parser가 실패하면 명시적 OLE chart fallback placeholder를 만들고 `rendered=true`로 처리한다. generic OLE placeholder보다 낫지만, chart가 아닌 `/Contents` OLE에도 이 경로가 적용될 수 있는지 추가 샘플로 확인이 필요하다.
- `cargo test --features charming-renderer --test issue_1251_ole_chart_contents -- --nocapture`는 `v8` 정적 라이브러리가 `cdylib` 링크에 들어오며 실패했다. 통합본에서는 `charming` feature/dependency를 제외하고 Rust SVG renderer 경로만 유지하는 편이 안전하다.

## 4. GitHub Actions

PR head 기준:

| workflow/check | 결과 |
|---|---|
| CI / Build & Test | pass |
| Render Diff / Canvas visual diff | pass |
| CodeQL / Analyze rust | pass |
| CodeQL / Analyze javascript-typescript | pass |
| CodeQL / Analyze python | pass |
| WASM Build | skipped |

## 5. 로컬 사전 확인

수행한 확인:

```text
gh pr checks 1268 --repo edwardkim/rhwp
```

GitHub connector로 확인한 상태:

| 항목 | 결과 |
|---|---|
| PR 상태 | open |
| draft | false |
| changed files | 25 |
| PR comments | 없음 |
| related issue #1251 | open |
| compare `devel...pull/1268/head` | diverged, ahead 1 / behind 5 |

## 6. 권장 처리

권장: **PR 기능은 수용하되, 최신 `devel` 기준 메인테이너 통합 브랜치에서 cherry-pick 후 문서 archive 정책과 충돌을 정리한다.**

진행 순서:

1. 현재 `devel`의 샘플 파일 커밋 `1a37bd14` 처리 방침 확정
   - 권장: 먼저 `origin/devel`에 push하거나, PR #1268 통합 브랜치가 이 커밋을 포함한다는 점을 명시
2. 최신 `devel` 기준 통합 브랜치 생성
3. PR #1268 commit `703398b1` cherry-pick
4. `mydocs/orders/20260603.md` 충돌은 현재 `devel` 기록을 보존하고 #1251 항목만 필요한 방식으로 반영
5. PR 추가 문서는 현행 정책에 맞춰 archive 위치로 이동
   - `mydocs/plans/archives/task_m100_1251*.md`
   - `mydocs/working/archives/task_m100_1251_stage*.md`
   - `mydocs/report/archives/task_m100_1251*.md`
6. 자동 검증
   - `cargo fmt --all --check`
   - `cargo test --test issue_1251_ole_chart_contents -- --nocapture`
   - `cargo check --target wasm32-unknown-unknown --lib`
   - `cargo test --tests --quiet`
   - `cargo clippy --all-targets -- -D warnings`
7. SVG export로 메인테이너 시각 판정 자료 생성
   - `samples/143E433F503322BD33.hwp`
8. 필요 시 `wasm` 빌드 후 rhwp-studio 시각 판정
9. `devel` 병합/push 후 CI 확인
10. PR #1268 및 issue #1251 종료 처리

## 7. PR 코멘트 초안

```markdown
검토했습니다. 이 fixture는 nested OLE 안에 `OOXMLChartContents`, `OlePres000`, native image preview가 없고 `/Contents` 스트림만 존재하므로, 기존 generic OLE placeholder보다 명시적인 legacy chart parser + RawSvg 경로를 두는 접근은 이슈 #1251의 목표와 잘 맞습니다.

특히 `charming`을 기본 browser/WASM renderer로 삼지 않고, Rust SVG renderer를 canonical `RawSvg` 경로로 둔 결정은 현재 rhwp의 page layer/multi-backend 방향과 맞다고 봅니다. 다만 통합 검증에서 `charming`의 `deno_core/v8` 의존성이 현재 `cdylib` 링크와 충돌했으므로, 이번 통합본에서는 `charming` optional adapter를 제외하고 Rust SVG 경로만 반영하겠습니다.

다만 현재 PR은 최신 `devel`보다 뒤처져 있고 문서 위치/archive 정책 및 `mydocs/orders/20260603.md` 충돌이 있으므로, 메인테이너 통합 브랜치에서 cherry-pick 후 충돌과 문서 위치를 정리해 반영하겠습니다.
```
