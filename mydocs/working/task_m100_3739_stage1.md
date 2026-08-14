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

## 1단계 보정 — field parameters 및 Windows 암호 stdin

1. `tac-img-02.hwp`의 field `parameters` 6건은 HWP 원본의 저장 슬롯 부재로 인해
   HWPX serializer가 `Command` 단일 parameter를 합성하고, 재파서가 그 원문을 보존해
   생기는 표현 차이였다. HWP/HWP3 입력의 **정확히 그 합성 표현만** 검증 diff에서
   제외해 `--verify`가 실제 손실을 계속 보고하도록 했다.
2. Windows PowerShell/.NET이 `--password-stdin` 첫 바이트에 붙이는 UTF-8 BOM을
   비밀번호 본문으로 해석하던 문제를 고쳤다. stdin 전체의 선두 BOM만 제거하므로 실제
   비밀번호와 두 번째 줄의 output password는 변경하지 않는다.
3. `tests/issue_3739_hwpx_same_char_shape_boundary.rs`에 다음 실제 표본 회귀를 더했다.
   - `tac-img-02.hwp`: `--verify --verify-pages` IR 무차이, 66쪽
   - `HWP3-password-123456.hwp`: BOM 포함 `--password-stdin`으로 HWPX 생성·24쪽 재열기
   - `HWP5-password-123456.hwpx`: 같은 경로로 생성·23쪽 재열기
   - `hwp3-sample16-hwp5-2024-password-123456.hwp`: 같은 경로로 생성·64쪽 재열기

암호 HWP3 표본은 위 변환 자체와 페이지 검증은 통과한다. 다만 `--verify`를 함께 주면
기존 hyperlink 2건 드롭, 문자 오프셋·그림 영역을 포함한 15건 IR 차이로 exit 3이므로,
이는 다음 스테이지에서 별도로 분석한다.

### 1단계 focused 검증 추가

| 검증 | 결과 |
|---|---|
| `issue_3739_hwpx_same_char_shape_boundary` | 3 passed |
| `issue_3739` library 단위 테스트 | 3 passed |
| BOM 포함 stdin으로 암호 HWP3·HWP5·HWPX `export-hwpx --verify-pages` | 모두 exit 0, 24·23·64쪽 |
| 변경 Rust 파일 `rustfmt --check`, `git diff --check` | 통과 |
