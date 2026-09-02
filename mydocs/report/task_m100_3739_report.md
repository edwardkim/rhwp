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

## 1단계 추가 보정

- `tac-img-02.hwp`의 field `Command` 단일 parameters는 HWP/HWP3 원본에 해당 raw XML
  저장 슬롯이 없어 HWPX 저장 과정에서 합성되는 표현이다. HWP/HWP3→HWPX `--verify`에서
  그 정확한 diff만 제거해 실제 parameters·char_shapes 등 다른 차이는 계속 실패한다.
- Windows PowerShell/.NET pipe가 `--password-stdin` 첫 바이트에 붙이는 UTF-8 BOM을
  인코딩 표식으로 제거했다. 따라서 암호 HWP3·HWP5·HWPX도 stdin 비밀번호로 HWPX로
  전환할 수 있다.
- 실제 암호 표본 3종은 BOM 포함 stdin으로 HWPX 산출물 생성과 `--verify-pages`
  재열기(24·23·64쪽)를 모두 통과했다.

## 2단계 — HWP3 IR 차이 분석 및 보정

암호 HWP3 표본의 15건 차이를 원인별로 분리했다. HWP3 파서는 개체 자리에 `U+FFFC` 한
글자를 남기지만 암호 표본의 `char_offsets`는 그 자리를 8 UTF-16 단위 슬롯으로 센다.
직렬화기가 이 표식을 일반 텍스트로 다시 쓰면 실제 HWPX 개체와 겹쳐 이후 run 경계가
밀렸다. `render_runs`가 확인된 표식만 해당 위치의 HWPX control로 치환하고, 슬롯 수
추정도 표식의 1단위/실제 8단위 차이를 보정했다. 이 조치만으로 15건 중 13건을 없앴다.

남은 하이퍼텍스트는 HWP3 `Control::Hyperlink`에 대응하는 HWPX control이 없으므로
`fieldBegin type="HYPERLINK"`와 `Command` parameter로 승격했다. HWP3 추가정보에 URL이
없으면 표시 문자열을 Command로 보존한다. HWPX 재파싱 시에는 `Field`가 되므로, HWP3의
비슬롯 Hyperlink와 `[field]` 한 개의 정확한 표기 차이만 HWP3→HWPX 검증 정규화에서
제외한다. HWP5에는 이 규칙을 적용하지 않는다.

마지막 그림 차이는 HWP3의 `[0,0,0,0]` crop 센티널을 HWPX가 실제
`(0,0,width,0)/(width,height,0,height)` 사각형으로 물질화한 경우뿐이었다. 크기나 다른
기하가 함께 달라지면 여전히 검증 실패로 남긴다. 결과적으로
`HWP3-password-123456.hwp`는 BOM 포함 stdin 비밀번호로도 `--verify --verify-pages`
exit 0, 24쪽을 통과한다.

## 변경 파일

- `src/serializer/hwpx/section.rs`
- `src/serializer/hwpx/context.rs`
- `src/serializer/hwpx/field.rs`
- `src/parser/hwpx/section.rs`
- `src/serializer/hwpx/roundtrip.rs`
- `src/main.rs`
- `tests/issue_3739_hwpx_same_char_shape_boundary.rs`
- `mydocs/plans/task_m100_3739.md`
- `mydocs/working/task_m100_3739_stage1.md`
- `mydocs/working/task_m100_3739_stage2.md`
- `mydocs/report/task_m100_3739_report.md`
- `mydocs/manual/cli_commands.md`

## 검증

- 실제 재현 샘플 CLI: `--verify --verify-pages` exit 0
- #3739 serializer/parser/검증 정규화 단위 테스트: 4 passed
- HWP3 URL 누락 fallback field 단위 테스트: 1 passed
- 실제 샘플 통합 회귀 테스트: 4 passed
- 기존 표 슬롯 동일-ID parser 테스트: 1 passed
- 변경 Rust 파일 rustfmt 검사 및 diff whitespace 검사: 통과
- 추가 HWP 4종(페이지 설정·탭·복합 표·각주/글상자): 모두 `--verify --verify-pages` exit 0
- BOM 포함 stdin 암호 HWP3·HWP5·HWPX HWPX 변환: 모두 산출물 생성 및 페이지 재열기 통과

이미지 포함 `tac-img-02.hwp`는 Command 단일 parameter의 표현 차이를 HWP→HWPX 경로에만
정규화한 뒤 66쪽 및 IR 검증을 통과한다.

전체 baseline·clippy·PR CI는 사용자 승인 후 다음 검증 단계에서 실행한다.
