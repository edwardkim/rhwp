# PR #4776 검토 기록 - 저장 시 본문 손실 4종

## 메타데이터

- PR: [#4776](https://github.com/edwardkim/rhwp/pull/4776)
- 작성자: `planet6897`
- 검토자: `jangster77` (maintainer 보정)
- 대상: `edwardkim/rhwp:devel`
- 코드 검토 head: `9f6a51a01`
- 상태: `MERGEABLE`, `CLEAN`
- 검토일: 2026-08-15

## 검토 범위

HWP3 완성형 좌표 문자 복원, OWPML `FieldType` 열거 정규화, 숨은 설명 직렬화,
소프트 하이픈 왕복, 누름틀 안쪽 컨트롤의 `fieldEnd` 위치 보존을 검토했다.

초기 검토에서 빈 문단 전용 HWPX 직렬화 경로가 모든 슬롯 뒤에 `fieldEnd`를 일괄 방출하는
결함을 확인했다. 연속 또는 중첩된 0-텍스트 필드에서는 종료 마커가 교차해 표·그림의
필드 소속과 `fieldid` 짝이 바뀔 수 있었다.

maintainer 보정 `9f6a51a01`은 빈 문단의 슬롯 루프에도 일반 경로와 같은
`control_idx + inner_slot_count == emitted_ctrl_idx` 조건을 적용했다. 따라서 각
필드는 마지막 내부 슬롯 직후 닫히고, 맞지 않는 범위만 기존 말미 fallback으로 남는다.
상세 계약은 [Stage 1](../../working/task_m100_4776_stage1_empty_paragraph_field_end_order.md)에
기록했다.

## 검증

새 코드 head `9f6a51a01`의 GitHub Actions run
[`31818862246`](https://github.com/edwardkim/rhwp/actions/runs/31818862246)은 다음을 통과했다.

- `Lint (fmt, clippy, WASM check)`
- `Native Skia tests`
- `build-test-archive` 3개와 기본 feature test shard 3개, slow shard
- `Build & Test`

동일 head에서 [CodeQL run 31818862092](https://github.com/edwardkim/rhwp/actions/runs/31818862092)의
JavaScript/TypeScript, Python, Rust 분석과
[Render Diff run 31818862119](https://github.com/edwardkim/rhwp/actions/runs/31818862119)의
Canvas visual diff도 통과했다. WASM Build와 Frontend unit gates는 경로 정책에 따라 skipped다.

maintainer는 별도 로컬 검증 명령을 실행하지 않았다. 원격 full CI가 보정 head 전체를 검증한다.

## 결론

빈 문단 field 종료 순서 결함은 maintainer 보정으로 해소됐다. 이 review 문서·구현 기록·오늘할일을
추가한 docs-only head는 fast-pass 완료 뒤 최신 head의 mergeability를 다시 확인하고 병합할 수 있다.
