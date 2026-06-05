# PR #1292 리뷰 - 교육 통합 문항 흐름 drift 보정

- PR: https://github.com/edwardkim/rhwp/pull/1292
- 작성일: 2026-06-05
- 작성자: `jangster77`
- 제목: `task 1284: #1274 후속 교육 통합 문항 흐름 drift 보정`
- base: `devel` / `9d3aa212454f5a7d9d7e081ddaec40a804aeda70`
- head: `task_m100_1284` / `1e49b1b6308039271873d79eb8ed8440fb4eee98`
- 상태: open, draft 아님
- GitHub mergeable: true

## 1. PR 요약

PR #1292는 #1274 / PR #1277 후속으로, 교육 통합 문서의 미주 기반 2단 문항 흐름에서 남아 있던 marker drift를 보정한다.

핵심 방향:

- PDF bbox와 rhwp render tree bbox를 비교하는 visual sweep 지표 강화
- compact 미주, 수식-only tail, 문항 제목 tail, TAC 그림-only 문단 뒤 흐름 보정
- `3-09월_교육_통합_2024-미주사이20`, `3-09월_교육_통합_2023`, `3-09월_교육_통합_2022`, `3-10월_교육_통합_2022`, `3-11월_실전_통합_2022` 계열의 문항 시작 위치와 tail overflow 보정
- `3-09월_교육_통합_2024-구분선아래20구분선위20` HWP/HWPX/PDF 샘플 추가

이번 PR은 특정 샘플 한두 개의 국소 패치라기보다, 미주 풀이 영역의 높이 커서와 typeset 분기 전체에 영향을 주는 변경이다.

## 2. 변경 범위

PR base 기준:

```text
18 commits
24 files changed
3636 insertions(+), 62 deletions(-)
```

| file | 변경 |
|---|---|
| `src/renderer/height_cursor.rs` | compact 미주 gap, 수식-only/tail/title 흐름 보정 helper 및 테스트 추가 |
| `src/renderer/layout.rs` | 미주 문항 제목 tail, direct bottom fit, 수식 tail 뒤 compact 처리 추가 |
| `src/renderer/typeset.rs` | 기본 7mm/large-between note tail, last-column visual split, frame 하단 보존 로직 추가 |
| `scripts/task1274_visual_sweep.py` | PDF bbox 기반 question flow/drift 분석, false positive 억제, `question_flow.json` 출력 추가 |
| `tests/issue_1139_inline_picture_duplicate.rs` | issue #1284 관련 교육 통합 페이지별 bbox 회귀 테스트 다수 추가 |
| `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` | 신규 검증 샘플 |
| `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx` | 신규 검증 샘플 |
| `pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf` | 신규 PDF 기준 파일 |
| `mydocs/working/task_m100_1284_stage*.md` | 컨트리뷰터 작업 기록 |
| `mydocs/orders/20260603.md` | 작업 메모 1줄 추가 |

## 3. GitHub Actions 상태

PR head `1e49b1b6308039271873d79eb8ed8440fb4eee98` 기준:

| workflow | run id | conclusion |
|---|---:|---|
| CI | 26986440097 | success |
| Render Diff | 26986440093 | success |
| CodeQL | 26986440101 | success |

단, PR head는 현재 `local/devel`보다 3커밋 뒤에 있다.

```text
current local/devel: 160e6bc8 Merge local/devel: PR #1295 vite update
PR merge base:      9d3aa212 chore(deps): #1217 후속 ...
PR head:            1e49b1b task_m100_1284
status vs current:  diverged, PR head behind current devel by 3 commits
```

따라서 GitHub Actions 성공은 PR head 자체의 신호로는 유효하지만, maintainer 수용 전에는 최신 `local/devel` 위에서 다시 통합 검증해야 한다.

## 4. 코드 검토

### 4.1 문제 정의 정합성

