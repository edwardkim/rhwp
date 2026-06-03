# Stage 9 분석 — Task #1139

## 배경

작업지시자가 Stage 8 반영 후에도 `3-09월_교육_통합_2022.hwp` 9쪽이 한컴오피스와 다르다고 재보고했다.

이번 단계에서는 작업지시자가 지정한 두 자료를 기준으로 미주 처리 방식을 재검토한다.

- `mydocs/tech/한글문서파일형식_5.0_revision1.3.md`
- 한컴 도움말: `주석 모양: 미주 모양`

## 한컴 도움말 기준

한컴 도움말의 미주 모양 설명은 다음 의미를 갖는다.

- 미주 내용은 본문에서 번호가 삽입된 쪽과 무관하게 `문서의 끝` 또는 `구역의 끝`에 한꺼번에 표시된다.
- `구분선 넣기`가 켜져 있으면 본문과 미주 내용 사이에 구분선을 긋는다.
- 여백은 세 종류다.
  - `구분선 위`: 본문과 구분선 사이 간격
  - `구분선 아래`: 구분선과 미주 내용 사이 간격
  - `미주 사이`: 앞 번호 미주 내용과 다음 번호 미주 내용 사이 간격
- 번호 매기기는 `앞 구역에 이어서` 또는 `현재 구역부터 새로 시작`을 선택한다.

즉 미주는 단순히 `Endnote.paragraphs`를 본문 뒤에 붙이는 구조가 아니라, 문서/구역 끝에 생기는 주석 영역이며 시작 지점에 구분선과 여백을 소비한다.

## HWP 5.0 스펙 기준

스펙 표 133 `각주/미주 모양`은 `HWPTAG_FOOTNOTE_SHAPE`에 다음 필드를 둔다.

- 속성
- 사용자 기호
- 앞 장식 문자
- 뒤 장식 문자
- 시작 번호
- 구분선 길이
- 구분선 위 여백
- 구분선 아래 여백
- 주석 사이 여백
- 구분선 종류
- 구분선 굵기
- 구분선 색상

표 134의 속성 비트도 현재 구현과 맞춰 다시 봐야 한다.

- bit 8~9: 미주 배치 위치
  - `0`: 문서의 마지막
  - `1`: 구역의 마지막
- bit 10~11: 번호 매기기
  - `0`: 앞 구역에 이어서
  - `1`: 현재 구역부터 새로 시작
- bit 13: 텍스트에 이어 바로 출력할지 여부

## OWPML 스키마 기준

로컬 OWPML 스키마의 `noteSpacing`은 세 속성을 명확히 구분한다.

- `betweenNotes`: 주석 사이 여백
- `belowLine`: 구분선 아래 여백
- `aboveLine`: 구분선 위 여백

이 이름 기준으로 보면 렌더링/조판에서 사용해야 할 의미는 다음이다.

- `aboveLine`: 미주 시작 전 본문과 구분선 사이
- `belowLine`: 구분선과 첫 미주 내용 사이
- `betweenNotes`: 서로 다른 미주 번호 사이

## 현재 raw와 모델 매핑

대상 파일의 두 번째 `FOOTNOTE_SHAPE` raw는 다음이다.

```text
00 00 00 00 00 00 38 bb 09 ff 01 00 5d 37 00 00 00 00 40 02 c0 07 0a 09 59 b8 59 00
```

현재 파서는 이를 다음처럼 읽는다.

- `prefix_char`: `문`
- `suffix_char`: `U+FF09`
- `separator_length`: 약 `50.0mm`
- `separator_margin_top`: `0.0mm`
- `separator_margin_bottom`: `0.0mm`
- `note_spacing`: 약 `2.0mm`
- `raw_unknown`: 약 `7.0mm`
- `separator_line_type`: `0x0a`
- `separator_line_width`: `0x09`
- `separator_color`: `0x0059b859`

작업지시자가 제공한 한컴 UI 값은 다음이다.

- `구분선 위`: `0.0mm`
- `구분선 아래`: `2.0mm`
- `미주 사이`: `7.0mm`

따라서 이 샘플에서 현재 모델 필드의 의미는 실제 UI와 다음처럼 대응한다.

- `separator_margin_top` → 구분선 위
- `note_spacing` → 구분선 아래
- `raw_unknown` → 미주 사이

`separator_margin_bottom`은 현재 HWP5 raw에서 `0.0mm`로 보존되는 별도 슬롯이며, 이 샘플의 UI `구분선 아래`가 아니다. Stage 8에서 `raw_unknown`을 단순히 매 미주 뒤에 더한 실험은 페이지 수가 25쪽으로 늘어 실패했다. Stage 9 구현 중에도 `betweenNotes`를 최소 간격으로 보정하면 전체 페이지 수가 24쪽으로 늘어났다. 따라서 이번 샘플에서는 `미주 사이=7mm`가 원본 `LINE_SEG` 흐름에 이미 반영되어 있다고 보고, Stage 9에서는 중복 가산하지 않는다.

## 현재 구현의 핵심 문제

`src/renderer/typeset.rs`의 미주 처리 경로는 다음 구조다.

1. 본문에서 `Control::Endnote`를 수집한다.
2. 섹션 끝에서 각 미주의 내부 문단을 `st.endnote_paragraphs`에 복사한다.
3. 복사한 문단을 `PageItem::FullParagraph`로 본문 흐름 뒤에 붙인다.

이 방식은 다음 한컴 동작을 빠뜨린다.

