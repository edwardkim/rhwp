# #7642 Stage 86 — 저장 좌표 되감김과 완성 미주 소유의 분리

Issue: #7642

## 독립 출력과 원인 경로

Stage 85 source `acf766cc9`에서 9월 2024 HWPX의 21쪽은 89.89297%다.
원본과 기준 Print는 Stage 85와 같은 파일이다. 저장 미주 모양의 사이 간격은
1984 HWPUNIT(약 7mm)이며, 마지막 수식 뒤 문29의 시작은 Print보다 앞선다.
문28의 마지막 문단은 저장 `vertpos=0`으로 되감기지만 실제 마지막 글줄을
현재 단에서 소비한다. 되감김은 저장 좌표 연결의 유효성을 끊는 신호이며,
앞 미주의 내용·물리 줄 소유가 사라진다는 뜻은 아니다.

`layout.rs`의 `complete_note_boundary`는 물리 소유 확인에
`HeightCursor.prev_layout_para`를 사용한다. 뒤쪽의 되감김 분기는 이 값을
`None`으로 지운다. 따라서 실제 완성 tail이 있어도 다음 제목에서 경계가
인식되지 않고 `owned_endnote_origin`이 발행되지 않는다.
`endnote_spacing::starts_note_after_complete_tail`의 완성 여부와 같은 미주
확인은 유지하고, 앞 항목의 문단 소유는 실제 방출한 `PageItem`에서 읽는다.
그 뒤 기존 점유 끝·문서 간격의 생산 → fit scratch 소비 → `vpos_adjust`의
실제 원점 선택 → 후속 문단의 저장 기준 연결을 대조한다.

## 구현과 반례 범위

- 문서 번호·고정 화면 좌표·되감김 크기를 조건으로 추가하지 않는다.
- 저장 좌표 연결이 끊겨도 같은 단의 완성 미주가 소유하는 간격을 보존한다.
  다른 미주의 첫 문단·완성 마지막 줄을 기존 helper로 확인한다.
- 같은 미주의 이어지는 문단, 미완성 PartialParagraph, 단 첫 항목과 독립
  개체의 경계에 간격이 중복되지 않는지 확인한다.
- 미달 21쪽과 9월 HWP/HWPX·11월 정상 대조군, 크기별 NOT·기호 대조군을
  한 후보 빌드로 검증한다. 90% 이상을 확인하기 전에는 해당 실물 회귀의
  기대값·baseline을 변경하지 않는다. 최신 Native/fresh WASM 전체 TSV와
  확대 판독 뒤 관계형 회귀를 변경하며 기존 검사를 삭제하지 않는다.

## 현재 상태

원인 경로를 코드와 입력 저장 정보에서 확인했다.후보는 바로 앞에 실제
방출한 항목의 문단 인덱스를 기존 `page_item_para_index`로 읽는다.
구분선은 문단 소유가 없으며 단 첫 항목에는 앞 항목이 없다.
소유 확인 뒤 기존 완성 경계 helper·같은 점유 끝·간격·fit·원점 경로를 사용한다.
미완성 조각과 같은 미주에는 기존 helper가 새 경계 원점을 발행하지 않는다.
실행 검증은 아직 미완료이며 이전 source의 통과 결과로 대체하지 않는다.

## Native의 실제 원본 비교

source `96f4f13d491affa89e3d33979230ba7c5c07cfd1`의 고정 Native 빌드는
exit 0이며 바이너리 SHA-256은
`8a7164f0b24d1c291c648c2d2b1e7c7a724cb698251e873498f9ac676a29244b`다.
같은 원본 HWPX·독립 Print를 96dpi 인쇄 프로필로 직접 비교했다.

| 쪽 | 현재 Native 일치율 | 판정 |
| --- | ---: | --- |
| 20 | 90.92339% | 수치 통과 |
| 21 | 98.09455% | 수치 통과·문29 경계 직접 확인 |
| 22 | 97.74390% | 수치 통과 |

21쪽의 같은 입력·Print는 이전 source에서 89.89297%였다.
문28 마지막 수식 줄 상자의 끝은 1000.9, 문29 시작은 1000.9 → 1027.4로
관측됐다. 차이는 문서의 1984 HWPUNIT 사이 간격에 해당한다.
이는 수정 전후의 진단 좌표이며 코드·회귀의 고정 기대값으로 쓰지 않는다.
문29 제목과 두 본문 줄의 상대 순서·쪽 소속·내용 보존, 앞 수식의 위치 유지와
독립 Print의 제목 간격을 `review_021.png`에서 직접 확인했다.

증거는 ignored `output/pr-review/issue7642/failed-inputs-20261008/`
`stage15-current-validation/physical-note-owner-focused-native/all-10/`의
provenance·실행 로그·각 쪽 overlay 지표·render tree·review PNG다.
양쪽 전체 쪽수 23쪽은 일치하지만 이 실행은 20~22쪽의 직접 비교다.
전체 페이지 수치 통과나 fresh WASM 통과로 확대하지 않는다.
이전 source의 전체 원본 결과와 현재 source를 구분하며 전체 TSV와 관계형
회귀 변경·최종 게이트는 계속 검증 중이다.

전용 검증 controller의 exec session은 exit 143으로 종료됐고 controller와
Native 집중 검증 PID가 모두 사라진 것을 확인했다. 완료한 13개 입력의
증거는 보존했으며 이 전용 실행의 fresh WASM은 시작되지 않았다.
Stage 87의 대문자 표시 보정과 이번 미주 경계를 함께 포함한 후보에서
Native/fresh WASM을 검증한다. 종료 원인을 출력 공백이나 완료로 추정하지 않는다.
