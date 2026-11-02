# 15 — 알려진 한계

이 스킬은 한계를 숨기지 않는다. 세 가지는 사용자에게 먼저 말한다.
`exam_paper` 빌더/writer 를 이 PR 에서 고쳐서 없애지 않는다.

## 1. Picture writer와 ingest 이미지 빌더의 차이

`src/serializer/hwpx/picture.rs`의 Picture 직렬화는 구현돼 있다.
하지만 `build-from-ingest`는 `--media-dir`의 존재만 확인하고 이미지 바이트를 읽지 않는다.
`src/document_core/builders/exam_paper.rs`는 image 블록을 `[이미지: …]` placeholder 텍스트로 만든다.
공유 지문·boxed 내부 이미지와 모든 placement에도 같은 제한이 적용된다.
따라서 올바른 경로·크기·배치를 주어도 이 경로에서는 Picture/BinData가 생성되지 않는다.
과거 #182의 writer 결함으로 오인하거나 경로·배치 변경만으로 재시도하지 않는다.

- **그림이 필수인 입력은 F18로 중단한다.** 텍스트만 있는 결과를 완성된 시험지로 제공하지 않는다.
- CLI 종료 코드 0, export-text 통과, crop 파일 생성은 이미지 삽입 성공을 뜻하지 않는다.
- 제한 재현 산출물을 검사할 때는 `unzip -l`과 section·manifest에서 이미지 파일·Picture 참조의 부재를 확인한다.
- 실제 한컴 표시는 별도 검증이다. 소스 확인이나 ZIP 검사로 표시 검증을 대체하지 않는다.
- 이 문서 정정 작업에서 빌더나 writer 구현을 함께 변경하지 않는다.

## 2. 수식과 수식 이미지

이 ingest 스키마에는 편집 가능한 수식 필드가 없다. `latex`, `omml`, `equation` 키를
발명하지 않는다. crop으로 수식 이미지를 준비할 수 있지만 현재 빌더에는 넣을 수 없다.
수식 이미지가 필수이면 F16을 설명하고 F18로 중단한다.
한 줄짜리 `E=mc^2` 수준의 평문으로 요청을 충족하는 경우만 text로 표현할 수 있다.

## 3. 표와 표 이미지

이 ingest 스키마에는 편집 가능한 표 구조 필드가 없다. `rows`/`cells`를 추가하지 않는다.
표 전체를 crop해도 현재 빌더로 Picture를 삽입할 수 없으므로 표 이미지가 필수이면
F17을 설명하고 F18로 중단한다. 텍스트 선택지만으로 요청을 충족하는 경우에는 `choices[]`를 쓴다.
원본에 이미 편집 가능한 HWP 표가 있을 때만 `rhwp-table-exchange`의 CSV 왕복 대상이다.

## 그 밖의 한계

| 한계 | 행동 |
| --- | --- |
| 손글씨 답·낙서 | 무시 |
| 2단 조판의 정확한 단 위치 | 읽는 순서(왼→오, 위→아래)로 문항만 재구성. 단을 재현하지 않음 |
| 페이지 머리글 장식 | `header_text` 문자열만 |
| 원 문자 이외의 특수 폰트 | 함초롬바탕 fallback |
| 색상·형광펜 | 무시. 텍스트만 |
| 듣기 평가 QR/음원 | 무시 |
| OMR 답안 마킹 | 만들지 않음 |

## 사용자에게 쓰는 한 줄

> 현재 시험지 ingest 빌더는 그림 대신 placeholder 텍스트를 만듭니다.
> 이 입력에는 그래프·수식·표 이미지가 필요하므로 완성된 시험지로 제공할 수 없습니다.
> 경로·크기·배치를 바꾸어도 해결되지 않아 여기서 중단합니다.

픽스처: `fixtures/matrices/known_limits.json`.
예제: `examples/17_picture_serialization_limit.md`,
`examples/18_equation_as_image.md`, `examples/19_table_as_picture.md`.