- 미주 영역 시작 구분선 표시
- 구분선 위/아래 여백 소비
- `미주 사이` 간격을 서로 다른 미주 번호 사이에 적용
- `FOOTNOTE_SHAPE.attr` bit 8~9의 문서 끝/구역 끝 차이
- bit 13의 텍스트에 이어 바로 출력 여부

현재 page 9 SVG에도 한컴 화면의 초록 미주 구분선에 해당하는 `#59b859` 선이 보이지 않는다. 한컴 화면에서는 답안표 아래에 초록 구분선이 있고, 그 아래에서 `문1)` 미주 내용이 시작한다. rhwp는 이 시작 영역을 생략해 미주 전체 흐름이 한컴과 달라진다.

## 수정 방향

소스 수정 승인 후 Stage 9에서는 다음 순서로 진행한다.

1. `FootnoteShape` 필드의 렌더링 의미를 정리한다.
   - HWP5 대상 파일 기준 `note_spacing`은 `belowLine`, `raw_unknown`은 `betweenNotes`로 사용한다.
   - 기존 HWPX 파서의 `aboveLine` 매핑은 별도 회귀를 보면서 손댄다.
2. 미주 시작 시 한 번만 구분선 영역을 조판한다.
   - `aboveLine`
   - 구분선 굵기
   - `belowLine`
   - 구분선 렌더 항목
3. 서로 다른 미주 번호 사이의 `betweenNotes`는 이번 Stage에서 추가 가산하지 않는다.
   - 대상 파일은 가산 시 24쪽으로 증가해 한컴 기준 23쪽과 어긋난다.
   - 원본 `LINE_SEG` vpos가 미주 사이 간격을 이미 포함한 것으로 판단한다.
4. 미주 구분선 렌더 항목을 추가한다.
   - 길이: `separator_length`
   - 색: `separator_color`
   - 굵기/종류: `separator_line_width`, `separator_line_type`
5. `placement` bit를 확인한다.
   - 이번 샘플은 한 구역이라 `문서의 끝`과 `구역의 끝` 차이가 표면화되지 않지만, 구현 위치는 향후 다구역 문서와 충돌하지 않게 둔다.
6. Stage 8의 `문8)` 10쪽 시작 조건은 유지하되, 9쪽의 `문1)`~`문7)` 세로 위치를 한컴 기준으로 다시 확인한다.

## 검증 계획

- `cargo fmt --check`
- `cargo test --test issue_1139_inline_picture_duplicate`
- `cargo test --test issue_1082_endnote_multicolumn_drift`
- `cargo build --release`
- page 9 SVG에서 초록 미주 구분선 렌더 여부 확인
- page 9/10 dump에서 23쪽 유지, 9쪽 `pi=522`, 10쪽 `pi=523` 조건 유지
- `wasm-pack build --target web --out-dir pkg`
- rhwp-studio 새로고침 후 작업지시자 시각 판정 대기

## 수정 결과

- `PageItem::EndnoteSeparator`를 추가해 미주 시작 구분선을 렌더 항목으로 표현했다.
- `typeset.rs`의 미주 가상 문단 삽입 직전에 미주 구분선 항목을 한 번만 추가한다.
  - `separator_margin_top`은 구분선 위로 사용한다.
  - 대상 HWP5 샘플의 UI 기준에 맞춰 `note_spacing`을 구분선 아래로 사용한다.
  - `raw_unknown`은 미주 사이 값으로 보존하되, 페이지 분할에는 중복 가산하지 않는다.
- `layout.rs`에서 미주 구분선을 실제 선 노드로 렌더한다.
  - 길이: `separator_length`
  - 색: `separator_color`
  - 굵기: `separator_line_width`
- `dumpPageItems` 진단에 `EndnoteSeparator`를 표시하도록 했다.
- `rhwp-studio`에서 `group` 타입 개체도 `개체 속성` 명령으로 `PicturePropsDialog`를 열도록 수정했다.
  - `[다른 풀이]`는 page control layout에서 `type=group`, `paraIdx=518`, `controlIdx=0`으로 노출된다.
  - `getShapeProperties(0, 518, 0)`은 Stage 8의 가상 미주 문단 역해석 경로로 실제 속성을 조회한다.

## 검증 결과

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 5개 통과
  - 전체 23쪽 유지
  - 9쪽에 `EndnoteSeparator` 존재
  - 9쪽 `pi=522` 유지, 9쪽 `pi=523` 미포함, 10쪽 `pi=523` 시작
  - page 9 render tree에 `[다른 풀이]` 텍스트 존재
  - page 9 control layout의 `[다른 풀이]` group 속성 조회 성공
- `npm run build` (`rhwp-studio`)
- `npm test` (`rhwp-studio`)
- `wasm-pack build --target web --out-dir pkg`
- Node/WASM 직접 확인
  - `pageCount=23`
  - page 9 SVG에 `#59b859` 미주 구분선 존재
  - page 9 control layout의 group: `secIdx=0`, `paraIdx=518`, `controlIdx=0`
  - `getShapeProperties` 결과: `width=5102`, `height=1474`

브라우저 자동화는 Codex 내장 브라우저와 Chrome 확장 백엔드가 현재 세션에서 노출되지 않아 직접 클릭까지 수행하지 못했다. 대신 실제 클릭 명령 경로인 `format:object-properties`와 `PicturePropsDialog`의 `group` 타입 허용을 TypeScript 빌드로 검증했고, WASM 속성 API 호출은 직접 검증했다.
