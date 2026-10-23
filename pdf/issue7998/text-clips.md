# #7998 B19 — 편집 가능한 텍스트의 실제 클립

`text-clips/`의 PDF는 이미 검증한 공개 `cell-regions/colspan.pdf` 한컴 Print에서
첫 머리글의 활성 PDF clip만 바꾼 경계 대조군이다. 정상 한컴 문서가 비사각 text clip을
직접 저장했다는 증거로 사용하지 않는다. PDF W/W*·q/Q의 기대값은
[Adobe PDF Reference 1.7 §4.4.3](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf)의
paint 계약과 독립 PDF raster를 기준으로 정했다.

- 원문: `MERGED HEADER`, `LOWER LEFT`, `LOWER RIGHT`. 첫 두 text-show에만 활성
  비사각 clip이 있으며 아래 글자는 정상이다. 가려진 글자도 실제 Paragraph에 남아야 한다.
- triangle·curve·파란 배경은 경로대로 paint를 가린다. 흰 덧칠·글자 삭제로 대체하지 않는다.
- 동일한 두 사각형은 even-odd에서 구멍, nonzero에서 채움이다. 부모 교집합과 Form 이동을
  보존한다. q/Q로 복원한 triangle은 이후 글자를 가리지 않는다.
- clipping을 이유로 의미 셀을 새로 추측하지 않으며, 첫 머리글은 원래 편집 가능한
  positional TextBox를 유지한다. 일반 본문/표의 문단 승격은 기존 정상 대조군으로 보호한다.

`origin.json`은 원본 bytes/SHA-256이다. 생성·관측 코드와 이전 무효 대조군도
로컬 `output/pr-review/issue7998/p4-text-clip/`에 보존했다. 초기 생성기는 첫 text-show에
활성화되지 않은 클립을 바꿨으므로 해당 7개 대조군의 통과를 증거에서 제외했다.
최종 원본은 활성 clip 관측을 각각 확인한 `sources-active/`와 같은 바이트다.

동일 source/Arial·96dpi에서 새 8개 입력 × PDF/HWP/HWPX 재열기 24쪽의 최저는
Native **99.81269%**, fresh Docker WASM **99.51172%**이다. 원래 실패 triangle은
89.94524%/89.61347% → 99.97535%/99.51172%이다. 전체 공통 범위는 backend별
321 case/354쪽이며, 기존 330쪽은 직전 B20과 PNG가 같다. TSV 원본은
`native/visual/`, `wasm/visual/`과 각 active 폴더에 보존했다. Native SVG도 같은 원본의
전쪽 비교로 검증한다. 세부 최종 결과는 stage11 보고서에서 연결한다.

2026-10-19 메인테이너가 `E:\tmp\issue7998-text-clip-review\index.html` 및
Studio 7793에서 **시각·편집 구조 통과**로 판정했다. 실제 Native/Studio의 M 삽입·삭제와
두 writer의 rhwp 재열기에서 원문과 clip chain을 확인했다.

## 저장 호환성의 범위

`/RhwpTextClips`와 `META-INF/rhwp-text-clips.json`은 typed IR을 rhwp에서 왕복하기 위한
제한된 보조 metadata다. ID와 현재 원문의 fingerprint가 유일하게 일치할 때만 복원한다.
한컴이 이 확장을 해석하거나 외부 재저장 뒤 보존하는 것은 입증하지 않았다. writer는
한컴 기본 표현에 대응시키지 못한 경계를 실제 content-loss 보고에 남긴다. 따라서 이
검사는 **PDF→IR→paint/rhwp 왕복**을 보호하며, 한컴 native clip 호환성 완료나 B19 전체
완료를 선언하지 않는다. 글자를 raster 그림으로 바꾸지 않는다.

좌표 pin·SVG hash 기대값은 추가하지 않았다. source의 활성 상태·원문 유일 소유/순서,
현재 clip chain과 실제 paint 소비, 양 저장·삽입/삭제·metadata 오결합 거절을 검사한다.
