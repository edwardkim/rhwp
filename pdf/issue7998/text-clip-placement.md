# #7998 B19 — 실제 내용 이동과 머리말·꼬리말 저장 소유

`text-clip-placement/`는 앞 절편의 공개 PDF 연산 대조군을 사용한다. 정상 한컴이
비사각 가림을 저장했다는 증거와 구분하며, 한컴 native clip 호환성 완료를 주장하지 않는다.

## 입력과 독립 기대값

1. `text-clips/{evenodd-hole,curve-clip,parent-intersection}.pdf`의 첫 `ET`까지
   원래 글자·font·metric·clip 연산을 보존하고 열린 `q`만 닫아 `sources/first-*.pdf`를 만든다.
   원문은 `MERGED `이다. `sources-origin.json`에 원본과 생성본 SHA를 기록했다.
2. 실제 PDF import가 만든 TextBox 하나를 보통 Header 또는 Footer의 문단으로 옮긴다.
   `VertRelTo::Para`, `HorzRelTo::Para`를 사용하며 임의 LineSeg/vpos는 작성하지 않는다.
   Footer는 위/가운데/아래 정렬과 6000HU 여백, 여유가 없는 1000HU 대조군이다.
   Header는 위쪽 여백 2000HU, 머리말 여백 6000HU이다. 정상 unclipped 대조군도 둔다.
3. 기준 PDF는 원래 paint에 용지 여백·source shape 높이·요청 정렬에서 산출한 `cm`
   이동만 적용한다. 글자를 rhwp 출력으로 다시 그리거나 그 결과로 위치를 정하지 않는다.
   `contexts-origin.json`과 `generate_references.py`로 이 계산과 PDF 연산을 재현할 수 있다.
   가로 용지는 저장 height가 아니라 실제 물리 높이를 사용한다.
4. HWPX는 같은 실제 IR의 제품 writer 저장본이다. 테스트는 이 저장본 재열기와 별도로
   source PDF→IR의 보통 Footer 정렬도 실행한다. 보조 metadata의 유일한 소유도 검사한다.
   `fixture-sha256.json`은 자료의 bytes/SHA이다. 실행 로그와 중간 JSON은 ignored output에 둔다.

기대 계약은 PDF의 활성 clip이 내용과 같은 graphics translation을 받는다는 것이다.
`parent`의 첫 원문은 교집합 밖에 있어 전부 가려지지만 Paragraph에는 남아야 한다.
곡선 입력은 아주 작은 글자 조각만 노출한다. 빈 PNG나 점수만으로 원문 보존을 판단하지 않는다.

## 검증한 범위

2026-10-19 Native/fresh Docker WASM에서 새 9개 입력 × PDF/HWP/HWPX 전쪽 27개씩
최저 **93.33333%**를 확인했다. 최저는 곡선의 작은 노출 경계이며 두 backend 모두 같다.
기존 클립 8개 × 세 형식의 24쪽도 다시 출력하여 각각 최저 **99.81269%/99.51172%**,
이전 절편과 PNG 바이트 동일을 확인했다. 이전 정상 330쪽은 적용 경로 감사와 기존 증거를
재사용했으며 이번 head 재출력이라고 세지 않는다.

실제 꼬리말 정렬에서 수정 전 내용은 62.37333px 이동했으나 clip 이동은 0px였다.
수정 후 둘이 같은 거리로 이동한다. 검사 기대값은 과거 px pin 대신 source-local 경로와
최종 내용 원점의 관계, 원문 유일 소유, 정렬의 순서와 여유 없는 반례이다.
새 저장 metadata는 Header/Footer도 순회하고 본문과 중복된 ID는 결합을 거절한다.

fresh WASM TSV는 공통 layer의 WebCanvas 출력이다. 별도 실제 Studio CanvasKit에서도
9개 원문·클립과 Header/Footer host 문단 X 삽입/삭제·두 저장 형식 재열기를 확인했다.
Native의 글상자 안 M 편집과 Studio의 host 문단 X 편집은 서로 다른 검증이며 혼동하지 않는다.
2026-10-19 메인테이너가 `E:\tmp\issue7998-text-clip-placement-review\index.html` 및
Studio 7793을 **시각·편집 구조 통과**로 판정했다.

원시 TSV·빌드·입력 provenance·실패 전후 실행과 그림은
`output/pr-review/issue7998/p4-text-clip-placement/` 및 stage12 보고서에 연결했다.
한컴 native 비사각 가림, footnote/master/caption의 목록 순회, 회전/resize,
clip이 있는 실제 table continuation 호출과 strict glyph-only V2는 남은 범위다.
