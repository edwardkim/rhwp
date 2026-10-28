# 캡션 자동 번호의 원문 자리표와 표시 문자열

PR #8049의 원본과 기존 정식 검사의 공백 없는 반례를 같은 독립 한컴 Print와 대조했다.
제품 `e787be830763d871703e025aa7adae499f954618`의 Native082/fresh WASM033이다.
각 입력은 전체1쪽이며 양쪽에서 PDF1=1·최저99.981%대다. 실제 제품과 Print 계약,
입력·PDF·TSV·대표 PNG SHA, 명령은 [provenance.json](provenance.json)에 보존한다.
로그와 중간 JSON은 ignored output에만 두었다. 폰트 바이너리는 포함하지 않는다.

| 입력 | Native raw | fresh WASM raw | 독립 Print |
| --- | ---:| ---:| --- |
| 원본 `ta-pic-001-r.hwpx` | 99.98114% | 99.98117% | `<그림 1>` |
| 공백 없는 기존 반례 | 99.98115% | 99.98118% | `<표 Ⅱ-1>` |

[Native TSV 원문](native/)과 [fresh TSV 원문](wasm/), [기준 PDF](pdf/),
[기존 검사의 같은 반례](inputs/ta-pic-caption-tight.hwpx)를 보존한다. 원본은 이미 추적하는
[samples/hwpx/ta-pic-001-r.hwpx](../../../../samples/hwpx/ta-pic-001-r.hwpx)다.
독립 Print의 번호·주변 공백·본문과 그림 경계를 Native/fresh에서 직접 판독했다.
얇은 표 선의 raster 농도 차이는 남고, raw99.98%를 RGB 동일로 해석하지 않는다.

| 대표 | review | standalone overlay |
| --- | --- | --- |
| Native의 공백 없는 반례 | [review](native_review.png) | [overlay](native_overlay.png) |
| fresh WASM의 원본 | [review](wasm_review.png) | [overlay](wasm_overlay.png) |

렌더 트리는 모델 자리표 한 칸을 `text`에 보존하고 실제 번호를 `display_text`에 둔다.
기존 검사의 화면 문자열 추출 helper는 `text`만 읽어 같은 제품에서2FAIL이었다.
독립 Print를 확인한 뒤 `TextRunNode::display_or_text()`를 읽도록 바꾸고 같은 제품에서
2PASS를 확인했다. 기대 문자열·좌표·공차·제품 구현은 이 검사 보정에서 바꾸지 않았다.
모델 한 칸의 인덱싱은 `text` 계약을 유지한다. 이 결과는 전체 통합 Rust/시각 검증을
대신하지 않는다. [개별 PR 리뷰](../../archives/pr_8049_review.md)를 함께 따른다.
