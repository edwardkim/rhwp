# #5511 Stage 2 네 번째 수직 절편 — 미주 line trace 진단 query 물리 분리

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `c3c35306b1428a2dcd97656d1cbe4a8c74c780a7`
- 구현 커밋: `ed522e0a2`
- 수행일: 2026-08-18
- 상태: 완료 — 다음 Stage 2 절편 승인 대기

## 1. 절편 선정

네 번째 이동 대상으로 `dump-endnote-lines`와 전용 line-trace helper 묶음을 선택했다. 시작 전
각 helper의 전체 참조를 다시 계측한 결과 `dump_paragraph_line_trace`,
`format_layout_tac_hits`, `format_control_positions`, `format_runs`, `format_u32_list`,
`brief_text`, `control_kind`는 모두 이 명령 계열에서만 사용됐다. 다른 명령이 공유하는 helper를
억지로 끌어가거나 root에 프록시를 남길 필요가 없으므로 handler와 함께 이동했다.

활성 PR #5525의 `src/main.rs` 변경은 `dump_pages`의 `--compat 2022|2024` 처리에만
한정된다. 이번 `dump-endnote-lines` dispatch와 handler 구간에는 겹치지 않으며 최신 PR 상태도
open·mergeable clean으로 확인했다.

## 2. 구현 결과

- `src/cli/queries/diagnostics.rs`가 `dump_endnote_lines`와 전용 helper 7개를 소유한다.
- `src/main.rs`의 최상위 match는 diagnostics 모듈 API만 호출한다.
- handler와 helper 본문은 공개 함수 표식 및 마지막 빈 줄을 제외하고 이동 전 원본과
  기계적으로 일치한다.
- catalog의 명령 metadata, help, capabilities와 사람용 진단 출력은 변경하지 않았다.
- `cli_catalog_contract`가 handler 구현 및 dispatch 소유권을 고정한다.

새 모듈은 기존 `load_document`, `LoadError::report`, exit code를 crate root에서 계속
사용한다. 이는 move-only 동등성을 위한 Stage 2의 의도적 binary-local seam이며 Stage 3에서
service 경계로 이행한다.

## 3. 지표 변화

| 항목 | Stage 2 절편 3 | Stage 2 절편 4 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 41,648 | 41,230 | -418 |
| `src/cli/queries/diagnostics.rs` | 101 | 521 | +420, 모듈 상한 1,200 이하 |
| `main.rs` 최상위 함수 | 342 | 334 | handler 1개·전용 helper 7개 이동 |
| 누적 이동 read-only handler | 7 | 8 | 1개 추가 |
| CLI CC>25 함수 | 19 | 19 | 변화 없음 |
| CLI 최대 CC | 68 | 68 | `dump_controls`, 변화 없음 |
| `wasm_api::HwpDocument` 직접 참조 | 42 | 42 | Stage 3 대상 |
| `rhwp::model` 직접 참조 | 70 | 64 | 6개 이동 |
| `rhwp::renderer` 직접 참조 | 28 | 25 | 3개 이동 |
| `rhwp::service` 참조 | 0 | 0 | Stage 3 대상 |

이 묶음의 함수는 모두 CC 25 이하이므로 복잡도 경고 수치는 변하지 않았다. diagnostics 모듈은
521줄로 파일별 1,200줄 상한을 충족한다.

## 4. 외부 동작 동등성

세 번째 절편 완료 시점의 release-test 바이너리와 이동 후 바이너리에 대해 다음 여섯 경로의
exit code와 stdout/stderr SHA-256을 비교했다.

1. `samples/endnote-01.hwp`의 `s0:p3:ci0` 미주 전체 trace 성공
2. 같은 미주의 `note-para 0` 필터 성공
3. section 인덱스 파싱 오류
4. 미주가 아닌 control 선택 오류
5. 존재하지 않는 파일의 런타임 오류
6. 필수 인자 누락의 사용법 오류

여섯 경로 모두 byte 단위로 일치했다. 문단·line segment·run·TAC trace, exit 0/1/2,
한글 오류 문구와 silent stdout 계약에 변화가 없다.

## 5. 검증 기록

| 검증 | 결과 |
|---|---|
| diagnostic focused nextest | 13/13 통과 |
| `cli_catalog_contract` | 8/8 통과 |
| 성공·필터·오류 출력 hash equivalence | 6/6 일치 |
| release-test 전체 nextest | 7,308/7,308 통과, 3 slow, 38 skipped, 157.414초 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 2 ignored |
| Rust test suite manifest | 통과, 716 sources / 43 integration targets |
| Rust unit tier | 통과, 4,225 tests / 298 modules |
| `check_markdown_links.py` | 기존 capability 등록부 무결성 오류 16건으로 실패, #5511 신규 오류 없음 |

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. 렌더러 자체의 동작은 변경하지 않고 진단 adapter의 위치만 옮겼으므로
계획서의 범위별 게이트에 따라 시각 검증과 WASM 빌드는 추가하지 않았다.

## 6. 다음 절편 관문

다음 Stage 2 후보는 `dump-extents`다. renderer tree type과 중첩 walker·gap 분석을 한 함수에
포함하므로 출력 기준선뿐 아니라 내부 helper 경계와 함수 복잡도를 먼저 계측해야 한다.
PR #5525가 먼저 병합되면 인접한 `dump_pages` 변경을 최신 `upstream/devel` 기준으로 다시
대조하되, `dump-pages` 자체는 해당 PR이 정리되기 전 이동하지 않는다.

다음 절편은 메인테이너 승인 전 시작하지 않으며 remote push도 별도 승인 전 수행하지
않는다.
