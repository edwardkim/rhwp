# Task M100 #3739 결과 보고서 — HWPX 동일 글자모양 경계 보존

## 결과

`samples/lseg-04-indent.hwp`를 HWPX로 내보낸 뒤 다시 읽을 때, 같은
`char_shape_id`를 가진 두 번째 경계가 사라져 `--verify`가 exit 3으로 끝나던 문제를
해소했다. 이제 `export-hwpx --verify --verify-pages`는 1쪽 검증과 IR 무차이를 모두
통과하고 exit 0을 반환한다.

## 원인과 조치

serializer와 parser가 모두 연속 동일 ID를 값만으로 dedup했다. 그러나
`(start_pos, char_shape_id)`에서 `start_pos`는 HWP `PARA_CHAR_SHAPE`의 보존 대상이다.
따라서 일반 run 경계는 위치까지 유지하고, 템플릿이 만드는 첫 `secPr` run의 동일-ID
handoff만 예외적으로 정규화했다.

## 변경 파일

- `src/serializer/hwpx/section.rs`
- `src/parser/hwpx/section.rs`
- `tests/issue_3739_hwpx_same_char_shape_boundary.rs`
- `mydocs/plans/task_m100_3739.md`
- `mydocs/working/task_m100_3739_stage1.md`

## 검증

- 실제 재현 샘플 CLI: `--verify --verify-pages` exit 0
- #3739 serializer/parser 단위 테스트: 2 passed
- 실제 샘플 통합 회귀 테스트: 1 passed
- 기존 표 슬롯 동일-ID parser 테스트: 1 passed
- 변경 Rust 파일 rustfmt 검사 및 diff whitespace 검사: 통과

전체 baseline·clippy·PR CI는 사용자 승인 후 다음 검증 단계에서 실행한다.
