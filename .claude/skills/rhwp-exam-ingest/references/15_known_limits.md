# 15 — 알려진 한계

이 스킬은 한계를 숨기지 않는다. 세 가지는 사용자에게 먼저 말한다.
`exam_paper` 빌더/writer 를 이 PR 에서 고쳐서 없애지 않는다.

## 1. Picture 직렬화 (#182)

rhwp HWPX writer 의 Picture inline 분기는 이슈 #182 계열에서 다룬다.
완료 전에는:

- `media[]` 가 ingest 에 있고 BinData 가 zip 에 들어 있어도
- 한컴오피스가 그림을 안 그릴 수 있다.

에이전트 행동:

- 텍스트(지문·선택지·보기)를 우선 옮긴다. 그게 시험지의 본체다.
- 그림을 넣었으면 "이미지 표시는 writer 한계로 한컴에서 비어 보일 수 있다" 고 고지.
- writer 를 이 스킬 작업에서 수정하지 않는다.

검증: `unzip -l out.hwpx` 로 바이트가 들어갔는지만 확인한다.
한컴 스크린샷이 비어 있다고 해서 ingest 를 되돌리지 않는다.
별도 writer 이슈로 넘긴다.

## 2. 수식은 이미지

복잡 수식(분수, 적분, 행렬, 화학 식)은 HWP Equation IR 로 매핑하지 않는다.
그 마일스톤은 후속이다.

에이전트 행동:

- 수식 bbox 를 crop 해 `type: image` 로 넣는다.
- ingest 에 `latex`, `omml`, `equation` 키를 만들지 않는다 (deny_unknown).
- 한 줄짜리 `E=mc^2` 수준의 평문은 text 로 남겨도 된다.
- F16.

사용자가 "수식을 한컴 수식 편집기로 넣어 줘" 라고 하면
이 스킬의 한계를 말하고, 이미지로 넣을지 물어본다.

## 3. 표는 그림

단순 2×2 도 지금은 Table IR 로 재조립하지 않는다. 표 전체를 Picture 로
캡처한다. 병합·테두리·셀 배경은 후속.

에이전트 행동:

- 표 bbox 를 crop.
- `rows`/`cells` 필드를 ingest 에 추가하지 않는다.
- 표 안의 선택지를 텍스트로 살릴 수 있으면 `choices[]` 가 우선.
  표가 자료(통계 표)이면 그림.
- F17.

표 칸을 나중에 고치고 싶으면 산출 HWPX 를 `rhwp-table-exchange` 로
넘기는 것이 아니라, **원본이 이미 HWP 표일 때** 그 스킬을 쓴다.
이 스킬이 만든 Picture 표는 CSV 왕복 대상이 아니다.

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

> 지문과 선택지는 HWPX 로 옮겼습니다. 그래프/수식/표는 그림으로 넣었습니다.
> 한컴에서 그림이 비어 있으면 rhwp Picture 직렬화 한계(#182)입니다.
> 수식 편집기·표 IR 은 이 경로에 없습니다.

픽스처: `fixtures/matrices/known_limits.json`.
예제: `examples/17_picture_serialization_limit.md`,
`examples/18_equation_as_image.md`, `examples/19_table_as_picture.md`.
