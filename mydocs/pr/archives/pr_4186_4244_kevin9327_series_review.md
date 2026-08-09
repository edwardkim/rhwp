# kevin9327 연작 통합 검토 - #4186~#4244

## 기준과 범위

- 검토 브랜치: `review/kevin9327-20260808`
- 검토 시작 `upstream/devel`: `c119e5db5`, PR 생성 전 최신 기준선: `fcc3b2135`
- Full CI 검증 head: `cbfb07de3` (최신 기준선 rebase와 검토 기록 반영 후)
- 변경 규모: 85개 파일, +11,733/-24줄
- #4185는 기여자 CI 워크플로 추가 PR이므로 작업지시자 지시에 따라 **통합 대상에서 제외**했다.
  뒤의 `.github/` 변경은 #4254에 따른 메인터너 fast-pass 보정이며, #4185의 변경을 포함하지 않는다.

## 대상 PR과 누적 순서

| PR | source head | 범위 | 판단 |
|---|---|---|---|
| #4186 | `2351af2c` | `verify` 게이트, Node 래퍼, MCP 배선 | 수용 |
| #4192 | `49e179b5` | `verify` 수치 축 확장, #4186 후속 | 수용 |
| #4193 | `308d317f` | provenance `recordFields` 계약 | 수용 |
| #4194 | `231a949e` | 지식지도 사전 드리프트 방지 | 수용, 재시도 빈 커밋은 생략 |
| #4195 | `8cab15b0` | Python 생성 모델 드리프트 시험 | 수용 |
| #4196 | `03093287` | Python CLI 래퍼 파리티 | 수용 |
| #4198 | `78c9b7e4` | 관측성 계약 | 수용 |
| #4200 | `b324dd88` | 편집 전후 render-diff 게이트 | 수용 |
| #4202 | `d1c8f247` | 생태계 지표 기준선 | 수용 |
| #4204 | `1909abb4` | 규모 사다리 측정 도구 | 수용 |
| #4206 | `dadd7ee2` | rmcp 교환비 결정 기록 | 수용 |
| #4209 | `7b0ed985` | 스키마 버전 레지스트리 계약 | 수용 |
| #4211 | `6c1e807d` | 스킬군 로드맵 | 수용 |
| #4213 | `3ace3ba8` | 양식·표·트리아지 스킬 | 수용 |
| #4215 | `c4145eee` | 안전 편집·MCP·provenance 스킬 | 수용 |
| #4217 | `6da6ccae` | `scan` 코퍼스 발견 명령 | 수용 |
| #4219 | `a44dacf6` | 보안·대량·시각 회귀 스킬 | 수용 |
| #4221 | `77e70f20` | 2026H2 동향 조사 | 수용 |
| #4223 | `85e56e45` | 매뉴얼 드리프트 정정 | 수용 |
| #4226 | `24d90689` | MCP annotations 유도 | 수용 |
| #4231 | `6c970c94` | 코드 유도 온톨로지 설계 | 수용 |
| #4233 | `ea216299` | MCP 스펙 대장·계약 | 수용 |
| #4235 | `e70f4f14` | 실패 수복 `nextCall` | 수용 |
| #4237 | `4486f515` | 에이전트 벤치마크 | 수용 |
| #4239 | `1f177a10` | `export-ontology`, Python 표면 | 수용 |
| #4242 | `10937c2c` | provenance 체인 도구 | 수용 |
| #4244 | `c711fb0f` | 읽기 전용 포크 수확 도구 | 수용 |

## 통합 보정

