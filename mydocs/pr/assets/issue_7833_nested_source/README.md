# 중첩 인라인 표와 미주 원문 소유 증거

통합 브랜치의 중간 코드 `3decf7196b`를 Native046·최적화 fresh WASM020으로 검증했다.
이 자료는 #7833 원본의 줄 높이와 소스 주소 검증이며, 통합 PR 전체의 완료 판정이 아니다.

## 원인과 공통 소비

저장된 호스트 줄 1,000 HWPUNIT이 안쪽 표 6,000과 원본 위아래 여백을 담지 못했다.
본문의 공통 `normalized_host_with_table_height`를 셀에서도 사용한다. 측정기의 실제
표 본체 높이와 원본 여백으로 기존 줄 상자 생산자가 기준선·점유를 발행한다.
HeightMeasurer와 table_layout은 같은 `paragraph_with_frame_geometry`를 소비한다.
높이만 바뀐 저장 줄은 원래 줄 소속과 HWPX 문자 주소 축을 유지한다.

저장 높이를 바꾸거나 저장 줄을 생략한 진단본의 통과를 원본의 증거로 대체하지 않았다.
아래 HWP/HWPX 원본은 모두 변경 없이 독립 Print와 비교했다. 양쪽 1쪽이며 쪽수 차이도 없다.

| 원본 | 입력 SHA-256 | 독립 Print SHA-256 | Native raw | fresh WASM raw |
| --- | --- | --- | ---: | ---: |
| [HWP](inputs/original.hwp) | `2178b260cf7adeab0ddccb6c8b4006229082a65ab85ed2ea179460b49d887e62` | `dd662cb83ed3dcfbede58117e723dd8841dbc0ba1a0f60bbcc4b45513a941cba` | 100% | 100% |
| [HWPX](inputs/original.hwpx) | `79f28d53191d62445f3ece3db3b4686ec1566572a81bb1354647111cae5081fb` | `fe8517d68e84d2b49629abab6adb1cc3f85c0df9d62b6b08c5c11f573987da91` | 100% | 100% |

MCP의 명시적 engine2020, 실제 한컴 `11.0.0.9136`, PrintMethod0·1-up 출력을 기준으로 했다.
HWP job은 `255f6b14-14c8-463f-a991-2658791cab6b`다. 원문 HWPX의 기존 Print도 재사용했다.
PDF는 [HWP 기준](pdf/hwp/original-2020.pdf), [HWPX 기준](pdf/hwpx/original-2020.pdf)에 보존한다.
Native는 `--profile print`, WASM은 Canvas2D print, 둘 다 `--compat 2022 --dpi 96`이다.
원본·PDF와 실행 파일의 SHA, 정확한 명령은 작업 중 실행 receipt에 연결했다.
실행 로그와 중간 JSON은 ignored output에만 보존한다.

TSV 원문은 `head/native/{hwp,hwpx}.tsv`와 `head/wasm/{hwp,hwpx}.tsv`에 있다.
해당 하위 폴더에 같은 쪽의 review와 standalone overlay도 보존했다.
표·셀 외곽, 원문과 번호, 앞뒤 글자의 같은 기준선, 후속 본문과 미주 접두사를 같은 배율로
직접 판독했다. 100%는 2px raw 실루엣이며 회색 테두리·검정 Print 획과 AA의 RGB 일치를 뜻하지 않는다.
고정 px 위치를 추가하지 않고 셀 내부 포함·앞뒤 순서·줄 기준선·누락과 중복을 보호한다.

## 새 이름 회귀

`tests/cases/nested_inline_table_source_ownership.rs`는 원본 파일을 그대로 읽는다.

- `nested_inline_table_keeps_host_baseline_and_cell_ownership`: 표 앞뒤 글자의 기준선과 안쪽
  본문 사이의 순서, 셀 내부 포함, 인라인 번호와 후속 원문·미주의 보존을 검사한다.
- `nested_endnote_trace_never_aliases_literal_or_body_source`: 원문 `1) `, 호스트 `앞뒤`, 안쪽
  `원문`의 개별 주소를 보존하고 생성 미주의 접두사만 source unavailable임을 확인한다.
  trace 전후 모델·SVG·HWP·HWPX 불변도 검사한다.

과거 검사는 미주 참조가 셀의 직접 TextRun이라고 가정했다. 실제 FootnoteMarker와 생성 미주
본문 TextRun을 구분하는 위 검사로 이관했다. 수정 전후 실행 결과는 실제 실행 후 기록한다.

## 수정 전후 실제 실행

동일한 새 이름 검사와 커밋한 원본을 `-D warnings`로 각각 컴파일했다.
Native045는 기준선 검사1FAIL·주소 정상 대조군1PASS, Native046은2PASS다.
이전에도 통과한 주소 검사는 새 높이 결함의 검출 증거로 주장하지 않는다.
줄 상자 원점과 실제 글자 기준선은 구분하며 `bbox.y + baseline`을 검사한다.

Native045→046과 WASM019→020의 기존82조건·242쪽은 각각 모든 페이지 점수가
동일했다. 기존 #7694 차트의 미달은 남아 있으므로 전체 통합 검토 완료로 보고하지 않는다.

정식 Cargo 자동 수집에서도 생성된 regression_suite_010의 새 검사2건이 모두
통과했다(컴파일13.98초, 실행0.041초). 공유 target/pr-review, 빌드 잡8개,
테스트 스레드12개를 사용했다. 자동 수집 준비 후 실제 실행 건수를 확인했다.
