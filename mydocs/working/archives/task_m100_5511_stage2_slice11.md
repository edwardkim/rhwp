# #5511 Stage 2 열한 번째 수직 절편 — 초소형 모델 digest query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 구현 커밋: `c37e86791`
- 수행일: 2026-08-19
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정과 경계 판단

열한 번째 이동 대상으로 `digest`를 선택했다. 이 명령은 초소형 모델이 한 번의 호출로
문서 메타·구조·발췌를 얻고 다음 읽기 범위를 결정하도록 만든 read-only query다. 기본 v1,
절 단위 청킹, 쪽 범위, 구조 없는 문서의 쪽 폴백, MCP 전달과 오류 경로를 기존 계약이
보호하므로 move-only 절편으로 적합했다.

후보였던 `show_info`는 현재 cognitive complexity 34라 단순 이동하면 새 CLI 모듈의
CC>25=0 불변식을 깨므로 제외했다. `dump-pages`는 #5525에서 확인된 계약 drift가 남아 있고,
`dump_controls`는 CC 68이라 같은 이유로 제외했다. `digest_document`는 임계 초과가 없으며
전용 상수와 `parse_digest_pages`를 함께 옮겨도 새 모듈은 332줄로 1,200줄 상한 이하다.

절편 시작·종료 시 `upstream/devel`은 `1a6ce79fd`로 동일했다. 활성 PR 중
`src/main.rs`, `src/cli/queries/digest.rs`, `src/cli/queries/mod.rs`, digest v1/v2 계약,
CLI catalog 계약과 이 보고서에 겹치는 변경은 없었다.

## 2. 구현 결과와 보호 불변식

- 새 `src/cli/queries/digest.rs`가 `digest_document`, `parse_digest_pages`와 digest 전용
  상수 8개를 소유한다.
- `src/main.rs`의 최상위 match는 digest query 모듈 API만 호출한다.
- 공개 함수 표식과 파일 경계의 빈 줄만 정규화한 기계 비교에서 이동 블록이 일치했다.
- `cli_catalog_contract`가 handler·전용 parser 소유권, root 재유입 금지와 dispatch를 고정한다.
- 기본·sections·pages 모드의 JSON schema, 절단 규칙, 주소, `nextStep`, exit code와
  stdout/stderr 분리를 바꾸지 않았다.

`info_json_value`는 `info`, `batch info`와 `digest`가 함께 소비한다. 이를 digest CLI 하위로
옮기면 다른 query와 batch가 digest adapter에 역의존하므로 crate root에 보존했다. 공유 문서
메타 query와 envelope를 application/service 경계로 내리는 일은 Stage 3에서 다룬다.

## 3. 지표 변화

| 항목 | Stage 2 절편 10 | Stage 2 절편 11 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 40,319 | 39,995 | -324 |
| `src/cli/queries/digest.rs` | 없음 | 332 | 신규, 모듈 상한 이하 |
| `main.rs` 최상위 함수 | 328 | 326 | handler·전용 parser 이동 |
| 누적 이동 read-only handler | 14 | 15 | 1개 추가 |
| `main.rs` CC>25 함수 | 19 | 19 | 변화 없음 |
| `main.rs` 최대 CC | 68 | 68 | 변화 없음 |
| digest 모듈 CC>25 함수 | 없음 | 0 | 새 경계 내 초과 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | 변화 없음 |
| `rhwp::model` 직접 참조 | 63 | 63 | 변화 없음 |
| `rhwp::parser` 직접 참조 | 89 | 88 | format detection 호출 이동 |
| `rhwp::renderer` 직접 참조 | 24 | 24 | 변화 없음 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

이번 절편은 전용 상수와 parser까지 책임 경계 안에 두되 공유 helper는 root에 남겼다. 따라서
복잡도를 다른 파일로 숨기지 않았고 service 이행과 관찰 가능한 동작 변경도 섞지 않았다.

## 4. 외부 동작 동등성

열 번째 절편 통합 바이너리와 이동 후 바이너리에 대해 다음 열다섯 경로의 exit code와
stdout/stderr SHA-256을 비교했다.

1. 기본 v1 JSON 봉투
2. 기본 모드 16자 상한
3. 구조 문서 sections 모드
4. sections 모드 절별 8자 상한
5. 구조 없는 문서의 page 폴백
6. pages 1..2 범위와 다음 호출 안내
7. 문서 끝을 넘는 pages 범위의 clamp
8. 존재하지 않는 파일
9. 필수 경로 누락
10. 위치 인자 초과
11. 잘못된 `--max-chars`
12. 역전된 pages 범위
13. sections/pages 동시 지정
14. 문서 밖 시작 쪽
15. 알 수 없는 옵션

열다섯 경로 모두 byte 단위로 일치했다. 대표적으로 기본 봉투는 4,357 bytes,
SHA-256 `075eef6f92d780ac7382661b761170d206a1e0d0cee96f26ef26343ebe46a0f9`, sections
봉투는 6,159 bytes,
SHA-256 `5db675546d003474e52f12fd780ce37bbad95d960bd48565620ceddec2093211`다. pages
1..2 봉투 4,496 bytes의 SHA-256은
`f3f389f8f759e511f0c5ed772275f82f1d3e7a6ee57ad7ad7f22288e085d94a7`로 일치했다.
오류 경로는 stdout이 비어 있으며 기존 exit 1/2와 stderr hash를 그대로 유지했다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| 이동 전·후 digest v1/v2·agent ladder focused nextest | 각각 27/27 통과 |
| `cli_catalog_contract` | 이동 전 13/13, 이동 후 14/14 통과 |
| 정상·청킹·범위·폴백·오류 출력 hash equivalence | 15/15 일치 |
| release-test 전체 nextest | 7,744/7,744 통과, 3 slow, 38 skipped, 163.334초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| Rust test suite manifest | 통과, 750 sources / 3,695 static test attrs / 43 integration targets |
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

다음 절편은 `explain`·`explore`처럼 남은 read-only query의 전용 helper 경계와 계약 밀도를
최신 `upstream/devel`에서 다시 조사한다. CC>25 handler는 먼저 내부 책임을 나누는 별도
수직 절편 없이 단순 이동하지 않는다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지 않는다.
