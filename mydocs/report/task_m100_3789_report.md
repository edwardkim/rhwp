# Task M100 #3789 완료 보고서

- **Issue**: [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **착수 기준**: `upstream/devel` `1b91c2025`
- **브랜치**: `task_m100_3789-render-boundary`
- **구현 완료일**: 2026-08-27 KST
- **최신 갱신일**: 2026-08-28 KST
- **상태**: 최신 `devel` 재최신화·focused 검증 완료, Stage 7 전체 회귀 승인 대기
- **절차 판정**: 기술 게이트 준수, 단계별 보고·승인 게이트 부분 미준수

## 결과

`src/main.rs`에 남아 있던 두 종류의 책임을 실제 소유 모듈로 분리했다. `test-caption`의 문서 mutation과
직접 SVG render는 `src/cli/commands/caption_validation.rs`, structure export와 공유 JSON 변환은
`src/cli/queries/structure.rs`가 소유한다. root에는 인자 해석과 dispatch만 남았으며 2,101줄에서
1,930줄로 줄었다.

structure helper의 소비자는 계획 당시 확인한 vector export와 batch query 외에 MCP structure 응답도
있었다. 컴파일 단계에서 이를 확인해 같은 query authority를 참조하도록 보정했다. 새 root re-export나
중복 JSON helper는 만들지 않았다.

CI는 `src/main.rs` 전체를 renderer source로 보던 blanket 경계를 제거했다. 대신 직접
`render_page_svg`를 호출하는 caption module의 정확한 경로를 Render Diff workflow, trusted classifier와
policy mirror에 동시에 등록했다. 그 결과 root와 structure query는 일반 Rust 변경으로, caption render는
Render Diff와 Native Skia가 필요한 변경으로 분류된다. #5776이 고정한 PDF/shared/raster adapter mapping은
그대로 유지된다.

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

## 계획 대비 실제

| 계획 | 실제 | 판정 |
| --- | --- | --- |
| caption 직접 render를 전용 command module로 이동 | `caption_validation.rs`로 move-only 분리 | 계획대로 |
| structure export/helper를 query module로 이동 | `structure.rs`를 단일 authority로 구성 | 계획대로 |
| root에는 composition만 유지 | renderer 호출·구조 JSON 구현 제거, 171줄 감소 | 계획대로 |
| vector와 batch 소비자를 새 authority로 연결 | 두 소비자와 추가 발견한 MCP 소비자까지 연결 | 계획 외 보정 |
| root negative, caption positive CI 분류 | workflow/classifier/policy와 fixture에 동일 반영 | 계획대로 |
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

- classifier·policy Node 계약: 67/67 통과
- CI workflow Python 계약: 68/68 통과
- `actionlint .github/workflows/render-diff.yml`: 통과
- release-test: 8,402/8,402 통과, 43 skip, 실패 0
- clippy `-D warnings`: 통과
- integration suite manifest와 source unit tier 정책 검사: 통과
- Cargo format과 `git diff --check`: 통과

## 시각·WASM 검증 판단

이번 변경은 renderer, paint/layout, PDF/SVG/raster 생성 알고리즘이나 WASM API를 바꾸지 않고 기존 direct
caller의 소유 파일만 이동한다. golden baseline도 변경하지 않았다. 따라서 Native Skia capture, WASM
build와 시각 baseline 재생성은 로컬 추가 게이트에서 제외했다. 다만 새 caption 파일이 앞으로 변경되면
CI classifier가 Render Diff와 Native Skia를 모두 활성화하도록 positive 계약을 고정했다.

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

이 결과는 [Stage 6 보고](../working/task_m100_3789_stage6.md)에 동시점 기록했다. 최신 기준의 전체
release-test와 clippy는 아직 실행하지 않았으며, Stage 7 별도 승인 뒤 수행한다. 초기 기준과 Stage 5의
전체·focused 결과를 최신 기준 결과로 간주하지 않는다.

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
[절차 복구 피드백](../feedback/task_m100_3789_hyper_waterfall_recovery.md)을 추가한다. 원 구현 commit을
재작성하거나 과거 승인 이력을 만든 것처럼 표현하지 않는다. 이 보정은 감사 가능성을 회복하지만 생략된
중간 승인 게이트를 소급해 완전 준수로 바꾸지 않는다.

## 제출 상태

로컬 구현과 필수 검증은 완료했다. generated integration suite·manifest는 제출 대상에 포함하지 않았다.
최신 기준은 다시 `upstream/devel@5645e1f5b`로 갱신됐고 Stage 6 focused 검증까지 완료했다. Stage 7 전체
release-test·clippy, remote push, PR 생성과 실제 PR CI는 각각 필요한 작업지시자 승인 전까지 남아 있다.
