# Task M100 #3739 — 구현·focused 검증 완료 보고서

## 작업 잠금과 기준

- 이슈 [#3739](https://github.com/edwardkim/rhwp/issues/3739)는 `jangster77`에게 할당했다.
- 기준은 `upstream/devel` `99f6c9312cdfd363a43246d44d90c3621e8f4ab3`이며,
  작업 브랜치는 `codex/issue-3739-preserve-charshape-boundaries`다.

## 구현

1. `src/serializer/hwpx/section.rs`
   - `RunSplitter`가 연속된 동일 `char_shape_id`를 제거하지 않고, 각 `start_pos`마다
     별도 `<hp:run>`을 방출하도록 변경했다.
2. `src/parser/hwpx/section.rs`
   - 일반 동일-ID run 경계를 보존하도록 전역 dedup을 제거했다.
   - 섹션 첫 문단의 `secPr` 템플릿 run과 같은 ID의 첫 텍스트 run만 합성 중복으로
     정규화해 기존 첫 문단 IR 계약을 유지했다.
3. 회귀 테스트
   - serializer 단위 테스트: 동일 ID 경계가 두 run으로 출력되는지 확인.
   - parser 단위 테스트: 일반 경계 보존과 `secPr` handoff 예외를 확인.
   - `tests/issue_3739_hwpx_same_char_shape_boundary.rs`: 실제
     `samples/lseg-04-indent.hwp`에 `export-hwpx --verify --verify-pages`를 실행해
     종료 코드 0을 고정.

## focused 검증

| 검증 | 결과 |
|---|---|
| `cargo build --profile release-test --target-dir target\pr-review --bin rhwp` | 통과 |
| `rhwp export-hwpx samples\lseg-04-indent.hwp <temp>\lseg-04-indent.hwpx --verify --verify-pages` | 통과 — 1쪽, IR 차이 없음, exit 0 |
| `issue_3739_hwpx_same_char_shape_boundary` 전용 테스트 실행 | 1 passed |
| `issue_3739` library 단위 테스트 실행 | 2 passed |
| 기존 `test_parse_control_keeps_interleaved_offsets` | 1 passed |
| 변경 Rust 파일 `rustfmt --check` 및 `git diff --check` | 통과 |

`cargo fmt --all`은 이 Windows 호스트에서 기존 경로 길이 제한(OS error 206)으로 실행되지
않아, 변경한 Rust 파일에는 동일 rustfmt 버전으로 직접 `--check`를 수행했다.

## 추가 HWP 샘플 확인

같은 Windows `rhwp export-hwpx --verify --verify-pages` 명령으로 다음 샘플을 추가 확인했다.

| 샘플 | 결과 |
|---|---|
| `hwp3-pagedef-1915.hwp` | exit 0 — IR 무차이 |
| `issue1892_hwp3_tab_roundtrip.hwp` | exit 0 — IR 무차이 |
| `hwpers_test4_complex_table.hwp` | exit 0 — IR 무차이 |
| `footnote-tbox-01.hwp` | exit 0 — IR 무차이 |
| `tac-img-02.hwp` | exit 3 — field `parameters` 6건 기존 차이, char_shapes 차이 없음; 페이지 66쪽 검증 통과 |

마지막 항목은 #3739의 수정 경로와 다른 field 메타데이터 축이므로 본 수정의 회귀로 분류하지
않았다.

## 보류

- HWPX baseline 전수, `cargo clippy`, 전체 PR CI 성격의 검증은 별도 승인 전에는 실행하지 않았다.
- 원격 push·PR 생성·merge는 수행하지 않았다.
