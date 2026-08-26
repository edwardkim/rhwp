# #5511 Stage 2 열두 번째 수직 절편 — 결정론적 explain query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 구현 커밋: `96ed8f2d0`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정과 경계 판단

열두 번째 이동 대상으로 `explain`을 선택했다. 이 명령은 처음 보는 문서의 형식·쪽·문단·표·
누름틀·각주·미주·암호 여부를 기존 query 결과로 조립하는 read-only query다. HWP5·HWPX·HWP3,
사람용·JSON 출력, 표와 필드, 각주와 미주, 암호 및 오류 경로를 전용 계약이 보호하므로
move-only 절편으로 적합했다.

`explain_document`와 전용 helper 네 개는 모두 cognitive complexity 25 이하이고 새 모듈은
240줄이라 CC>25=0과 1,200줄 상한을 지킨다. 후보였던 `explore`는 security detector,
structure·chart·field·table query와 MCP tool registry를 함께 소비해 소유 경계를 더 조사해야 하므로
이번 절편에 섞지 않았다. `show_info`는 CC 34, `dump_controls`는 CC 68이고 `dump-pages`는 #5525의
계약 drift가 남아 있어 단순 이동 대상에서 계속 제외했다.

절편 시작·종료 시 `upstream/devel`은 `1a6ce79fd`로 동일했다. 활성 PR 중 `src/main.rs`,
`src/cli/queries/explain.rs`, query module index, explain·CLI catalog 계약과 이 보고서에 겹치는
변경은 없었다.

## 2. 구현 결과와 보호 불변식

- 새 `src/cli/queries/explain.rs`가 `explain_document`와 explain 전용 helper 네 개를 소유한다.
- `src/main.rs`의 최상위 match는 explain query 모듈 API만 호출한다.
- 공개 함수 표식만 정규화한 기계 비교에서 이동 블록이 byte 단위로 일치했다.
- `cli_catalog_contract`가 handler·전용 helper 소유권, root 재유입 금지와 dispatch를 고정한다.
- 사람용 문장, JSON schema·provenance, 표·필드·각주·미주 집계, format label, exit code와
  stdout/stderr 분리를 바꾸지 않았다.

`collect_field_records`는 `fields`, batch와 MCP도 함께 소비한다. 이를 explain CLI 하위로 옮기면
다른 adapter가 explain에 역의존하므로 crate root에 보존했다. 공유 field query와 envelope를
application/service 경계로 내리는 일은 Stage 3에서 다룬다.

## 3. 지표 변화

| 항목 | Stage 2 절편 11 | Stage 2 절편 12 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 39,995 | 39,763 | -232 |
| `src/cli/queries/explain.rs` | 없음 | 240 | 신규, 모듈 상한 이하 |
| `main.rs` plain 최상위 함수 | 326 | 321 | handler·전용 helper 4개 이동 |
| 누적 이동 read-only handler | 15 | 16 | 1개 추가 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| `main.rs` 최대 CC | 68 | 68 | 변화 없음 |
| explain 모듈 CC>25 함수 | 없음 | 0 | 새 경계 내 초과 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 63 | 63 | 변화 없음 |
| `rhwp::parser` 직접 참조 | 88 | 80 | detect call·FileFormat variant 참조 이동 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

이번 절편은 explain의 format detection과 label mapping까지 책임 경계 안에 두되 공유 field helper는
root에 남겼다. 따라서 복잡도를 다른 파일로 숨기지 않았고 service 이행과 관찰 가능한 동작 변경도
섞지 않았다.

## 4. 외부 동작 동등성

열한 번째 절편 통합 바이너리와 이동 후 바이너리에 대해 다음 열다섯 경로의 exit code와
stdout/stderr byte 수·SHA-256을 비교했다.

1. 필드 문서 사람용 요약
2. 필드 문서 JSON 봉투
3. 병합 셀이 있는 표 문서 JSON
4. 표·필드가 없는 일반 문서 JSON
5. 각주 문서 JSON
6. 미주 문서 JSON
7. HWPX format label 경로
8. HWP3 format label 경로
9. 존재하지 않는 파일
10. 필수 경로 누락
11. 위치 인자 초과
12. 알 수 없는 옵션
13. 암호 문서에 비밀번호 없음
14. 암호 문서에 잘못된 비밀번호
15. 암호 문서에 올바른 비밀번호

열다섯 경로 모두 byte 단위로 일치했다. 사람용 요약 284 bytes의 SHA-256은
`b4ab885d1c6a6f8e066a028056c14a879975028ed0f5819ebe2364fd0632b0fb`, 필드 JSON
669 bytes는 `e906e7eb4b49dbd66f9185cd1927159fef1964d9567a15279dc2af2e200bb988`, HWP3
JSON 7,203 bytes는 `39c5091f369c03ee8dc933abcb2e0a03097c8edc49d5d6c05e3cd28d1bd5892d`다.
올바른 비밀번호 경로 7,221 bytes의 hash는
`d7307c7bfc663875609c71ce3907c8d72bd95fdddad8df6890cd63d1a8054ce9`로 일치했다.
오류 경로는 stdout이 비어 있으며 기존 exit 1/2와 stderr hash를 그대로 유지했다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 전·후 explain·agent ladder focused nextest | 각각 21/21 통과 |
| `cli_catalog_contract` | 이동 전 14/14, 이동 후 15/15 통과 |
| 사람용·JSON·format·암호·오류 출력 hash equivalence | 15/15 일치 |
| release-test 전체 nextest | 7,745/7,745 통과, 3 slow, 38 skipped, 173.508초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| Rust test suite manifest | 통과, 750 sources / 3,696 static test attrs / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| test policy 자체 계약 | integration 16/16, unit tier 12/12 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. Cargo가 최신 원격에서 이름이 인접한 두 verifier package의 lockfile 순서만
재정렬했으나 #5511과 무관한 파생 변화이므로 추적 변경에서 복원했다. parser·구조 추론·조판
로직은 바꾸지 않고 adapter 위치만 옮겼으므로 시각 검증과 WASM 빌드는 추가하지 않았다.

Markdown 검사는 기준선에도 존재하는 `agent_capability_registry.md`의 중복 ID·진입점 링크
무결성 오류 16건을 보고했다. 이번 절편이 추가한 보고서 링크 오류는 없다.

## 6. 다음 절편 관문

다음 절편은 `explore`의 query·security detector·MCP registry 의존 경계와 계약 밀도를 최신
`upstream/devel`에서 다시 조사한다. CC>25 handler는 먼저 내부 책임을 나누는 별도 수직 절편 없이
단순 이동하지 않는다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
