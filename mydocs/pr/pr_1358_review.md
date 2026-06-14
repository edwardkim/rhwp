# PR #1358 검토 — 미주 다단 오버플로 조사 + 시각 회귀 하니스 확장

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1358 |
| 작성자 | `planet6897` |
| 상태 | open / draft 아님 |
| base | `devel` |
| head | `planet6897:chore/endnote-overflow-investigation-1357` |
| 관련 이슈 | #1357, #1336, #1082 |
| 변경 규모 | 7 files, +248 / -0 |
| mergeable | `MERGEABLE`, `BEHIND` |

## 2. 변경 요약

이 PR은 렌더러 소스 수정 PR이 아니다.

핵심은 #1336 잔여인 `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` 미주 다단 p21 col0 overflow를 조사하고, 정밀 수정 시도가 cascade 회귀를 만들었음을 문서화한 뒤, 해당 샘플을 시각 회귀 sweep 대상으로 추가하는 것이다.

변경:

- `scripts/task1274_visual_sweep.py`
  - 새 target `2024-09-below20above20` 추가
  - HWP: `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`
  - PDF: `pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf`
- `mydocs/plans/task_m100_1357.md`
- `mydocs/plans/task_m100_1357_impl.md`
- `mydocs/report/task_m100_1357_report.md`
- `mydocs/working/task_m100_1357_stage1.md`
- `mydocs/working/task_m100_1357_stageA.md`
- `mydocs/working/task_m100_1357_stageBC.md`

PR 본문 결론:

- 정밀 수정은 p21 col0 overflow를 줄였지만 overflow가 col1로 전이되어 전체 회귀가 악화됨.
- 전 문서 오버플로 총합이 50.1px에서 72.9px로 악화되어 `issue_1082` 가드 실패.
- 따라서 현재는 60px 바운드 추적으로 유지하고, 근본 정정은 typeset 누적기 정합 전용 대형 타스크로 분리하는 것이 타당하다는 결론.

## 3. GitHub 상태

GitHub Actions:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL Analyze (javascript-typescript) | pass |
| CodeQL Analyze (python) | pass |
| CodeQL Analyze (rust) | pass |
| CodeQL | pass |
| WASM Build | skipped |

관련 이슈:

- #1357은 현재 open 상태.
- PR 본문은 `refs #1357 #1336 #1082` 형식이라 자동 close는 기대하기 어렵다.
- 이 PR을 수용하면서 #1357을 "deferred-tracked"로 닫을지, open 상태로 유지할지는 작업지시자 판단이 필요하다.

## 4. 로컬 검토 방식

검토 기준 브랜치:

```text
local/devel @ 37710a8b
```

PR head fetch:

```text
local/pr1358-upstream @ 5ffa0f1b
```

통합 시뮬레이션:

```text
local/pr1358-merge-test @ 6918f2e0
```

적용 방식:

- `origin/devel` 기준 검증 브랜치 생성
- PR 단일 커밋 `5ffa0f1b` cherry-pick
- 충돌 없음

## 5. 로컬 검증

실행 완료:

```bash
git diff --check
python3 -m py_compile scripts/task1274_visual_sweep.py
python3 scripts/task1274_visual_sweep.py --help
cargo test --test issue_1082_endnote_multicolumn_drift
cargo fmt --all -- --check
python3 scripts/task1274_visual_sweep.py --target 2024-09-below20above20 --out output/poc/pr1358-visual-sweep --rhwp-bin target/debug/rhwp
```

결과:

| 명령 | 결과 |
|---|---|
| `git diff --check` | 통과 |
| `python3 -m py_compile scripts/task1274_visual_sweep.py` | 통과 |
| `python3 scripts/task1274_visual_sweep.py --help` | 통과, 새 target 노출 확인 |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과, 5 passed |
| `cargo fmt --all -- --check` | 통과 |
| 새 target sweep 실행 | 통과 |

새 target sweep 산출:

- summary: `output/poc/pr1358-visual-sweep/summary.json`
- manifest: `output/poc/pr1358-visual-sweep/2024-09-below20above20/manifest.json`
- contact sheet: `output/poc/pr1358-visual-sweep/2024-09-below20above20/contact_sheet.png`
- annotated pages:
  - `output/poc/pr1358-visual-sweep/2024-09-below20above20/analysis/annotated_009.png`
  - `output/poc/pr1358-visual-sweep/2024-09-below20above20/analysis/annotated_010.png`
  - `output/poc/pr1358-visual-sweep/2024-09-below20above20/analysis/annotated_013.png`
  - `output/poc/pr1358-visual-sweep/2024-09-below20above20/analysis/annotated_017.png`
  - `output/poc/pr1358-visual-sweep/2024-09-below20above20/analysis/annotated_018.png`
  - `output/poc/pr1358-visual-sweep/2024-09-below20above20/analysis/annotated_019.png`
  - `output/poc/pr1358-visual-sweep/2024-09-below20above20/analysis/annotated_022.png`

sweep 결과 요약:

- 23페이지 모두 렌더/비교됨.
- flagged: 7/23
- overflow lines:
  - p21 col0 `para=1156~1159`, 최대 overflow `82.1px`
- frame overflow pages: 없음
- render tree frame tail overflow pages: 9, 17, 18, 19, 22
- line/column drift 후보가 산출되어 이 타겟이 회귀 검출용으로 의미 있게 동작함을 확인했다.

## 6. 검토 의견

차단 이슈는 발견하지 못했다.

수용 가능한 이유:

- PR은 런타임 렌더러 소스를 변경하지 않는다.
- `scripts/task1274_visual_sweep.py`에 대상 샘플을 추가하는 변경은 작고 명확하다.
- 새 target은 실제 파일 존재 확인, `--help` 노출, 전체 sweep 실행까지 통과했다.
- `issue_1082_endnote_multicolumn_drift` 5개 테스트가 통과하여 현재 바운드 추적 상태가 유지된다.
- 문서에는 정밀 수정 시도가 왜 실패했는지와 어떤 회귀를 만들었는지 기록되어 있어, 같은 경로 재조사를 줄이는 가치가 있다.

주의점:

- 이 PR은 #1357의 근본 버그를 고치는 PR이 아니다. "조사 결과 보존 + 회귀 하니스 확장" PR이다.
- PR head는 `BEHIND` 상태였으나, 최신 `origin/devel` 기준 cherry-pick은 충돌 없이 적용되었다.
- contributor 작업 문서가 활성 `mydocs/plans`, `mydocs/report`, `mydocs/working`에 추가된다. 수용 시 archive 정책에 맞춰 이동하는 것이 좋다.
- #1357을 닫을 경우, "해결"보다는 "deferred-tracked / 바운드 추적으로 전환"이라는 코멘트를 명확히 남겨야 한다.

## 7. 권장 처리

권장: **수용 가능**.

권장 절차:

1. 작업지시자 승인.
2. `local/devel` 기준으로 PR 커밋 반영.
3. contributor 작업 문서 archive 정리.
4. 최종 검증:
   - `git diff --check`
   - `python3 -m py_compile scripts/task1274_visual_sweep.py`
   - `python3 scripts/task1274_visual_sweep.py --help`
   - `cargo test --test issue_1082_endnote_multicolumn_drift`
5. 처리 보고서 작성.
6. `origin/devel` push.
7. PR #1358에 처리 코멘트 작성 후 close.
8. #1357은 작업지시자 판단에 따라 `deferred-tracked` 설명과 함께 close하거나 open 유지.

## 8. 승인 요청

위 검토 결과 기준으로 PR #1358 수용 절차를 진행해도 되는지 승인 요청한다.
