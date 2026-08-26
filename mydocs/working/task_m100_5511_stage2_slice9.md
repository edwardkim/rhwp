# #5511 Stage 2 아홉 번째 수직 절편 — 문서 필드 inventory query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `3751cb41be5839885cff62eb355200aafec2e10f`
- 구현 커밋: `97a6c02fd`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정과 경계 판단

아홉 번째 이동 대상으로 `fields`를 선택했다. 이 명령은 문서의 누름틀·필드를 이름, 안내문,
현재 값, 편집 가능 여부와 중첩 좌표까지 열거하는 read-only query다. JSON·사람용 출력,
필드가 없는 문서, 표 셀·글상자 중첩 위치, 오류 exit code와 Unicode 보안 신호까지 기존
집중 계약이 보호하고 있어 move-only 절편으로 적합했다.

`fields`는 첫 절편에서 만든 `document_inventory`의 책임과 같다. 별도 단일-handler 파일을
늘리지 않고 220줄이던 해당 모듈에 합쳐도 289줄이므로 1,200줄 모듈 상한에 충분한 여유가
있다. 절편 시작·종료 시 활성 PR 중 `src/main.rs`, `src/cli/queries/document_inventory.rs`,
`tests/fields_json_contract.rs`, `tests/cli_catalog_contract.rs`와 이 작업 보고서에 겹치는 변경은
없었다.

## 2. 구현 결과와 보호 불변식

- `src/cli/queries/document_inventory.rs`가 `show_fields` 전체를 소유한다.
- `src/main.rs`의 최상위 match는 document inventory query 모듈 API만 호출한다.
- 공개 함수 표식만 정규화한 기계 비교에서 handler 본문이 이동 전과 일치했다.
- `run_edit` rustdoc에 잘못 붙어 있던 `fields` 제목을 실제 handler와 함께 이동했다.
- `cli_catalog_contract`의 document inventory 소유권 표에 `fields`를 추가했다.
- 옵션 파싱, 중복 위치 인자의 현행 마지막 값 우선 동작, exit code, stdout/stderr 분리와
  JSON schema를 바꾸지 않았다.

`collect_field_records`와 `fields_json_value`는 단건 CLI뿐 아니라 batch와 MCP session
조회도 소비한다. 이를 CLI 하위로 옮기면 MCP가 CLI에 역의존하므로 Stage 2에서는 crate
root에 보존했다. 공유 field query와 envelope를 application/service 경계로 내리는 작업은
Stage 3에서 다룬다.

## 3. 지표 변화

| 항목 | Stage 2 절편 8 | Stage 2 절편 9 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 40,535 | 40,468 | -67 |
| `src/cli/queries/document_inventory.rs` | 220 | 289 | +69, 모듈 상한 이하 |
| `main.rs` 최상위 함수 | 330 | 329 | handler 1개 이동 |
| 누적 이동 read-only handler | 12 | 13 | 1개 추가 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| `main.rs` 최대 CC | 68 | 68 | `dump_controls`, 변화 없음 |
| document inventory 모듈 CC>25 함수 | 0 | 0 | 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 63 | 63 | 변화 없음 |
| `rhwp::parser` 직접 참조 | 89 | 89 | 변화 없음 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

`show_fields`는 CC 25 이하라 복잡도 경고 수치가 변하지 않았다. 이번 절편도 공용 helper를
root에 둔 binary-local seam을 명시적으로 유지하며 service 이행과 동작 변경을 섞지 않았다.

## 4. 외부 동작 동등성

여덟 번째 절편 통합 바이너리와 이동 후 바이너리에 대해 다음 여덟 경로의 exit code와
stdout/stderr SHA-256을 비교했다.

1. 필드가 있는 문서의 사람용 출력
2. 필드가 있는 문서의 JSON 출력
3. memo 지시문이 있는 문서의 JSON 출력
4. 필드가 없는 문서의 빈 JSON 목록
5. 존재하지 않는 파일
6. 필수 인자 누락
7. 알 수 없는 옵션
8. 위치 인자 두 개의 현행 마지막 입력 우선 동작

여덟 경로 모두 byte 단위로 일치했다. 대표적으로 사람용 출력은 399 bytes,
SHA-256 `8a7eaa7355fc25a94952c1bf4506e3d31b1f0315cacb9f9f918d2a2e701fa71f`, JSON
출력은 3,481 bytes,
SHA-256 `ebe59a02fb8ae6d4584562f3ebf4d4d076d4575b4a31f21ffbc25602bed769ac`다. 필드
0건은 exit 0과 빈 `fields` 목록을 유지했고 오류 경로는 stdout이 비어 있으며 기존 exit
1/2와 stderr hash가 일치했다.

중복 위치 인자를 마지막 값으로 덮어쓰는 현행 동작은 승인한 UX가 아니라 이동 동등성의
기준선이다. 엄격한 인자 검증은 별도 동작 변경 이슈로 분리해야 한다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 전·후 `fields_json_contract` focused nextest | 각각 13/13 통과 |
| `cli_catalog_contract` | 12/12 통과 |
| 정상·빈 문서·오류·현행 호환 출력 hash equivalence | 8/8 일치 |
| release-test 전체 nextest | 재배치 전 7,317/7,317, 최종 7,322/7,322 통과, 3 slow, 38 skipped, 최종 150.929초 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 2 ignored |
| Rust test suite manifest | 통과, 718 sources / 3,280 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| `git diff --check` | 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 최종 기준선의 30/30 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. parser·필드 수집·조판 로직은 바꾸지 않고 adapter 위치만 옮겼으므로 시각
검증과 WASM 빌드는 추가하지 않았다.

최종 검증 직후 `upstream/devel`이 #4161의 기본 장평 정합 변경을 포함해 10커밋 전진했다.
파일 교집합이 없음을 확인하고 별도 브랜치 없이 현재 작업 브랜치를 최신 기준선에 직접
재배치했다. 새 test source 때문에 발생한 generated harness drift를 `--prepare`로 해소했고,
추적 파일 변경 없이 새 계약 5건을 포함한 전체 7,322건과 all-target clippy를 다시 통과했다.
그 뒤 추가 유입된 #5536은 CI workflow와 전용 Python 계약만 변경해 파일 교집합이 없었다.
최신 기준선에 다시 직접 재배치하고 해당 Python 계약 30건을 모두 통과했다. Rust 소스와
생성 test manifest는 바뀌지 않아 직전 7,322건 결과를 그대로 유효한 최종 Rust 근거로 삼았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건 때문에 전체 exit는 실패했다. 이번 절편이 추가한 보고서 링크 오류는 없다.

## 6. 다음 절편 관문

다음 절편은 남은 read-only query 중 공용 helper 소비 방향과 기존 계약 밀도가 명확한
최소 단위를 최신 `upstream/devel`에서 다시 선정한다. `extract-data`는 handler가 약 160줄이고
JSON helper를 batch와 공유해 이번 `fields`와 비슷한 구조지만, 출력 파일·kind·limit 계약을
모두 재대조한 뒤에만 후보로 확정한다.

`dump-pages`는 #5525에서 확인된 계약 드리프트를 먼저 해소해야 하며 `dump_controls`는
move-only 대상이 아니다. 다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도
별도 승인 전 수행하지 않는다.