두 문서 충돌(#4233)에서는 관측성 계약과 MCP 대장 링크를 모두 보존했다. Python 바인딩 충돌(#4239)에서는
`scan`, `explain`, `export_plan_schema`, `export_agent_manifest`, `export_ontology` 공개 API를 함께 유지했다.

통합 뒤 실제 계약이 잡은 병합 보류 사유는 다음 메인터너 보정으로 해소했다.

| 커밋 | 보정 | 근거 |
|---|---|---|
| `fa274cb3` | 지식지도 사전에 6개 실제 출력 필드와 개수를 반영 | `knowledge_map_field_dictionary_contract` 실패 |
| `6e1686eb` | Node 생성 봉투 타입 수 35→37 정합 | `npm run gen:check` 드리프트 |
| `7d2b7d16` | 벤치 문서의 존재하지 않는 태스크 ID를 실제 ID로 수정 | reference 솔루션 실실행 |
| `374e14f7` | trailing `mydocs/**` commit에서 Node/Python binding 재실행을 막는 fail-closed preflight 추가 | #4254, workflow Full CI 통과 |

제목에 `fix(ci)`가 있는 네 커밋은 GitHub Actions 변경이 아니라 제품 코드의 프로필·Rustfmt·CodeQL
오탐 정합이다. 기여자 CI 설정 변경은 #4185만 해당하며 제외했고, `374e14f7`은 이 통합 PR에서
별도로 검증한 메인터너 보정이다.

## 완료한 검증

| 검증 | 결과 |
|---|---|
| `git diff --check`, `cargo fmt --check` | 통과 |
| `cargo test --profile release-test --tests` | 종료코드 0 |
| `cargo clippy --all-targets -- -D warnings` | 종료코드 0 |
| 새 Rust 계약 11개 | 74개 테스트 통과 |
| Python `pytest bindings/python/tests` | 267 passed |
| Python `ruff`, `mypy` | 통과 |
| Node `typecheck`, `test`, `gen:check`, `build`, `npm pack --dry-run` | 420개 테스트 포함 통과 |
| Node production `npm audit --omit=dev` | 취약점 0건 |
| 실제 CLI | `verify`, `scan`, `export-ontology` 정상 출력 |
| 새 도구 실실행 | agent bench, provenance chain, scale ladder, render gate, fork harvest 정상 |
| GitHub Full CI | [CI 31248491466](https://github.com/edwardkim/rhwp/actions/runs/31248491466), [CodeQL 31248491386](https://github.com/edwardkim/rhwp/actions/runs/31248491386), [Render Diff 31248491377](https://github.com/edwardkim/rhwp/actions/runs/31248491377), Node/Python binding 모두 `cbfb07de3`에서 통과 |
| #4254 workflow 보정 Full CI | [CI 31249497089](https://github.com/edwardkim/rhwp/actions/runs/31249497089), [CodeQL 31249496988](https://github.com/edwardkim/rhwp/actions/runs/31249496988), [Render Diff 31249496981](https://github.com/edwardkim/rhwp/actions/runs/31249496981), [Node binding 31249496995](https://github.com/edwardkim/rhwp/actions/runs/31249496995), [Python binding 31249497087](https://github.com/edwardkim/rhwp/actions/runs/31249497087)가 `374e14f72`에서 통과 |
| #4254 trailing docs 재현 | [Node binding 31250118443](https://github.com/edwardkim/rhwp/actions/runs/31250118443), [Python binding 31250118444](https://github.com/edwardkim/rhwp/actions/runs/31250118444)가 `2fdf5c66d`에서 성공했고 unit·integration·package/wheel·generated-types job은 모두 skip |

`npm ci`의 개발 의존성 감사에는 기존 low/high 항목 2건이 남았으나 lockfile을 이 연작이 변경하지 않았고,
production 의존성 감사는 0건이었다.

## 최종 권고

**#4185를 제외한 27개 PR은 메인터너 보정 포함 수용 권고**다. 통합
[PR #4253](https://github.com/edwardkim/rhwp/pull/4253)는 #4254 보정 후 `374e14f72`에서 Full CI를
통과했고 `mergeStateStatus=CLEAN`을 확인했다. 이어 `2fdf5c66d`의 trailing 문서 commit에서
Node/Python binding까지 비용 큰 job을 재실행하지 않고 성공함을 확인했다. 이 최종 기록 갱신은
코드 후보를 바꾸지 않는 trailing 문서 commit으로 반영하며, 최신 head의 docs-only gate와
작업지시자 승인 뒤 merge한다.
