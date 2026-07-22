# task_m100_2901 처리결과 보고서 — HWPX secPr pageBorderFill@type 파싱값 폐기 + 직렬화 위치 라벨 고정 해소

- **이슈**: [#2901](https://github.com/edwardkim/rhwp/issues/2901)
- **브랜치**: `task/m100-2901-pageborderfill-type` (base `origin/devel` @ `95509062`)
- **범위**: `src/model/page.rs`, `src/parser/hwpx/section.rs`, `src/parser/hwp3/mod.rs`,
  `src/serializer/hwpx/section.rs` (+ 테스트 1개), 본 보고서
- **분류**: 결함 수정 (저장 충실도, #2742 후속) — 잠재 결함

## 1. 문제

`#2742` 전수 조사(`mydocs/report/task_m100_2742_report.md` §2.2)가 secPr
`pageBorderFill@type` 을 "고정(IR 필드 없음)" 슬롯으로 분류하며 "위치 기반 합성" ·
"실측(경미) — 잔여" 로 남겨 둔 항목을 코드 레벨에서 직접 확인·수정했다.

**파서는 `type` 속성(BOTH/EVEN/ODD)을 읽지만 즉시 버린다.**
`parse_page_border_fill_empty()` 가 `apply_type` 지역변수에 담아
`page_border_fill_attr(&text_border, &fill_area, &apply_type, ...)` 로 넘기지만,
`page_border_fill_attr()` 함수 바디는 `apply_type` 파라미터를 전혀 참조하지
않는다(죽은 파라미터). `PageBorderFill` 모델에도 이 값을 담을 필드가 없어 함수
스코프를 벗어나는 순간 완전히 소실된다.

**직렬화기는 인코딩 순서 슬롯 인덱스로 라벨을 합성한다.** `push_page_border_fill()`
이 1번째 요소를 `page_border_fill`(primary), 2·3번째를 `extra_page_border_fills`
(벡터)에 위치로만 저장하고, `replace_page_border_fill()` 은
`[("BOTH", primary), ("EVEN", extra[0]), ("ODD", extra[1])]` 로 라벨을 고정
매핑한다. 한컴 표준 문서는 항상 BOTH→EVEN→ODD 순서라 우연히 값이 맞지만, 원본이
그 순서를 벗어나면(다른 생산기, 재정렬된 문서 등) 저장본은 실제로 다른 유형에
적용되던 테두리 설정을 엉뚱한 유형에 붙인다.

## 2. 실측과 한계 (정직하게 명시)

`samples/hwpx/*.hwpx` 59개를 재검사(`Contents/section*.xml` 의
`<hp:pageBorderFill type="...">` 순서 추출 후 BOTH→EVEN→ODD 기대 순서와 대조)한
결과, **59파일 전부 이미 표준 순서**였고 이탈 사례는 0건이었다. 이번 결함은
현재 코퍼스 기준으로는 **잠재적**이며, #2742 보고서가 언급한 "왕복 불일치
1 파일"은 같은 비정상 문서(`issue2019_floating_form_74312.hwpx`, section 1개에
`<hp:secPr>` 10개 중첩)의 "다중 secPr → 구역 1개 축약" 구조 문제와 얽혀 있어
이 `type` 로직만의 독립 재현 사례로 분리하지 못했다. 실측 손실을 과장하지 않고
"코드 자체가 결함을 증명하는" 잠재 결함으로 명시해 이슈·커밋에 기록했다.

## 3. 수정

`PageBorderFill` 모델에 원본 `type` 문자열을 보존하는 `apply_type: String`
필드를 추가:

- 파서(`parse_page_border_fill_empty`): 읽은 `apply_type` 을
  `page_border_fill.apply_type` 에 대입(1줄).
- 직렬화기(`replace_page_border_fill`): 위치 라벨(`slot_ty`) 대신
  `pbf.apply_type` 이 비어있지 않으면 그 값을 우선 방출. 빈 문자열이면 기존
  위치 기반 라벨로 폴백(HWP5/HWP3 유래 문서 등 `type` 개념이 없는 경로와
  하위호환 유지).
- `src/parser/hwp3/mod.rs`: `PageBorderFill` 구조체 리터럴에
  `apply_type: String::new()` 보강(필드 추가로 인한 컴파일 정합, HWP3 원본에는
  대응 개념 없음).

## 4. 검증

### 4.1 red → green

**RED** — `replace_page_border_fill` 의 `ty` 선택을 `slot_ty` 고정으로 되돌리고 실행:

```
thread '...page_border_fill_type_preserves_ir_label_over_positional_default' panicked:
첫 pageBorderFill 은 위치 기반 고정 라벨 BOTH 대신 IR 이 보존한 원본 type="ODD" 를
방출해야 함: <hp:pageBorderFill type="BOTH" borderFillIDRef="0" textBorder="CONTENT" ...>
test result: FAILED. 0 passed; 1 failed
```

**GREEN** — 복원 후:

```
test serializer::hwpx::section::tests::page_border_fill_type_preserves_ir_label_over_positional_default ... ok
test result: ok. 1 passed; 0 failed
```

### 4.2 CI 3종

| 항목 | 결과 |
|---|---|
| `cargo build --lib` | 통과 |
| `cargo test --lib` (전체) | **통과** — 2284 passed / 0 failed / 7 ignored |
| `cargo clippy --all-targets --profile release-test -- -D warnings` | 통과 (경고 0) |
| `rustfmt --edition 2021` (변경 4개 `.rs`) | 변경 없음(포맷 위반 0). Windows 체크아웃 CRLF 로
  `cargo fmt --all -- --check` 는 항상 `Incorrect newline style` 만 찍는 거짓 경고이므로
  changed-file 대상 `rustfmt` 직접 실행으로 대체 |

동일 영역 관련 테스트 6개(`page_border_fill*`)도 개별 재확인, 전부 통과.

## 5. 잔여

- `pageBorderFill@type` 순서 이탈은 현재 코퍼스에서 실측 0건 — 향후 비표준
  생산기 문서가 코퍼스에 추가되면 재검증 대상.
- `extra_page_border_fills` 자체가 여전히 위치 기반 슬롯(최대 2개)이라, 문서가
  3개 미만의 `pageBorderFill` 만 가진 경우 라벨은 살아나지만 개수 자체의
  완전한 왕복(예: 2개만 있던 문서가 3개로 늘어나는지 여부)은 이번 범위 밖이다.