PR 설명의 문제 정의는 최근 교육 통합 문서에서 반복 관찰된 현상과 맞다.

- 미주 풀이 문항 제목이 다음 단/다음 페이지로 과도하게 밀림
- 수식-only 또는 TAC 그림-only 문단 뒤에서 실제 한컴/PDF와 marker 흐름이 벌어짐
- frame 하단에 남아야 할 문항 제목/tail이 조기 split되거나, 반대로 frame 밖으로 bleed됨

visual sweep을 PDF bbox와 render tree bbox 기준으로 확장한 것도 이 문제군에는 적절하다. 단순 페이지 수 비교만으로는 문항 drift를 잡기 어렵기 때문이다.

### 4.2 `height_cursor.rs`

`HeightCursor`에 compact 미주 전용 상태와 helper가 많이 추가된다.

주요 변경:

- `para_has_equation_only`
- `compact_between_notes_gap_px`
- `last_compacted_endnote_title_gap`
- compact 미주 title/body gap, stale forward gap, 수식-only tail, TAC 그림-only 문단 뒤 vpos 조정

장점:

- 기존 개별 케이스들이 `layout` 또는 `typeset`에 흩어지지 않고 height cursor 단계에서 일부 공통화된다.
- 신규 단위 테스트가 여러 gap 케이스를 직접 검증한다.

위험:

- 조건식이 많고 `endnote_between_notes_hu`, question title 여부, visible text 여부, TAC 여부가 결합되어 있다.
- 기존 샘플에는 맞지만 일반 미주 문서에서 gap을 과소 적용할 가능성이 있다.
- 상태 플래그 `last_compacted_endnote_title_gap`은 다음 문단 처리에 영향을 주므로 split/column 이동 경계에서 누락 회귀가 생길 수 있다.

### 4.3 `layout.rs`

미주 문항 제목 tail을 frame 하단에 남길 수 있도록 직접 bottom fit과 backtrack 계열 처리가 추가된다.

장점:

- PDF/한컴처럼 제목 일부 또는 첫 풀이 줄이 현재 단 하단에 남는 케이스를 설명할 수 있다.
- 직전 수식 tail 이후 title gap이 과도하게 누적되는 문제를 줄인다.

위험:

- `prev_item_content_bottom_y`, `title_tail_backtracked`, compact equation-tail title gap이 서로 영향을 준다.
- 단/페이지 전환 경계에서 title만 남고 본문이 다음 단으로 넘어가는 케이스는 시각 판정이 필수다.

### 4.4 `typeset.rs`

가장 위험도가 높은 변경이다.

주요 변경:

- 기본 7mm 미주 문항 제목 tail 허용
- large-between note에서 last-column visual split 판단
- local `HeightCursor` 예측으로 render overflow를 미리 계산
- `large_between_title_tail_render_overflows`, `large_between_last_column_visual_split` 등 split 후보 확장

장점:

- 기존 분할표/미주 drift에서 계속 문제되던 "실제 렌더 기준으로는 들어가는데 논리 계산에서 다음 단으로 밀리는" 케이스를 잡을 수 있다.
- PR 테스트가 page/question 단위로 상당히 촘촘하다.

위험:

- typeset 내부에서 예측용 height cursor를 다시 돌리는 구조는 실제 렌더 pipeline과 미세하게 어긋날 수 있다.
- 특정 교육 통합 문서군에는 맞지만, 미주가 있는 일반 문서/다단 문서/수식 많은 문서에서 다른 drift를 만들 수 있다.
- 최근 페이지네이션 PR들이 계속 누적된 상태라, #1145, #1153, #1285 계열 회귀 샘플을 함께 확인해야 한다.

### 4.5 `task1274_visual_sweep.py`

PDF word bbox와 render tree bbox를 함께 비교하고 `question_flow.json`을 생성하도록 확장된다.

좋은 점:

