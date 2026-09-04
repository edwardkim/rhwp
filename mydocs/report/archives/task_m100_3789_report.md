# Task M100 #3789 완료 보고서

- **Issue**: [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **착수 기준**: `upstream/devel` `1b91c2025`
- **브랜치**: `task_m100_3789-render-boundary`
- **구현 완료일**: 2026-08-27 KST
- **최신 갱신일**: 2026-08-29 KST
- **상태**: PR #6276 리뷰 보정·최신 devel 로컬 검증 완료, 보정 candidate push 대기
- **절차 판정**: 기술 게이트 준수, 단계별 보고·승인 게이트 부분 미준수

## 결과

`src/main.rs`에 남아 있던 책임을 실제 소유 모듈로 분리했다. `test-caption`의 문서 mutation과 직접 SVG
render는 `src/cli/commands/caption_validation.rs`, structure export와 공유 JSON 변환은
`src/cli/queries/structure.rs`, CLI 문서 로더·인증 입력은 `src/cli/document_io.rs`, 단위 변환은
`src/cli/units.rs`가 소유한다. root에는 인자 해석과 dispatch·composition만 남았으며 2,101줄에서
1,716줄로 줄었다.

structure helper의 소비자는 계획 당시 확인한 vector export와 batch query 외에 MCP structure 응답도
있었다. 컴파일 단계에서 이를 확인해 같은 query authority를 참조하도록 보정했다. 새 root re-export나
중복 JSON helper는 만들지 않았다.

CI는 `src/main.rs` 전체를 renderer source로 보던 blanket 경계를 제거했다. PR 리뷰에서 direct renderer
caller 여부와 실제 workflow consumer가 다름을 확인해, Render Diff의 `export-pdf`와 Native Skia의
`export-png`가 공유하는 `src/cli/document_io.rs`를 workflow, trusted classifier와 policy mirror에 동시에
등록했다. root·caption·vector·structure·units는 일반 Rust, raster는 Native Skia, document input은 Render
Diff와 Native Skia 양쪽으로 분류된다. #5776이 고정한 PDF/shared/raster adapter mapping은 유지된다.

## 커밋 경계

| commit | 내용 |
| --- | --- |
| `fcaff2afd` | 수행·구현 계획과 기준선 |
| `17fa14198` | caption render와 structure query source 소유권 분리 |
| `514ff74bc` | Render Diff false-positive 경계와 CI 계약 보정 |
| `3c509c7d1` | Stage 1~4 사후 보고와 최종 검증 기록 |
| `212fa79a4` | 하이퍼 워터폴 절차 감사 보정 |
| `39d6aa1dd` | `upstream/devel@2166f4065` current-base merge |
| `a4d7023f7` | Stage 5 동시점 보고와 최신 기준 상태 기록 |
| `3db893274` | `upstream/devel@5645e1f5b` second current-base merge |
| `a76e88085` | Stage 6 second refresh 검증 기록 |
| `7a5781840` | Stage 7 전체 회귀 결과 기록 |
| `7c6ee5461` | `upstream/devel@1a43a507c` pre-push current-base merge |
| `764439a15` | Stage 8 제출 직전 검증 기록 |
| `dc0e3c5b5` | PR #6276 self-review trailing 기록 |
| `3e439a534` | PR update-branch merge와 원격 review head |
| `2357800d2` | `upstream/devel@96da78a9c` review correction 전 current-base merge |
| `eeffb3e8f` | 실제 workflow consumer 기준 CLI 렌더 입력 경계 보정 |
| `16ea38cd2` | `upstream/devel@f6a6bee8f` final local current-base merge |

## 계획 대비 실제

| 계획 | 실제 | 판정 |
| --- | --- | --- |
| caption 직접 render를 전용 command module로 이동 | `caption_validation.rs`로 move-only 분리 | 계획대로 |
| structure export/helper를 query module로 이동 | `structure.rs`를 단일 authority로 구성 | 계획대로 |
| root에는 composition만 유지 | renderer 호출·구조 JSON 구현 제거, 171줄 감소 | 계획대로 |
| vector와 batch 소비자를 새 authority로 연결 | 두 소비자와 추가 발견한 MCP 소비자까지 연결 | 계획 외 보정 |
| root negative, caption positive CI 분류 | 실제 consumer 대사 뒤 caption negative·공유 document input positive로 조정 | 리뷰 보정 |
| renderer 의미·출력 계약 보존 | 알고리즘·schema·golden 변경 없음, 계약·전체 회귀 통과 | 계획대로 |

## 검증 결과

### CLI와 소유 경계

- `issue_cli_test_caption_no_panic`: 1/1 통과
- `cli_json_contract`: 31/31 통과
- `mcp_session_structure_extract_contract`: 6/6 통과
- `provenance_contract`: 10/10 통과
- `batch_axes_contract`: 17/17 통과
- `diagnostics_flag_contract`: 15/15 통과
- `cli_exit_codes`: 13/13 통과
- `cli_catalog_contract`: 20/20 통과

### CI와 전체 회귀

- classifier·policy Node 계약: 69/69 통과
- Render Diff·CI impact workflow Python 계약: 37/37 통과
- `actionlint .github/workflows/render-diff.yml`: 통과
- release-test: runnable 8,553/8,553 통과, 43 ignored, 실패 0
- clippy `-D warnings`: 통과
- integration suite manifest 1,017 sources / 4,503 attrs / 48 targets와 source unit tier 정책 검사: 통과
- Cargo format과 `git diff --check`: 통과

## 시각·WASM 검증 판단

이번 변경은 renderer, paint/layout, PDF/SVG/raster 생성 알고리즘이나 WASM API를 바꾸지 않고 기존 direct
caller와 공유 입력의 소유 파일만 이동한다. golden baseline도 변경하지 않았다. 따라서 Native Skia capture,
WASM build와 시각 baseline 재생성은 로컬 추가 게이트에서 제외했다. 다만 실제 PDF·PNG workflow가 공유하는
`document_io.rs`가 앞으로 변경되면 CI classifier가 Render Diff와 Native Skia를 모두 활성화하도록 positive
계약을 고정했다. caption·vector는 현행 workflow가 직접 실행하지 않으므로 과분류하지 않는다.

## 최신 `devel` 재기준화

절차 보정 승인 뒤 `upstream/devel`을 다시 fetch했다. 작업 branch는 5커밋 ahead, 63커밋 behind였고 최신
기준은 `2166f4065`였다. 계획·보고 문서가 원 구현 SHA를 감사 증거로 참조하므로 rebase로 이력을 바꾸지
않고, `git merge-tree --write-tree HEAD upstream/devel`의 무충돌 결과를 확인한 뒤 `39d6aa1dd`로
current-base merge했다.

양쪽이 함께 바꾼 `scripts/ci-impact-policy.cjs`와 그 Node test는 서로 다른 hunk에서 자동 병합됐다.
#3789의 caption positive/main negative 경계와 #6205의 duration-policy audit job 등록이 모두 남아 있음을
확인했다.

### Stage 5 focused 재검증

- caption·structure·MCP·batch·exit·ownership Rust 계약: 113/113 통과
- classifier·policy Node 계약: 67/67 통과
- CI workflow Python 계약: 70/70 통과
- `actionlint .github/workflows/render-diff.yml`: 통과
- Cargo format: 통과
- integration suite manifest: 981 sources, 4,399 static test attrs 확인
- source unit tier: 4,221 tests, 299 modules 확인
- Markdown 603개 링크와 branch diff 검사: 통과

전체 release-test와 clippy는 최신 기준에서 아직 다시 실행하지 않았다. Stage 5 결과를 작업지시자에게
공유하고 승인받은 뒤 Stage 6에서 실행하기로 당시 분리했다. 초기 기준 `1b91c2025`에서 통과한 전체 결과와
혼동하지 않는다.

### Stage 6 second current-base refresh

Stage 5 보고 뒤 `upstream/devel`은 `5645e1f5b`까지 52커밋 더 진전했다. 병합 전 branch 관계는
`ahead 7 / behind 52`였다. 작업지시자의 재최신화 승인 뒤 dry merge tree가 충돌 없이 생성됨을 확인하고
`3db893274`로 current-base merge했다. 병합 뒤 관계는 `ahead 8 / behind 0`이다.

양쪽이 다시 함께 바꾼 `scripts/ci-impact-policy.cjs`는 자동 병합됐다. #3789의 정확한 caption render path와
`src/main.rs` negative 계약뿐 아니라 upstream의 Archive D job, duration-policy resolve·refresh job과 trusted
review 계약이 함께 남아 있다.

- caption·structure·MCP·batch·exit·ownership Rust 계약: 113/113 통과
- classifier·policy Node 계약: 67/67 통과
- CI workflow Python 계약: 71/71 통과
- `actionlint .github/workflows/render-diff.yml`, Cargo format: 통과
- integration suite manifest: 987 sources, 4,423 static test attrs, nextest 최소 6,559 cases 확인
- source unit tier: 4,221 tests, 299 modules 확인
- Markdown 604개 링크와 branch diff 검사: 통과

이 결과는 [Stage 6 보고](../../working/task_m100_3789_stage6.md)에 동시점 기록했다. 최신 기준의 전체
release-test와 clippy는 아직 실행하지 않았으며, Stage 7 별도 승인 뒤 수행한다. 초기 기준과 Stage 5의
전체·focused 결과를 최신 기준 결과로 간주하지 않는다.

### Stage 7 full regression

작업지시자의 별도 승인 뒤 최신 기준에서 현재 권위 문서의 전체 회귀를 실행했다.

- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests
  --no-fail-fast`: 8,473/8,473 통과, 43 skip, 10 slow, 실패 0
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`: 통과

착수 계획에 있던 `scripts/release-test.mjs`는 최신 upstream에서 제거돼 호출 즉시 `MODULE_NOT_FOUND`로
종료됐다. 테스트 실패로 계산하지 않고 현재 canonical 직접 nextest 명령으로 대체했다. 필수 범위를 넘는
`--workspace --all-targets --all-features` 추가 clippy 진단은 `vello 0.9/0.10`의 `Scene` 타입 불일치로
컴파일되지 않았다. #3789 branch는 관련 `Cargo.toml`, `Cargo.lock`, `src/renderer/gpu.rs`를 변경하지 않으며,
현재 권위 문서의 필수 clippy 범위는 통과했다. 상세 명령과 판정은
[Stage 7 보고](../../working/task_m100_3789_stage7.md)에 기록한다.

### Stage 8 pre-push refresh

remote push와 PR 생성 승인을 받은 직후 fetch에서 `upstream/devel`이 `1a43a507c`까지 15커밋 진전한 것을
확인했다. 이 변경은 #4969의 렌더 shaping 통합으로 83개 파일을 바꾸지만 #3789 경계와 dry merge tree는
충돌하지 않았다. `7c6ee5461` current-base merge 뒤 branch 관계는 `ahead 11 / behind 0`이다.

- #3789 focused Rust: 113/113 통과
- classifier·policy Node: 67/67 통과
- CI workflow Python: 71/71 통과
- 전체 nextest: 8,519/8,519 통과, 43 skip, 5 slow, 실패 0
- 필수 clippy, actionlint, format, manifest, unit-tier, Markdown 링크와 diff: 통과
- manifest: 995 sources, 4,469 static test attrs, 48/48 integration targets

세부 내용은 [Stage 8 보고](../../working/task_m100_3789_stage8.md)에 기록한다. 이 보고 commit까지를 최초
원격 code candidate로 제출하고, PR 번호 기반 self-review·오늘할일은 후속 review-only commit으로 추가한다.

### Stage 9 PR review correction

[PR 리뷰 comment](https://github.com/edwardkim/rhwp/pull/6276#issuecomment-5452147207)는 최초 CI 경계가
실제 workflow consumer와 어긋나고 `main.rs`에도 PDF·PNG 입력 helper가 남아 있음을 지적했다. 작업지시자의
보정 승인 뒤 다음을 반영했다.

- 문서 로더·오류 분류·인증 pre-scan을 `src/cli/document_io.rs`, 단위 변환을 `src/cli/units.rs`로 이동
- Render Diff·Native Skia 공유 입력을 `document_io.rs`로 등록하고, 미소비 caption trigger 제거
- root의 모든 `.render_page_` family를 막고 `src/cli/**/*.rs` direct renderer caller를 explicit bucket으로
  전수 강제
- rustfmt 중괄호에 결합된 dispatch assertion과 `vector.rs` module doc 보정

리뷰 당시 원격 head `3e439a534` 뒤 `upstream/devel@96da78a9c`와 `@f6a6bee8f`를 각각
`2357800d2`, `16ea38cd2` current-base merge로 반영했다. 최종 로컬 결과는 다음과 같다.

- #3789 focused Rust: 113/113 통과
- classifier·policy Node: 69/69 통과
- Render Diff·CI impact workflow Python: 37/37 통과
- 전체 nextest: runnable 8,553/8,553 통과, 43 ignored, 실패 0
- 필수 clippy, actionlint, Cargo format, diff, manifest, unit-tier: 통과
- manifest: 1,017 sources, 4,503 static test attrs, 48/48 integration targets

`test-caption`의 모든 mutation이 실패해도 exit 0인 기존 동작은 이번 책임 이동과 별개인 false-pass 문제로
남긴다. 별도 이슈 후보이며 외부 이슈 생성은 별도 승인을 받는다. 세부 판단과 재현 명령은
[Stage 9 보고](../../working/task_m100_3789_stage9.md)에 기록한다.

## 하이퍼 워터폴 절차 감사

2026-08-27 작업지시자의 요청으로 canonical 절차와 실제 commit 계보를 대사했다.

| 게이트 | 실제 | 판정 |
| --- | --- | --- |
| 이슈·담당자·착수 잠금 | #3789 담당자와 착수 comment를 구현 전에 고정 | 준수 |
| 최신 기준 branch | 착수 당시 `upstream/devel@1b91c2025`에서 생성 | 준수 |
| 수행·구현 계획 | 구현 전 `fcaff2afd`로 작성·commit | 준수 |
| 계획 승인 | 작업지시자의 `진행해줘` 뒤 구현 착수 | 준수 |
| source·CI 단계 commit | `17fa14198`, `514ff74bc`로 분리 | 준수 |
| 단계별 완료 보고 | Stage 1~4 문서를 최종 검증 뒤 `3c509c7d1`에서 함께 작성 | 미준수 |
| 단계별 작업지시자 승인 | Stage 2·3 종료와 Stage 4 진입 사이 별도 승인 없음 | 미준수 |
| 로컬 검증·제출 경계 | 필수 검증 통과, remote push·PR 미수행 | 준수 |

작업지시자는 감사 결과를 확인한 뒤 사후 보정을 승인했다. Stage 문서에 사후 작성 사실을 명시하고
[절차 복구 피드백](../../feedback/task_m100_3789_hyper_waterfall_recovery.md)을 추가한다. 원 구현 commit을
재작성하거나 과거 승인 이력을 만든 것처럼 표현하지 않는다. 이 보정은 감사 가능성을 회복하지만 생략된
중간 승인 게이트를 소급해 완전 준수로 바꾸지 않는다.

## 제출 상태

로컬 구현과 필수 검증은 완료했고 generated integration suite·manifest는 제출 대상에 포함하지 않았다.
Open PR [#6276](https://github.com/edwardkim/rhwp/pull/6276)은 `devel` 대상 Draft다. 현재 원격 head는
`3e439a534`이며 그 head의 GitHub Actions는 성공했지만, 리뷰 보정이 반영된 로컬 candidate는
`16ea38cd2`와 이 보고의 후속 문서 commit이므로 원격 CI 증적으로 재사용하지 않는다.

[self-review](../../pr/archives/pr_6276_review.md)를 리뷰 보정 기준으로 갱신했다. 작업지시자의 별도 push 승인 뒤
보정 candidate를 원격에 반영하고, 최신 head Full CI·mergeability를 확인한 다음 보정 완료 comment를
게시한다. `test-caption` false-pass 별도 이슈 생성과 PR merge도 각각 별도 승인 게이트로 남긴다.
