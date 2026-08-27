# Task M100 #3789 완료 보고서

- **Issue**: [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **기준**: `upstream/devel` `1b91c2025`
- **브랜치**: `task_m100_3789-render-boundary`
- **완료일**: 2026-08-27 KST
- **상태**: 로컬 구현·검증 완료, remote push·PR 승인 대기

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

## 제출 상태

로컬 구현과 필수 검증은 완료했다. generated integration suite·manifest는 제출 대상에 포함하지 않았다.
remote push, PR 생성과 실제 PR CI는 작업지시자의 별도 승인 전까지 남아 있다.
