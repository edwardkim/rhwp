# 값축 숫자의 글줄 점유 회귀 입력

18pt 글꼴·하단 범례인 공개 차트 호스트에서 차트 높이만 바꾼 여섯 합성 경계 입력이다.
저장 LineSeg가 없는 공개 API 호스트를 사용했다. `Chart/chart1.xml`과 CFB 안 OOXML의 값·
서식·범례는 그대로 보존하고, 현대·fallback 개체의 크기를 함께 바꿨다. 원본 문서 전체의
fidelity를 입증하는 자료와 구분한다.

## 독립 기대값

같은 입력을 공식 Hancom Office 2020 MCP의 비동기 start/status/download로 Print 출력했다.
실제 엔진은 `2020`, 한컴 빌드는 `11.0.0.9136`, `pdf_print_method=0`,
`hancom2020_pdf_driver_one_up`이다. PDF는 모두 한 쪽이며 내용을 고치지 않았다.
CLI의 `--compat 2022`는 이 저장 입력을 읽는 호환 모드이고, 한컴 출력 엔진을 뜻하지 않는다.

| 높이(원본 HWPUNIT) | Print에 보이는 값축 숫자 | 수정 전 Native074 |
| ---: | --- | --- |
| 16875 | 0, 6 | 0 |
| 16950 | 0, 6 | 0 |
| 17025 | 0, 6 | 0 |
| 17100 | 0, 6 | 0 |
| 17250 | 0, 4 | 0 |
| 18000 | 0, 4 | 0, 6 |

숫자는 독립 PDF의 확대 판독과 600dpi 숫자 밴드에서 정했다. 구현의 생략 간격 계산을
기대값으로 재인용하지 않았다. 실제 축의 왼쪽에서 소유 숫자를 찾고 값·순서·비겹침,
항목 네 개·범례 세 개·제목의 보존과 앞뒤 관계를 검사한다. 절대 px 위치는 고정하지 않는다.

## 시각 선행 증거

제품 코드 `a19d7c3d164807d69ef85b40a75908957cb0b2d8`의 Native075와 최적화 fresh WASM028은
기록된 제품 소스 180개가 같았다. 같은 입력·독립 PDF를 96dpi, 인쇄 프로필로 전쪽 비교했다.
여섯 입력·두 경로 모두 쪽수 1=1, 누락 0이며 전쪽 raw 최저는 Native **97.24897%**,
fresh WASM **96.34703%**다. 개별 해시·쪽수·점수·원 TSV·Print receipt 경로는
[`expected.json`](expected.json)에 있다. TSV·실행 로그는 ignored output에 보존한다.

- `output/pr-review/semanticist21-20261019/chart-axis-line-height-candidate-1018-cartesian-axis-line-height-fixed-native/`
- `output/pr-review/semanticist21-20261019/chart-axis-line-height-candidate-1019-cartesian-axis-line-height-fixed-wasm/`
- `output/pr-review/semanticist21-20261019/chart-axis-six-case-panels-1024/`

1024의 review와 독립 overlay를 Native/fresh 각각 여섯 입력 모두 직접 열어 숫자 보존을
확인했다. 확대용 crop은 같은 전쪽 PNG의 같은 좌표이고 전쪽 점수를 대신하지 않는다.
외곽선·막대·글자의 작은 차이와 전체 차트 범위의 다른 90% 미달 조건은 별도 보정 중이다.
이 여섯 회귀의 추가를 통합 PR 전체의 완료나 제출 승인으로 취급하지 않는다.

## 실제 회귀 실행

새 원본은 `tests/cases/chart_axis_label_line_occupancy.rs`다. 기존 검사·기대값을 교체하지 않는다.
검토 환경에서 manifest를 준비한 뒤 공용 `target/pr-review`, 빌드 잡 8개로 실행한다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_BUILD_JOBS=8 node scripts/run-rust-test.mjs chart_axis_label_line_occupancy -- \
  --cargo-profile release-test --target-dir target/pr-review --test-threads 6
```

동일하게 컴파일한 검사를 수정 전 Native074와 수정 후 Native075 실행 파일에 연결해
원인에 따른 FAIL/PASS를 대조한다. `CARGO_BIN_EXE_rhwp` 런타임 override는 이 대조에만 사용하고,
일반 CI는 Cargo가 지정한 현재 `rhwp` 실행 파일로 실제 문서를 출력한다.

1026 실행에서 같은 검사 바이너리는 Native074에서 숫자 assertion으로 **6FAIL**, Native075에서
**6PASS**였다. 일반 nextest도 **6PASS**였다. 이후 실행한 focused Clippy는 새 테스트에 도달하기
전에 root lib의 `pua_glyph.rs:193` 수동 find와 `render_tree.rs:400` 큰 enum 두 오류로 실패했다.
1035에서 이 두 오류를 보정한 뒤 focused Clippy와 PUA 3건+차트 6건의 nextest 9건이 통과했다.
1039에서 생성기로 수집 상태를 다시 맞춘 뒤 manifest·fmt check도 통과했다. 관련 기록은
`mydocs/working/task_semanticist21_20261021_renderer_lint.md`를 참조한다. 최종 전체 검증과
차트 범위의 추가 보정이 남아 있으며 PR 제출 gate는 통과하지 않았다.
