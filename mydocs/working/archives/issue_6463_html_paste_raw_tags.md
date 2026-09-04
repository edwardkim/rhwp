# #6463 Gmail류 웹메일 서명 붙여넣기가 서식 없이 raw HTML 태그로 삽입되는 문제 수정

## 무엇을

사용자가 실제로 Gmail에서 복사한 회사 연락처 서명 블록("🇰🇷 서울 사무소 (한국 지사)" 제목 +
전화번호/이메일/주소 목록)을 rhwp-studio에 붙여넣었을 때, 서식 있는 문서로 렌더링되지 않고
`jscontroller`, `jsaction`, `data-copy-service-computed-style` 같은 속성값과 `<span>`,
`<strong>`, HTML 주석이 그대로 문자로 삽입됐다.

## 왜 (원인)

`src/document_core/commands/html_import.rs`의 `parse_html_to_paragraphs`(최상위 파서)는
`<table>`/`<img>`/`<p>`/`<div>`/`<br>`만 블록으로 인식한다.

1. **`<span>`이 `<p>` 밖에 바로 올 때** — 예전 코드는 span 내부(중첩 `<u>`/`<strong>`/주석
   포함)를 여는 태그의 `>` 뒤부터 그대로 `pending_text`에 밀어 넣었다. 실측 문서의 제목은
   `<div><span>...<u><span>...서울 사무소...</span></u></span></div>` 구조라, 바깥 `<span>`이
   이 경로를 타 태그가 그대로 노출됐다.
2. **`<ul>`/`<li>`가 전혀 인식되지 않음** — "기타 태그 무시"로 태그만 건너뛰어 목록 항목이
   문단으로 분리되지 않고, 1번 버그와 겹쳐 항목 전체가 raw 텍스트가 됐다.

## 어떻게 (변경)

- 최상위 `<span>` 처리를 `<p>`와 같은 패턴으로 바꿨다: span 내용을 `parse_inline_content`
  (중첩 `<strong>`/`<b>`/`<em>` 등 서식 해석 가능한 기존 함수)에 넘겨 문단으로 만든다.
- `<ul>`/`<ol>`을 `<div>`처럼 컨테이너로 재귀 처리(`parse_html_to_paragraphs` 재호출)하고,
  `<li>`는 내부 전체(중첩 태그 포함)를 `parse_inline_content`로 한 번에 파싱해 "• " 글머리
  기호를 붙인 문단으로 만든다. 글머리 기호만큼 `char_shapes`의 `start_pos`(UTF-16 코드유닛
  단위)를 밀어 서식 위치를 맞춘다.

## 검증

실제 사용자가 붙여넣은 HTML(1207자, Gmail 클립보드 원본)로 확인:

**수정 전**: 문단 하나에 원본 raw HTML 전체가 텍스트로 삽입 (jscontroller 등 속성 그대로 노출).

**수정 후**:
```
para[0] = "🇰🇷 서울 사무소 (한국 지사)"
para[1] = "• 전화번호: 02-2135-3428"
para[2] = "• 이메일: koreacs@trungnguyenlegend.com"
para[3] = "• 주소: 서울특별시 강남구 도산대로 145 인우빌딩 408호 (우: 06036)"
```
"전화번호:"/"이메일:"/"주소:" 라벨은 `<strong>` 인식으로 굵게 `char_shape`도 정확한 위치에
적용됨(각 문단 `start_pos=2`, 글머리 기호 "• " 2 UTF-16 코드유닛 뒤).

### 무회귀

- `cargo test --lib -p rhwp` 3889 passed / 0 failed — 기존 HTML 붙여넣기 단위 테스트
  34개(스타일/표/colspan-rowspan/다중 문단) 포함 전부 통과.
- `cargo fmt --check`, `cargo clippy --lib -- -D warnings` 통과.
- `wasm-pack build --target web`로 rhwp-studio용 WASM 재빌드.

## 참고

같은 문서에서 붙여넣기가 "응답 없음"으로 멈추는 별개 증상(#6449)이 보고됐으나, 그건 빈
문서·큰 문서(382쪽) 양쪽 모두에서 재현되지 않아 사용자의 특정 문서(자동복구 파일) 상태
특이점으로 추정된다. 이 PR은 그와 무관하게 항상 재현되는 서식 손실만 다룬다.
