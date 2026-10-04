# RowBreak 선행 보정 시각 증거

검증 code SHA: `7380b29a2ce94be692e44e70cc65868cf87bc835`.
통합 base: `8497729b4fb0e071c484fc5740f9bb2400bed437`.
수정 전 비교 SHA: `6b3faf77d8085441f9f26d88d65a49791e910352`.

원본과 기준 PDF는 변경하지 않았다. 출처·해시·빌드 식별·측정 범위는
[provenance.json](provenance.json), 순차 실행한 검증 명령·종료 코드는
[local-validation.json](local-validation.json)에 기록한다.

- Native·fresh WASM 각각 전체 18쪽, 최저 90.53132%, 90% 미달·누락 0쪽.
- 2px 이웃 관용 내용 실루엣 수치이며 경계 정합 7,117픽셀을 포함한다. 정합 전 최저도 90.52231%다.
- 영향 페이지 2–8·11–16쪽을 새로 캡처했다. 두 출력의 해당 13쪽 PNG가 동일하다.
- [Native 전쪽 TSV](native_silhouette.tsv), [fresh WASM 전쪽 TSV](wasm_silhouette.tsv).
- [대표 7쪽 수정 전후](native_before_after_p007.png), [7쪽 WASM overlay](wasm_overlay_p007.png).
- [대표 11쪽 수정 전후](native_before_after_p011.png), [11쪽 WASM review](wasm_review_p011.png).
- [전체 대표 이미지 모음](gallery.html): 5·7·8·11·12·14·16쪽의 수정 전후·review·overlay.
- [개체 기하 비교](geometry-summary.json): 정상 대조군 6종 변화 0건(2px 기준). RowBreak는 의도한 4개 개체의 위치·높이 변화가 있으며 무회귀로 세지 않는다.

수정 전/후/한컴 PDF 3패널은 좌표 이동이나 배율 변경 없이 배치했다.
PDF와 Native의 페이지 높이 1px 차이에는 아래쪽 흰 여백만 추가했다.
글자 굵기·일부 표 배경색·열 폭·작은 테두리 차이는 남는다. 글꼴 예외는 사용하지 않는다.
제공받은 글꼴 원본·로컬 경로·글꼴을 포함한 중간 출력은 공개 자산에 포함하지 않는다.

로컬 검증과 작업지시자의 최종 시각 판정은 완료됐다(2026-10-04). 원격 게시와 GitHub CI는 대기 중이다.
