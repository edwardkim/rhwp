# #6449 Gmail류 클립보드 HTML 붙여넣기 응답 없음(브라우저 멈춤) 수정

## 무엇을

사용자가 실제로 rhwp-studio에서 Gmail 서명 블록("서울 사무소" 연락처)을 복사해 붙여넣었을 때
문서가 멈추고 Chrome이 "응답 없는 페이지" 팝업을 띄웠다. 붙여넣기 시도 후 화면에는 파싱되지
않은 raw HTML 속성들(`jscontroller`, `jsaction`, `data-copy-service-computed-style` 등)이
그대로 텍스트로 노출됐다.

## 왜 (원인)

`src/document_core/commands/html_import.rs`의 HTML→문단 파서 두 곳:

1. **`parse_html_to_paragraphs`의 `<div>` 재귀 하강에 깊이 상한이 없었다.** `<div>`를 만나면
   `find_closing_tag_chars`로 하위 전체를 훑어 매칭 닫는 태그를 찾고, 그 내용 전체를 **재귀
   호출**로 다시 파싱한다. Gmail류 클립보드는 wrapper `<div>`가 수십 겹인 게 흔한데(사용자
   사례: 서명 블록 하나에 `</div>` 8개 이상 연속), 재귀 단계마다 남은 하위 콘텐츠 전체를
   다시 스캔해 깊이 × 콘텐츠 크기에 비례해 느려진다.
2. **`parse_inline_content`의 `<span>` 처리가 이미 계산해둔 깊이-인식 경계
   (`find_closing_tag_chars` 결과 `span_end_tag`)를 버리고 `"</span>"` 리터럴을 처음부터
   다시 선형 재탐색**했다. 스타일 하나마다 별도 span으로 감싸는 Gmail류 입력에서 span마다
   이 재탐색이 반복돼 실질적으로 O(n²)에 가까워진다. 깊이를 무시한 첫 매치라 중첩 span에서
   내부 span의 닫는 태그를 잘못 집는 경계 버그이기도 했다.

## 어떻게 (변경)

- `parse_html_to_paragraphs`를 깊이 파라미터를 받는 내부 함수
  `parse_html_to_paragraphs_at_depth(html, depth)`로 감쌌다. `<div>`/`<p><table>` 재귀
  호출은 `depth + 1`로 전달한다. 깊이가 `HTML_PASTE_MAX_RECURSION_DEPTH`(16)를 넘거나
  전체 HTML 바이트 수가 `HTML_PASTE_MAX_BYTES`(400,000)를 넘으면 태그 트리 파싱을 포기하고
  `html_strip_tags` + `flush_text_to_paragraphs`로 평문 문단 폴백한다 — 서식은 잃어도
  붙여넣기 자체는 항상 즉시 끝난다.
- `parse_inline_content`의 `<span>` 처리에서 중복 선형 재탐색을 제거하고, 이미 계산된
  `span_end_tag`(`"</span>".len() == 7`만큼 뺀 위치)를 그대로 내용 경계로 쓴다 — 성능과
  중첩 span 경계 정확도를 동시에 고친다.

## 검증

### 합성 재현 (깊이·span 수를 조절한 Gmail류 마크업)

| | 수정 전 | 수정 후 |
| --- | ---: | ---: |
| depth=40, spans=30 (실사용 근사) | 51ms | 24ms |
| depth=150, spans=150, 71KB (스트레스) | 881ms | 112ms (깊이 상한으로 조기 폴백) |

`cargo test --lib -p rhwp` 3889 passed / 0 failed — HTML 붙여넣기 관련 기존 단위 테스트 34개
(스타일·표·colspan/rowspan·다중 문단 등) 전부 무회귀 통과.

### 로컬 검증 게이트

- `cargo fmt --check`, `cargo clippy --lib -- -D warnings` 통과.
- `wasm-pack build --target web`로 rhwp-studio용 WASM 재빌드, 로컬 rhwp-studio에서 확인 예정.

## 남은 범위

`#4414`(OPEN, 별개) — 문서 간 복사에서 도형이 `[도형]` 리터럴로, 필드/누름틀이 소실되는 문제.
같은 클립보드 붙여넣기 영역이지만 다른 트리거·다른 원인이라 이번 수정에 포함하지 않았다.
