# #6133 그림 앵커와 TAC 제목 줄·쪽 소속의 정식 회귀

## 원 해결과 현재 공통 계약

원 이슈 #6133의 해결 [PR #6169](https://github.com/edwardkim/rhwp/pull/6169),
source `c7f3b0e5841b9adc9c0016cc2725873833ec4a38`와 실제 통합 [PR #6191](https://github.com/edwardkim/rhwp/pull/6191),
반영 `dac39f92b3a359a43ac133af8c9280490683681f`를 대조했다. 전체 원본 4쪽의 제목을
그림 위에 보존하는 계약과, 폭이 다른 IR 축약본의 실제 제목 줄·그림 이월을 구분한다.

현재 공통 보정은 `9f79fb8e7c73a41eecb509e780526a7e45a79f93`이며 이번에는 제품 코드를
다시 변경하지 않았다. `controls/mixed_host_line::prepare → state::place_mixed_host_line →
inline_placements / paragraph_float_placements → 실제 표·그림 배치 → 문단 닫기`가 같은
확정 소유와 원점·물리 끝을 소비한다. 작은 그림, 원문 선언 순서와 음수/0/양수 오프셋,
그림만 또는 제목만 다음 쪽으로 이동하는 예산 경계, 뒤 본문의 저장 간격을 함께 보호한다.

가시 본문·공백·명시적 개행·여러 밴드·분할·캡션이 있는 입력을 한 줄 전용 경로에
기계적으로 포함하지 않는다. 다른 소유자가 처리하는 입력의 근거는 기존 구현·작업 기록을 따른다.

## 같은 원본과 독립 Print의 전체 페이지 검증

runtime `8dc75860cec88ddf7c3f8aeee230b2ca7e2e5537`, Native224 / 최적화 fresh WASM063,
96 DPI·compat 2022·Print 프로필을 사용했다. 실제 실행과 같은 바이트의 입력·PDF를
보존하고 전체 18조건·29쪽의 Native/fresh TSV 원문을 각각 복사했다. 누락·쪽수 불일치·
90% 미만 쪽은 없다. 전체 원본의 최저는 90.70001/90.31892%, 나머지 17조건·25쪽은
두 경로 모두 raw 100%다. 좌표를 검사하는 제목·그림 쪽은 모두 양쪽 raw 100%다.
이 페이지에만 독립 PDF의 ±2 CSS px를 적용하며 쪽·개체 소유·내용 순서·누락·중복·
겹침·용지 포함 및 표 뒤 원문 간격 관계도 유지한다.

전체 원본은 작업지시자가 내용을 바꾸지 않고 Print로 출력한 PDF다. 실제 Creator는
`Hwp 2022 12.0.0.4605`이며 파일명 `-2020.pdf`로 제품 연도를 추정하지 않는다.
본문 휴먼명조를 Gulim으로 대체한 과거 MCP PDF를 이 원본의 기준으로 사용하지 않았다.
나머지 17개 PDF는 같은 입력/PDF SHA의 MCP 성공 기록에서 engine 2020,
실제 한컴 `11.0.0.9136`, `pdf_print_method=0`, 전쪽 수·수신 바이트를 확인했다.
job ID와 실제 PDF 메타데이터는 [provenance.json](provenance.json)에 있다.

| 조건 | 입력 / PDF | Native / fresh 전체 TSV | 전쪽 raw 최저 Native / fresh |
| --- | --- | --- | ---: |
| `offset-negative-1200-picture-first-coherent` | [입력](inputs/offset--1200-picture-first-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/offset--1200-picture-first-coherent-2020.pdf) | [Native](tsv/native/offset-negative-1200-picture-first-coherent.tsv) / [fresh](tsv/wasm/offset-negative-1200-picture-first-coherent.tsv) | 100.00000% / 100.00000% |
| `offset-0-picture-first-coherent` | [입력](inputs/offset-0-picture-first-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/offset-0-picture-first-coherent-2020.pdf) | [Native](tsv/native/offset-0-picture-first-coherent.tsv) / [fresh](tsv/wasm/offset-0-picture-first-coherent.tsv) | 100.00000% / 100.00000% |
| `offset-1200-picture-first-coherent` | [입력](inputs/offset-1200-picture-first-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/offset-1200-picture-first-coherent-2020.pdf) | [Native](tsv/native/offset-1200-picture-first-coherent.tsv) / [fresh](tsv/wasm/offset-1200-picture-first-coherent.tsv) | 100.00000% / 100.00000% |
| `offset-2831-picture-first-coherent` | [입력](inputs/offset-2831-picture-first-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/offset-2831-picture-first-coherent-2020.pdf) | [Native](tsv/native/offset-2831-picture-first-coherent.tsv) / [fresh](tsv/wasm/offset-2831-picture-first-coherent.tsv) | 100.00000% / 100.00000% |
| `offset-1200-table-first-coherent` | [입력](inputs/offset-1200-table-first-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/offset-1200-table-first-coherent-2020.pdf) | [Native](tsv/native/offset-1200-table-first-coherent.tsv) / [fresh](tsv/wasm/offset-1200-table-first-coherent.tsv) | 100.00000% / 100.00000% |
| `offset-3518-picture-first-coherent` | [입력](inputs/offset-3518-picture-first-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/offset-3518-picture-first-coherent-2020.pdf) | [Native](tsv/native/offset-3518-picture-first-coherent.tsv) / [fresh](tsv/wasm/offset-3518-picture-first-coherent.tsv) | 100.00000% / 100.00000% |
| `picture-height-60000-offset-3518-coherent` | [입력](inputs/picture-height-60000-offset-3518-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/picture-height-60000-offset-3518-coherent-2020.pdf) | [Native](tsv/native/picture-height-60000-offset-3518-coherent.tsv) / [fresh](tsv/wasm/picture-height-60000-offset-3518-coherent.tsv) | 100.00000% / 100.00000% |
| `offset-3518-table-first-coherent` | [입력](inputs/offset-3518-table-first-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/offset-3518-table-first-coherent-2020.pdf) | [Native](tsv/native/offset-3518-table-first-coherent.tsv) / [fresh](tsv/wasm/offset-3518-table-first-coherent.tsv) | 100.00000% / 100.00000% |
| `picture-height-63000-offset-3518-coherent` | [입력](inputs/picture-height-63000-offset-3518-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/picture-height-63000-offset-3518-coherent-2020.pdf) | [Native](tsv/native/picture-height-63000-offset-3518-coherent.tsv) / [fresh](tsv/wasm/picture-height-63000-offset-3518-coherent.tsv) | 100.00000% / 100.00000% |
| `picture-height-62000-offset-3518-coherent` | [입력](inputs/picture-height-62000-offset-3518-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/picture-height-62000-offset-3518-coherent-2020.pdf) | [Native](tsv/native/picture-height-62000-offset-3518-coherent.tsv) / [fresh](tsv/wasm/picture-height-62000-offset-3518-coherent.tsv) | 100.00000% / 100.00000% |
| `picture-height-65000-offset-3518-coherent` | [입력](inputs/picture-height-65000-offset-3518-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/picture-height-65000-offset-3518-coherent-2020.pdf) | [Native](tsv/native/picture-height-65000-offset-3518-coherent.tsv) / [fresh](tsv/wasm/picture-height-65000-offset-3518-coherent.tsv) | 100.00000% / 100.00000% |
| `picture-height-68200-offset-3518-coherent` | [입력](inputs/picture-height-68200-offset-3518-coherent.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/picture-height-68200-offset-3518-coherent-2020.pdf) | [Native](tsv/native/picture-height-68200-offset-3518-coherent.tsv) / [fresh](tsv/wasm/picture-height-68200-offset-3518-coherent.tsv) | 100.00000% / 100.00000% |
| `original-6133-slice` | [입력](../../../../samples/issue6133/156483831_poster_title_above_offset_float.hwp) / [PDF](../../../../pdf/issue6133-mixed-host-line/original-slice-2020.pdf) | [Native](tsv/native/original-6133-slice.tsv) / [fresh](tsv/wasm/original-6133-slice.tsv) | 100.00000% / 100.00000% |
| `original-6133-full` | [입력](inputs/full-source-156483831.hwp) / [PDF](../../../../pdf/issue6133-mixed-host-line/full-source-156483831-2020.pdf) | [Native](tsv/native/original-6133-full.tsv) / [fresh](tsv/wasm/original-6133-full.tsv) | 90.70001% / 90.31892% |
| `offset-0-picture-first-coherent-successor` | [입력](inputs/offset-0-picture-first-coherent-successor.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/offset-0-picture-first-coherent-successor-2020.pdf) | [Native](tsv/native/offset-0-picture-first-coherent-successor.tsv) / [fresh](tsv/wasm/offset-0-picture-first-coherent-successor.tsv) | 100.00000% / 100.00000% |
| `offset-3518-table-first-coherent-successor` | [입력](inputs/offset-3518-table-first-coherent-successor.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/offset-3518-table-first-coherent-successor-2020.pdf) | [Native](tsv/native/offset-3518-table-first-coherent-successor.tsv) / [fresh](tsv/wasm/offset-3518-table-first-coherent-successor.tsv) | 100.00000% / 100.00000% |
| `picture-height-60000-offset-3518-coherent-successor` | [입력](inputs/picture-height-60000-offset-3518-coherent-successor.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/picture-height-60000-offset-3518-coherent-successor-2020.pdf) | [Native](tsv/native/picture-height-60000-offset-3518-coherent-successor.tsv) / [fresh](tsv/wasm/picture-height-60000-offset-3518-coherent-successor.tsv) | 100.00000% / 100.00000% |
| `picture-height-68200-offset-3518-coherent-successor` | [입력](inputs/picture-height-68200-offset-3518-coherent-successor.hwpx) / [PDF](../../../../pdf/issue6133-mixed-host-line/picture-height-68200-offset-3518-coherent-successor-2020.pdf) | [Native](tsv/native/picture-height-68200-offset-3518-coherent-successor.tsv) / [fresh](tsv/wasm/picture-height-68200-offset-3518-coherent-successor.tsv) | 100.00000% / 100.00000% |

## 직접 판독과 남은 차이

현재 runtime으로 대표 6조건·9쪽을 양쪽에서 다시 캡처해 canonical review·standalone
overlay 36개를 보존했다. 원본 포스터, 축약본의 제목 쪽, 그림 먼저/제목 먼저,
그림 1쪽·제목 2쪽과 제목 1쪽·그림 2쪽의 양방향 이월, 뒤 본문의 간격을 대조했다.
raw 100%는 RGB 완전 일치가 아니다. 그림 가장자리와 선·글자의 raster/농도 차이는
남으며 전체 원본 1~2쪽의 글자 농도·일부 미세 간격 잔차도 별도로 남긴다.
포스터 4쪽의 높은 점수를 문서 전체의 픽셀 동일성으로 표현하지 않는다.

| 대표 쪽 | Native review / overlay | fresh review / overlay |
| --- | --- | --- |
| `original-6133-full` p004 | [review](png/native/original-6133-full/review_004.png) / [overlay](png/native/original-6133-full/overlay_004.png) | [review](png/wasm/original-6133-full/review_004.png) / [overlay](png/wasm/original-6133-full/overlay_004.png) |
| `original-6133-slice` p001 | [review](png/native/original-6133-slice/review_001.png) / [overlay](png/native/original-6133-slice/overlay_001.png) | [review](png/wasm/original-6133-slice/review_001.png) / [overlay](png/wasm/original-6133-slice/overlay_001.png) |
| `original-6133-slice` p002 | [review](png/native/original-6133-slice/review_002.png) / [overlay](png/native/original-6133-slice/overlay_002.png) | [review](png/wasm/original-6133-slice/review_002.png) / [overlay](png/wasm/original-6133-slice/overlay_002.png) |
| `offset-0-picture-first-coherent-successor` p001 | [review](png/native/offset-0-picture-first-coherent-successor/review_001.png) / [overlay](png/native/offset-0-picture-first-coherent-successor/overlay_001.png) | [review](png/wasm/offset-0-picture-first-coherent-successor/review_001.png) / [overlay](png/wasm/offset-0-picture-first-coherent-successor/overlay_001.png) |
| `offset-3518-table-first-coherent-successor` p001 | [review](png/native/offset-3518-table-first-coherent-successor/review_001.png) / [overlay](png/native/offset-3518-table-first-coherent-successor/overlay_001.png) | [review](png/wasm/offset-3518-table-first-coherent-successor/review_001.png) / [overlay](png/wasm/offset-3518-table-first-coherent-successor/overlay_001.png) |
| `picture-height-60000-offset-3518-coherent-successor` p001 | [review](png/native/picture-height-60000-offset-3518-coherent-successor/review_001.png) / [overlay](png/native/picture-height-60000-offset-3518-coherent-successor/overlay_001.png) | [review](png/wasm/picture-height-60000-offset-3518-coherent-successor/review_001.png) / [overlay](png/wasm/picture-height-60000-offset-3518-coherent-successor/overlay_001.png) |
| `picture-height-60000-offset-3518-coherent-successor` p002 | [review](png/native/picture-height-60000-offset-3518-coherent-successor/review_002.png) / [overlay](png/native/picture-height-60000-offset-3518-coherent-successor/overlay_002.png) | [review](png/wasm/picture-height-60000-offset-3518-coherent-successor/review_002.png) / [overlay](png/wasm/picture-height-60000-offset-3518-coherent-successor/overlay_002.png) |
| `picture-height-68200-offset-3518-coherent-successor` p001 | [review](png/native/picture-height-68200-offset-3518-coherent-successor/review_001.png) / [overlay](png/native/picture-height-68200-offset-3518-coherent-successor/overlay_001.png) | [review](png/wasm/picture-height-68200-offset-3518-coherent-successor/review_001.png) / [overlay](png/wasm/picture-height-68200-offset-3518-coherent-successor/overlay_001.png) |
| `picture-height-68200-offset-3518-coherent-successor` p002 | [review](png/native/picture-height-68200-offset-3518-coherent-successor/review_002.png) / [overlay](png/native/picture-height-68200-offset-3518-coherent-successor/overlay_002.png) | [review](png/wasm/picture-height-68200-offset-3518-coherent-successor/review_002.png) / [overlay](png/wasm/picture-height-68200-offset-3518-coherent-successor/overlay_002.png) |

축약본 그림은 원 기여자가 1×1 흰색 PNG로 바꾼 입력이다. 독립 PDF의 흰 쪽에서
그림의 픽셀 원점을 읽었다고 주장하지 않는다. 이 조건은 제목의 독립 괘선 위치와
PDF의 2쪽 수, 원문 개체의 유일한 소유·쪽 순서 및 같은 크기의 유색 대조군을 함께
검사한다. 보이지 않는 흰 그림에는 px 좌표를 추가하지 않았다. 다른 유색 그림은
독립 PDF의 실제 이미지 연산과 직사각형 clip을 적용한 띠의 합집합에서 경계를 읽었다.

## 기존 검사 보존과 새 이름의 검사

기존 모듈 전체와 helper는 [`.rs.txt`](incoming-tests/issue_6133_host_line_above_offset_float.rs.txt)로
바이트 그대로 보존했다. [provenance.json](provenance.json)에 source commit·Git blob·
SHA-256·이전/보존 경로·제거 함수와 활성 대체 모듈 및 복원 조건이 있다. 원본 HWP와
PDF는 활성 검증 경로에 그대로 남긴다. 90% 미만 원본을 별도 이슈로 이관한 처리가 아니다.

[새 정식 모듈](../../../../tests/cases/verified_mixed_float_tac_host_line.rs)의 18개 함수는
입력/PDF SHA와 독립 쪽수, 원문 문단·개체 주소의 유일성, 제목 셀의 원문 순서·내용,
같은 쪽의 포함·비겹침, 뒤 본문의 누락·중복·소유와 원본 간격을 검사한다. 기존
83.1px를 164.14px로 옮긴 한 개의 화면 핀으로 대체하지 않는다. 독립 PDF에서 적절한
페이지가 확인된 ±2px 검사와 내용 관계를 함께 둔다.

같은 정식 소스를 보존 Native211(`b7fca6dbe92a2f357b401e3b9ce4ce93dec22687`)과
현재 Native224의 실제 CLI로 실행해 수정 전 **1 PASS / 17 FAIL**, 현재 **18 PASS /
0 FAIL**을 확인했다. ignore·실행 누락은 0건이다. 전체 원본 정상 대조군은 전후 모두
통과했고, 축약본의 쪽 소속·내어쓰기와 다른 선언 순서·높이 경계에서 이전 결함을
검출했다. 단순 진단 데이터 비교가 아니라 정식 함수가 동결 CLI의 실제 render tree를
내보내 검사한 결과다. Cargo 전체 suite 또는 전체 Rust 실행 완료로 표현하지 않는다.

새 소스의 rustfmt·focused Clippy는 통과했다. 자동 수집은 같은 1810개 source·48개
integration target에서 기존 1개 대신 새 18개를 수집해 static test 수가 7856→7873개다.
보존 `.rs.txt`와 생성된 harness·manifest는 새 테스트 소스로 커밋하지 않는다.
전체 Rust·lint·Studio·원 #8064 corpus·새 base 대비 기능 보존과 CI는 별도 필수 게이트다.
