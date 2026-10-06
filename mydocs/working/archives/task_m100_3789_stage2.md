# Stage 2 사후 감사 보고 — Task M100 #3789: source 책임 이동

- **일자**: 2026-08-27 KST
- **브랜치**: `task_m100_3789-render-boundary`
- **구현 commit**: `17fa14198`
- **이슈**: [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **보고 작성 commit**: `3c509c7d1`
- **문서 성격**: Stage 3 진입 뒤 실제 결과를 대사해 작성한 사후 보고

## 구현 결과

- `test-caption` 구현을 `src/cli/commands/caption_validation.rs`로 옮기고 root에는 dispatch만 남겼다.
- `export_structure`와 `structure_json_value`를 `src/cli/queries/structure.rs`로 옮겼다.
- vector output, batch query와 컴파일 중 확인한 MCP structure 응답을 새 단일 authority로 연결했다.
- `src/main.rs`는 2,101줄에서 1,930줄로 줄었고 직접 renderer 호출과 구조 JSON 구현이 제거됐다.
- `cli_catalog_contract`에 소유 경계 회귀를 추가해 root·vector로 구현이 되돌아오는 것을 막았다.

## focused 검증

| 계약 | 결과 |
| --- | ---: |
| `issue_cli_test_caption_no_panic` | 1/1 통과 |
| `cli_json_contract` | 31/31 통과 |
| `mcp_session_structure_extract_contract` | 6/6 통과 |
| `provenance_contract` | 10/10 통과 |
| `batch_axes_contract` | 17/17 통과 |
| `diagnostics_flag_contract` | 15/15 통과 |
| `cli_exit_codes` | 13/13 통과 |
| `cli_catalog_contract` | 20/20 통과 |

integration suite는 review용 `--prepare`로 생성해 실행했으며 generated suite·manifest는 Git 제출 대상에
포함하지 않았다. 공개 CLI 결과를 만드는 코드는 move-only로 유지했고 renderer 알고리즘·JSON schema·
golden baseline은 바꾸지 않았다.

## 종료 판단

root composition 경계와 caption/structure 소유권은 `17fa14198`과 focused test로 고정됐다. 그러나 이
결과를 Stage 2 보고로 작성해 작업지시자 승인을 받은 뒤 Stage 3에 진입하지는 않았다. 실제로는 CI 변경
commit `514ff74bc`까지 진행한 뒤 이 문서를 `3c509c7d1`에서 사후 작성했다.