- 문항 marker drift를 자동 검출할 수 있어 maintainer 시각 판정 부담을 줄인다.
- 작은 tail bleed와 실제 frame overflow를 분리하려는 시도도 적절하다.

주의점:

- `pdftotext`, `pdftoppm`, `rsvg-convert` 같은 외부 도구 의존성이 있다.
- CI/로컬 환경에서 도구 유무에 따라 실행 가능성이 달라질 수 있으므로, 필수 테스트와 보조 스윕을 구분해야 한다.

### 4.6 테스트 추가

`tests/issue_1139_inline_picture_duplicate.rs`에 issue #1284 관련 페이지별 bbox 테스트가 대량 추가된다.

강점:

- 2022/2023/2024 교육 통합 문서의 여러 페이지를 직접 고정한다.
- 문항 제목 y 위치, tail 유지, 수식/본문 겹침 회귀를 수치로 잡는다.

주의점:

- 테스트 파일 하나가 계속 커지는 구조다.
- 지난 #1146에서 확인했던 것처럼 장기적으로는 issue/문서군별 테스트 분리 필요성이 커진다.
- 이번 PR 수용 blocker는 아니지만, 후속 정리 후보로 남겨야 한다.

## 5. 샘플/문서 처리 검토

신규 샘플:

- `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`
- `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx`
- `pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf`

이 샘플들은 PR 검증 목적이 명확하므로 수용 가능하다.

다만 `mydocs/working/task_m100_1284_stage*.md`는 PR 처리 후 repo 정책에 맞춰 archive 정리 여부를 판단해야 한다. 최근 `mydocs/pr`, `plans`, `report`, `working` 문서는 archive로 정리하는 흐름이 있으므로, 수용 커밋에 포함할지 또는 maintainer 정리 커밋에서 이동할지 확인이 필요하다.

## 6. 권장 처리

권장: **수용 방향으로 진행하되, 최신 `local/devel` 위 maintainer integration + 강한 시각 판정 게이트 필수**.

근거:

- PR head 기준 CI / Render Diff / CodeQL 모두 성공
- 문제 정의가 실제 교육 통합 drift 이슈와 맞음
- 신규 visual sweep과 bbox 테스트가 PR 주장에 대한 검증 근거를 제공함
- 하지만 `typeset`/`height_cursor` 변경량이 크고 조건식이 복잡하여 일반 페이지네이션 회귀 가능성이 있음

권장 절차:

1. 최신 `local/devel`에서 통합 브랜치 생성
2. PR #1292 head를 fetch
3. 전체 head merge가 아닌 실제 기능 커밋 cherry-pick 또는 PR branch merge를 비교 후 선택
4. 충돌/문서 위치 정리
5. `cargo fmt --all -- --check`
6. `cargo build --verbose`
7. `cargo check --target wasm32-unknown-unknown --lib`
8. `cargo test --features native-skia skia --lib --verbose`
9. `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
10. `python3 -m py_compile scripts/task1274_visual_sweep.py`
11. 가능하면 `python3 scripts/task1274_visual_sweep.py --target all`
12. maintainer SVG/웹 시각 판정
13. 통과 시 `devel` 병합/push 및 PR 종료 처리

시각 판정 우선 샘플:

- `samples/3-09월_교육_통합_2024-미주사이20.hwp`
- `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`
- `samples/3-09월_교육_통합_2023.hwp`
- `samples/3-09월_교육_통합_2022.hwp`
- `samples/3-10월_교육_통합_2022.hwp`
- `samples/3-11월_실전_통합_2022.hwp`

회귀 guard로 함께 볼 샘플:

- `samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx`
- `samples/synam-001.hwp`
- `samples/kps-ai.hwp`
- `samples/21_언어_기출_편집가능본.hwp`

## 7. PR 코멘트 초안

```markdown
검토했습니다. 이번 PR은 #1274 / PR #1277 이후 남아 있던 교육 통합 문서의 문항 marker drift를 PDF bbox와 render tree bbox 기준으로 추적하고, compact 미주/수식-only tail/문항 제목 tail 흐름을 공통 로직으로 보정하는 큰 변경입니다.

