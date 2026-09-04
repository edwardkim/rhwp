# 구현계획 — Task M100 #3789 `src/main.rs` 잔여 렌더 경계 분리

- **상위 수행계획**: [task_m100_3789.md](task_m100_3789.md)
- **이슈**: [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **작성일**: 2026-08-27 KST
- **작업 브랜치**: `task_m100_3789-render-boundary`
- **착수 기준**: `upstream/devel@1b91c2025`
- **구현 상태**: PR 리뷰 보정과 최신 `devel@f6a6bee8f` 전체 회귀 완료, 로컬 보정 candidate push 대기

## 1. 구현 불변식

- 공개 command 이름, argument parsing, stdout/stderr와 exit code를 유지한다.
- 문서 mutation 순서, page 순회, SVG 저장 이름·내용을 바꾸지 않는다.
- structure JSON의 schema와 모든 소비자의 단일 변환 authority를 유지한다.
- root에서 구현을 옮기되 다른 root helper나 re-export로 숨겨 되돌리지 않는다.
- #5776이 분류한 `outputs/mod.rs`, `outputs/pdf.rs`, `outputs/raster.rs`, `outputs/vector.rs`의 실제
  consumer mapping과 신규 adapter fail-closed inventory를 유지한다.
- CI source 목록은 workflow, trusted classifier, policy mirror에서 동시에 바꾼다.

## 2. 예정 소유 구조

```text
src/main.rs
└─ command parse / dispatch / composition

src/cli/
├─ commands/
│  └─ caption 전용 모듈       # test-caption mutation + SVG render
├─ outputs/
│  └─ vector.rs               # export-svg / export-render-tree만 소유
├─ queries/
│  └─ structure.rs            # export-structure + structure_json_value
└─ batch/query.rs             # queries::structure의 helper 재사용
```

caption 모듈의 최종 파일명은 Stage 1 dependency 확인에서 확정한다. 단순 내부 validation과 달리 직접
`render_page_svg`를 호출하는 source이므로, 어느 디렉터리에 두더라도 그 **정확한 파일 경로**를 Render Diff
workflow·classifier·policy mirror가 추적해야 한다.

## 3. 파일별 변경안

| 파일 | 예정 변경 | 보존 계약 |
| --- | --- | --- |
| [`src/main.rs`](../../../src/main.rs) | `test_caption`, `structure_json_value` 구현 제거와 module dispatch | command spelling, argument order, exit code |
| `src/cli/commands/<caption>.rs` | caption mutation·SVG render 구현 move-only 수용 | page 순회, SVG 이름·내용, 오류 문구 |
| [`src/cli/commands/mod.rs`](../../../src/cli/commands/mod.rs) | caption module wiring | 기존 command visibility |
| `src/cli/queries/structure.rs` | `export_structure`, `structure_json_value` 소유 | JSON schema·provenance |
| [`src/cli/queries/mod.rs`](../../../src/cli/queries/mod.rs) | structure query wiring | query API 범위 |
| [`src/cli/outputs/vector.rs`](../../../src/cli/outputs/vector.rs) | structure export와 root helper import 제거 | SVG·render-tree 출력 |
| [`src/cli/batch/query.rs`](../../../src/cli/batch/query.rs) | structure helper의 새 소유 경로 import | batch JSON 결과 |
| [`src/mcp_serve.rs`](../../../src/mcp_serve.rs) | 조사 중 확인한 세 번째 structure helper 소비자를 새 authority로 연결 | MCP structure JSON 결과 |
| [`.github/workflows/render-diff.yml`](../../../.github/workflows/render-diff.yml) | root 대신 caption render source 추적 | #5776 PDF/shared trigger |
| [`scripts/ci-impact-classifier.cjs`](../../../scripts/ci-impact-classifier.cjs) | `main-render-boundary` 제거, caption path 분류 | adapter별 영향 matrix |
| [`scripts/ci-impact-policy.cjs`](../../../scripts/ci-impact-policy.cjs) | workflow와 같은 path mirror | path 순서·일치 계약 |
| CI·CLI contract tests | root negative, caption positive, CLI 회귀 추가 | 기존 positive fixtures |

`vector.rs`가 렌더 출력만 남게 되는 사실만으로 Render Diff를 자동 활성화하지 않는다. #5776에서 확인한
현재 readiness-only workflow가 vector native capture를 소비하지 않는 한 일반 Rust 분류를 유지하고, 실제
consumer가 추가되는 변경에서 함께 승격한다.

## 4. source 이동 순서

1. 현행 `test-caption`의 입력 validation, fixture index guard, mutation 순서, 출력 경로와 SVG file 목록을
   characterization한다.
2. caption 전용 module을 만들고 본문을 move-only로 옮긴 뒤 root dispatch를 연결한다.
3. `structure_json_value`와 `export_structure`를 같은 query 모듈로 옮긴다.
4. vector output과 batch query가 새 authority를 참조하게 하고 root import 의존을 제거한다.
5. `rg`로 root의 renderer 호출·구조 JSON 구현 잔존 여부와 새 순환 dependency를 검사한다.
6. focused CLI 회귀와 대표 SVG·JSON 비교가 통과한 상태를 source 이동 commit으로 고정한다.

## 5. CI 경계 변경 순서

1. caption module의 확정 경로를 Render Diff `pull_request.paths`에 등록한다.
2. trusted classifier의 render source 집합에 같은 경로를 등록하고 `src/main.rs` 특례를 제거한다.
3. policy mirror에도 같은 경로·순서를 반영하고 `src/main.rs`를 제거한다.
4. classifier tests에 다음 대표 행렬을 고정한다.

| 변경 경로 | Rust | Render Diff | Native Skia | 기대 이유 |
| --- | ---: | ---: | ---: | --- |
| `src/main.rs` | true | false | false | 일반 Rust source |
| caption render module | true | true | true | 직접 SVG render boundary |
| `src/cli/outputs/mod.rs` | true | true | true | #5776 shared adapter |
| `src/cli/outputs/pdf.rs` | true | true | true | #5776 PDF consumer |
| `src/cli/outputs/raster.rs` | true | false | true | #5776 native raster consumer |
| `src/cli/outputs/vector.rs` | true | false | false | 현행 workflow가 미소비 |
| `src/cli/queries/structure.rs` | true | false | false | 비렌더 JSON query |

5. workflow trigger include/exclude와 policy mirror 일치 test를 갱신한다.
6. adapter inventory와 `render=true` 경로의 workflow 추적 불변식을 다시 실행한다.

## 6. 테스트와 검증

### 6.1 focused CLI 계약

- `issue_cli_test_caption_no_panic`: 임의 문서에서 panic 없이 SVG가 생성되는지 확인
- `diagnostics_flag_contract`: 내부 진단 command argument 계약 확인
- `cli_json_contract`: `export-structure` JSON schema·출력 확인
- `cli_exit_codes`: caption·structure 관련 usage/runtime code 확인
- `provenance_contract`, `batch_axes_contract`, `cli_catalog_contract`: 공유 helper 소비자 회귀 확인
- representative fixture의 이동 전후 SVG·structure JSON을 byte 또는 정규화 semantic 비교

### 6.2 CI 운영 계약

```bash
node --test scripts/tests/ci-impact-classifier.test.cjs
node --test scripts/tests/ci-impact-policy.test.cjs
python3 -m unittest scripts.tests.test_ci_impact_workflow
python3 -m unittest scripts.tests.test_codeql_workflow
python3 -m unittest scripts.tests.test_render_diff_workflow
python3 -m unittest scripts.tests.test_review_only_fast_pass_workflows
actionlint .github/workflows/render-diff.yml
```

### 6.3 Rust·제출 게이트

```bash
cargo nextest run --locked \
  --cargo-profile release-test \
  --target-dir target/pr-review \
  --tests --no-fail-fast
cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings
cargo fmt --all
cargo fmt --all -- --check
python3 scripts/check_markdown_links.py
git diff --check
```

착수 당시 사용한 `scripts/release-test.mjs` wrapper는 최신 `devel@5645e1f5b`에 존재하지 않는다. Stage 7은
현재 권위 문서인 `mydocs/manual/pr_review/local_validation.md`의 직접 nextest·clippy 명령으로 실행한다.
`--all-features` GPU 조합은 필수 clippy 게이트 밖의 추가 진단으로 구분한다.

`src/**`의 `#[cfg(test)]`를 변경하게 되면 추가로
`node scripts/rust-unit-test-tiers.mjs --check`를 실행한다. integration test case를 새로 추가하면 원본만
제출하고 review worktree에서 준비한 generated suite·manifest는 stage하지 않는다.

## 7. 단계별 산출물과 커밋

1. **계획 준비**: 수행·구현 계획과 기준선 — 현재 단계의 local commit
2. **source 경계 이동**: caption·structure module과 focused CLI contracts
3. **CI 경계 보정**: workflow·classifier·policy mirror와 운영 tests
4. **검증·보고**: Stage 보고, 최종 보고와 재현 명령

각 단계가 끝날 때 exact path만 stage해 독립 local commit으로 고정한다. 제품 구현, remote push, PR 생성은
각각 현재 승인 범위를 넘지 않도록 수행계획의 승인 게이트를 따른다.

## 8. rollback과 중단 조건

- source 이동 commit과 CI 경계 commit을 분리해 어느 한쪽의 계약 실패를 독립적으로 되돌릴 수 있게 한다.
- caption output 또는 structure JSON이 달라지면 해당 move commit을 고치기 전 다음 단계로 진행하지 않는다.
- `src/main.rs` 제거 뒤 render-positive path가 workflow 추적에서 빠지면 CI 변경을 완료로 보지 않는다.
- 실제 Render Diff consumer가 계획과 다르면 path 이름으로 추정하지 않고 workflow 실행 그래프를 다시 확인한다.
- 구현 중 renderer 알고리즘, CLI schema 또는 #3790 범위 변경이 필요해지면 별도 승인을 받는다.

## 9. 구현 결과

- caption 전용 파일명은 `src/cli/commands/caption_validation.rs`로 확정했다.
- structure JSON 단일 authority는 `src/cli/queries/structure.rs`로 옮겼다.
- 계획에 없던 `src/mcp_serve.rs` 소비자를 컴파일 단계에서 발견해 같은 authority를 참조하도록 보정했다.
- source 책임 이동은 `17fa14198`, CI 경계 보정은 `514ff74bc`로 각각 고정했다.
- classifier schema version은 새 경계 fixture와 함께 4에서 5로 올렸다.
- `src/main.rs`는 2,101줄에서 1,930줄로 줄었고 직접 `render_page_svg` 호출과 구조 JSON 생성 구현이 없다.
- renderer 알고리즘·출력 schema·golden baseline은 변경하지 않았다.
- 전체 로컬 결과와 계획 대비 차이는
  [최종 보고서](../../report/archives/task_m100_3789_report.md)에 기록한다.
- Stage 1~4 보고는 각 단계 전환 시점이 아니라 최종 검증 뒤 `3c509c7d1`에서 함께 작성됐다. 이 실제
  계보와 중간 승인 생략은 [절차 복구 피드백](../../feedback/task_m100_3789_hyper_waterfall_recovery.md)에
  기록하며, 원 구현 commit을 재작성하지 않는다.
- 기존 구현 SHA를 보존하기 위해 rebase 대신 current-base merge를 사용했다. merge commit은
  `39d6aa1dd`이며, 겹친 CI policy 파일은 #3789 render 경계와 #6205 duration-policy job 계약을 모두
  보존한다. 세부 focused 결과는 [Stage 5 보고](../../working/task_m100_3789_stage5.md)에 기록한다.
- Stage 5 뒤 다시 진전한 `upstream/devel@5645e1f5b`도 같은 이유로 `3db893274` current-base merge로
  반영했다. 자동 병합된 CI policy에서 #3789 경계와 upstream Archive D·duration-policy 계약을 모두
  보존했고, 세부 focused 결과는 [Stage 6 보고](../../working/task_m100_3789_stage6.md)에 기록한다.
- 작업지시자의 별도 승인 뒤 전체 nextest 8,473개와 필수 clippy를 통과했다. 제거된 wrapper와 추가
  `--all-features` 진단의 upstream GPU dependency 불일치는 필수 결과와 분리해
  [Stage 7 보고](../../working/task_m100_3789_stage7.md)에 기록한다.
- remote 제출 승인 뒤 다시 진전한 `upstream/devel@1a43a507c`를 `7c6ee5461` current-base merge로
  반영했다. 새 shaping 통합을 포함한 focused Rust 113개, 전체 nextest 8,519개와 필수 clippy를 통과했고
  [Stage 8 보고](../../working/task_m100_3789_stage8.md)에 기록한다.

## 10. PR 리뷰 뒤 계획 조정

이 절은 2026-08-29 PR 리뷰 뒤 추가한 사후 결정 기록이다. 위 1~8절의 최초 계획을 당시부터 현재 결정이었던
것처럼 고쳐 쓰지 않는다.

- 최초 계획의 “direct SVG caller인 caption source를 Render Diff가 추적한다”는 전제가 실제 workflow 실행
  그래프와 달랐다. Render Diff는 `test-caption`이나 `export-svg`를 실행하지 않고 `export-pdf` report를
  소비하며, Native Skia는 `export-png`를 소비한다.
- 따라서 path 이름이나 direct render 호출 여부가 아니라 **workflow direct consumer와 그 공유 입력**을 CI
  기준으로 삼는다. caption·vector는 일반 Rust, raster는 Native Skia, `document_io.rs`는 Render Diff와
  Native Skia 양쪽으로 분류한다.
- `main.rs`를 negative로 둘 수 있도록 문서 로더·인증 pre-scan과 단위 변환도 실제 소유 모듈로 이동한다.
  이 조정은 최초 계획 §8의 “render-positive path가 빠지면 완료로 보지 않는다” 중단 조건을 해소한다.
- direct CLI page renderer caller를 `src/cli/**/*.rs`에서 전수 발견하고 explicit bucket을 요구해 새 command
  디렉터리에도 같은 결정을 강제한다.
- 보정 source는 `eeffb3e8f`, 최신 `devel@f6a6bee8f` 병합은 `16ea38cd2`에 고정했다. focused Rust
  113개, Node 69개, Python 37개, 전체 runnable nextest 8,553개와 필수 clippy가 통과했다. 상세 결과는
  [Stage 9 보고](../../working/task_m100_3789_stage9.md)에 기록한다.
- `test-caption`의 모든 mutation이 실패해도 exit 0인 기존 동작은 이번 move-only 불변식과 충돌하므로 제품
  보정에 섞지 않는다. 별도 false-pass 이슈 후보로 유지하고 외부 이슈 생성은 별도 승인을 받는다.
