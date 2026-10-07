---
kind: investigation
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-10-07
---

# #7626 — 재조판 높이와 저장 LineSeg의 쪽 예산

- 대상: [이슈 #7626](https://github.com/edwardkim/rhwp/issues/7626), 기준 source `7076f836e2300f7d760d74b58c468cc0f73098a2`.
- 입력: [공개 gist](https://gist.github.com/flamingo8006/cf171c692b6d4c484027832a060da894)의 Base64 원문을 그대로 디코드했다.
  입력 SHA-256은 `8fa018bafb94ae023ed1a9be50cd710bec2a09ee19d76dbc755bb1ba0310ed92`.
  52개 LineSeg 모두 `horzsize=0`, `vertpos=0`, `vertsize=1000`, `flags=0`이다.
  저장 제품 메타데이터는 한컴 2020이며 실제 작성·치환 과정은 확인되지 않았다.
- 독립 기준: 동일 입력을 한컴 2020 `11.0.0.9136`의 1-up Print로 생성한 PDF는 2쪽이다.
  PDF SHA-256은 `2fd348b692e2c555bc33dd26c4feb2f5ed5f33e2f3a7ce2214b515836e5d1546`.
  직접 판독한 1쪽은 4개 절의 본문, 2쪽은 표와 표준 작성지침을 포함한다.
- 기준 source의 Native 출력은 1쪽이며 마지막 본문이 y=1278.4px까지 내려가 본문 하단을 넘는다.
  npm 0.8.4는 2쪽, 0.8.7은 1쪽이다. LineSeg 제거본은 진단용 변형이며 원본 일치의 대용으로 쓰지 않는다.
- 실제 소비 경로: `composer::recompose_stored_lines_in_frame_with_known_square_band`가 새 줄을 생산하고,
  `typeset/paragraph/format.rs`가 새 줄 메트릭을 계산한다. 그러나 `height_for_fit`은 원본 vpos span으로
  다시 낮아지고 `FormattedParagraph::flow_advance_height`가 이를 흐름 높이에 재사용한다.
  원본 pi6의 관측은 total=38.7px, fit=13.3px, advance=13.3px다. 실제 배치는 새 줄을 그려 누적 차이가 난다.
- 수정 방향: 프레임이 거절한 줄의 저장 높이를 재조판의 쪽 예산에 다시 적용하지 않는다.
  유효 저장 줄 및 빈 문단의 공간 계약을 별도 대조한다. 임의 높이 보정·쪽 경계 clamp를 추가하지 않는다.
- 완료 검증: 원본 전체의 Native/fresh WASM·한컴 PDF 비교, 본문·표·뒤 문단의 쪽 소속/순서,
  정상 저장 줄·빈 줄 대조군, 수정 전 실패/수정 후 통과를 확인한다. 기준·시각 증거 미달 범위는 미검증으로 남긴다.
  실행 로그·TSV·중간 산출물은 ignored `output/pr-review/issue7626/`에 보존한다.