PR head 기준 CI / Render Diff / CodeQL이 모두 성공했고, visual sweep과 page/question 단위 회귀 테스트가 함께 추가되어 있어 수용 방향으로 진행할 수 있다고 판단합니다.

다만 `HeightCursor`, `layout`, `typeset`의 페이지네이션 조건이 넓게 바뀌므로 maintainer integration에서는 최신 `devel` 위에서 다시 검증하겠습니다. 특히 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`, native-skia lib test, wasm check, visual sweep, 그리고 maintainer SVG/웹 시각 판정을 게이트로 두겠습니다.

기여 감사합니다.
```

## 8. 통합 진행 기록

통합 브랜치:

```text
local/pr1292-integration
```

적용 방식:

```text
git fetch origin pull/1292/head:local/pr1292-upstream
git checkout -B local/pr1292-integration local/devel
git cherry-pick f3f4c0d7 a583c66c f2fa3632 b16e4e8f bc53f250 0495601b a83058e2 21b126f8 a8376127 b0462d67 28404d93 dcd99f59 54a214fb 7af89f67 b22c0a68
```

주의:

- PR head는 최신 `local/devel`보다 3커밋 뒤에 있었다.
- PR branch 전체를 merge하면 PR #1295의 Vite 업데이트와 archive 문서가 되돌아가는 diff가 발생했다.
- 따라서 PR branch의 `Merge branch 'devel' into task_m100_1284` 계열 merge commit 3개는 제외하고, 실제 기능 커밋 15개만 최신 `local/devel` 위에 cherry-pick했다.
- 적용 후 diff는 PR 의도 범위인 24파일로 정리되었고, `rhwp-studio/package*.json` 및 `mydocs/pr/archives/pr_1295_*`에는 변경이 없다.

자동 검증:

| 항목 | 결과 | 비고 |
|---|---|---|
| `cargo fmt --all -- --check` | 통과 |  |
| `python3 -m py_compile scripts/task1274_visual_sweep.py` | 통과 |  |
| `cargo build --verbose` | 통과 | Cargo global cache last-use DB readonly 경고만 발생 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |  |
| `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` | 통과 | 67 passed |
| `cargo test --features native-skia skia --lib --verbose` | 통과 | 39 passed |
| `python3 scripts/task1274_visual_sweep.py --target all` | 미실행 | 로컬 필수 도구 `rsvg-convert` 없음 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 | Done in 2m 54s, `pkg/rhwp_bg.wasm` 5.3M |

WASM 동기화:

```text
cp pkg/rhwp.js pkg/rhwp_bg.wasm pkg/rhwp.d.ts pkg/rhwp_bg.wasm.d.ts rhwp-studio/public/
```

해시 확인:

| file | 상태 |
|---|---|
| `pkg/rhwp_bg.wasm` / `rhwp-studio/public/rhwp_bg.wasm` | 동일 |
| `pkg/rhwp.js` / `rhwp-studio/public/rhwp.js` | 동일 |
| `pkg/rhwp.d.ts` / `rhwp-studio/public/rhwp.d.ts` | 동일 |
| `pkg/rhwp_bg.wasm.d.ts` / `rhwp-studio/public/rhwp_bg.wasm.d.ts` | 동일 |

메인테이너 시각 판정:

```text
2026-06-05 통과
```

현재 판정:

- 자동 검증과 maintainer SVG/웹 시각 판정을 모두 통과했다.
- `python3 scripts/task1274_visual_sweep.py --target all`은 로컬 필수 도구 `rsvg-convert`가 없어 미실행이지만, PR head 기준 Render Diff는 성공했고 maintainer 시각 판정으로 최종 게이트를 통과했다.
- PR #1292는 수용 가능하다.
