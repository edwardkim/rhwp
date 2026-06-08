# PR #1327 검토 — HWPX 직렬화 누락 인라인 컨트롤 6종 보강

- PR: edwardkim/rhwp#1327 (author: oksure)
- 연결 이슈: #1326 (HWPX 직렬화 시 인라인 컨트롤 누락 — 저장 시 데이터 손실)
- base ← head: `devel` ← `contrib/hwpx-page-controls`
- 규모: +409 / −2, 3 files
- mergeable: MERGEABLE / CLEAN
- CI: Analyze(js-ts/python/rust) pass, Build&Test pass, CodeQL pass, WASM skip

## 1. 배경 / 문제

`parse_hwpx → serialize_hwpx → 재파싱` 시 일부 인라인 컨트롤이 조용히 사라짐(데이터 손실).
`render_control_slot` 이 미지원 컨트롤을 `_ => {}` 로 버리고 `is_hwpx_inline_slot` 에도
누락. 측정상 pageHiding(49)/pageNum(16)/newNum(12)/header(9)/footer(9)/autoNum(1) 손실.

## 2. 변경 내용

### `src/serializer/hwpx/section.rs` (+main)
- `is_hwpx_inline_slot` 에 6종 추가, `render_control_slot` 에 6종 분기 추가.
- 역매핑 렌더러: `render_page_hiding`, `render_page_num`, `render_new_num`,
  `render_autonum`, `render_header`/`render_footer`(공통 `render_header_footer`).
- 코드↔문자열 매핑 함수: `page_num_pos_to_str`(표150), `page_num_format_to_str`(표134),
  `auto_number_type_to_str`(Picture→FIGURE), `apply_page_type_to_str`.
- 장식문자 `'\0'` → 빈 문자열 처리(`ctrl_char_attr`), pageNum `sideChar` 기본 `'-'` 폴백.
- header/footer 는 중첩 문단(subList) 을 기존 `render_paragraph_parts` 경로로 직렬화.

### `src/parser/hwpx/section.rs` (버그 수정)
- `parse_ctrl_autonum` 의 `autoNumFormat type` 을 `parse_u8` 로 읽어 문자열 enum
  (DIGIT/CIRCLE_DIGIT/…) 을 0 으로 떨구던 결함 → 문자열 매핑으로 정정. **직렬화와 무관한
  파싱 정확도 버그까지 발견·수정** (긍정적).

### `tests/hwpx_roundtrip_integration.rs`
- roundtrip 보존 테스트 4건: pageHiding+pageNum, newNum, header/footer(apply/문단수/첫
  텍스트), autoNum. **카운트뿐 아니라 속성값까지 검증** (역매핑 정확성).

## 3. 로컬 검증 결과

- `cargo fmt --all -- --check`: 클린
- `cargo test --test hwpx_roundtrip_integration`: **22 passed, 0 failed**
  (신규 4건 모두 통과: page_hiding_and_page_num / new_num / header_footer / auto_num)
- `cargo clippy --release` (hwpx/section): 경고 없음
- CI 전체 pass

## 4. 평가

- **정확성**: 매핑이 파서 역방향과 일치(주석에 parse 함수 명시). 파서 autoNumFormat 버그
  동반 수정은 추가 이득. roundtrip 값 보존 테스트로 회귀 가드 확보.
- **범위**: 이슈가 1차로 명시한 3종(pageHiding/pageNum/newNum)을 넘어 header/footer/autoNum
  까지 6종 처리. `form`(10건)은 미포함(이슈에서 "복합 구조 별도" 로 명시) — 합리적 분할.
- **코드 품질**: 함수 분리·주석 양호, 인라인 스타일/하드코딩 우려 없음(상수 매핑은 스펙 표).

### 사소한 관찰(블로커 아님)
1. header/footer subList 속성 일부(`id`, `linkListIDRef`, `textDirection` 등)는 IR 미보존
   값이라 고정 기본값으로 출력 — IR 보존 범위 내 무손실. 추후 IR 확장 시 정밀화 여지.
2. header/footer roundtrip 테스트가 `text_width/height/ref` 수치까지는 비교하지 않음
   (apply/문단수/첫 텍스트만). 보강 여지(필수 아님).
3. `form` 미포함은 의도된 후속 — #1326 에 follow-up 항목으로 남아 트래킹됨.

## 5. 판단(잠정)

**Merge 권장.** 데이터 손실 수정 + 파서 버그 동반 수정 + 값 보존 회귀 테스트로 품질 충족,
CI/로컬 검증 통과. 사소 관찰 1·2는 후속 보강 여지일 뿐 블로커 아님.

> 메인테이너 결정 필요: (a) 그대로 merge, (b) 사소 관찰 2(header/footer 수치 비교) 보강
> 요청 후 merge. 작업지시자 승인 요청.
