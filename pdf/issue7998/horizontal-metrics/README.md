# 공백 전진폭과 실제 셀 테두리 계약

독립적으로 작성한 PDF 연산과 문단 주석을 사용한다. 폰트 프로그램은 포함하지 않고,
ArialMT의 실제 Widths·FontDescriptor와 공급 글꼴 SHA를 `*-source.json`에 기록한다.

- `native/`: 독립한 네 변 stroke. 넓은 TJ 공백 15pt, 좁은 공백 2.88pt, 원래 Tw 공백
  15pt를 일반 간격 6pt·합성 한도 밖 27pt와 구분한다.
- `rectangle/`: 같은 셀을 닫힌 `re S`와 공유 선으로 발행한다. 모든 셀의 테두리는
  실제 BorderFill이며 셀 위 Picture/Shape는 0개여야 한다.
- `closed-large/`: 같은 2셀·내용과 더 큰 용지를 사용한다. 실제 행·열 추가 뒤에도
  테두리 속성·내용 순서·겹친 개체 0개를 HWP/HWPX 재열기에서 검사한다.
- `compound/`: 한 `S`로 여러 subpath를 그린 별도 입력이다. 이를 일반 사각형으로
  승격하지 않으며 실제 source paint·행열 band의 첫 slot과 저장 배치를 검사한다.
- `alpha/`, `occluded/`: 반투명·나중에 가린 외곽의 반례다. bbox만으로 불투명한
  표준 테두리를 만들지 않는다. 앞 문장 입력의 frame-only 역할도 검증한다.

`generate.py --paint native|rectangle|compound|closed-large|alpha|occluded --out <새 폴더>
--arial-font <arial.ttf>`로 재생성한다. 14개 PDF는 최종 검증 입력과 바이트까지 같으며
글꼴 프로그램은 복사하지 않는다. PDF 연산의 전진폭과 주석이 독립 기대값이고, 현재
구현이 출력한 CharShape 필드를 다시 읽어 정답으로 삼지 않는다.

기준은 원본 PDF 및 동일한 실제 저장·편집 파일의 독립 한컴 2020 Print다. 최종 Native와
fresh Docker WASM, compat 2022·96DPI·명시한 Arial 공급 조건에서 등록 대상 원본과
관련 저장/편집본이 모두 raw 90% 이상임을 확인한 뒤 등록한다. 공백 전진폭의 ±2px
검사는 원본과 양 저장의 해당 완료 쪽이 양쪽 raw 98% 이상이고 직접 판독까지 완료된
경우에만 사용하며, 셀 내부 포함·줄 순서·누락/중복·후속 본문 관계도 유지한다.

반쪽 클립의 선 농도 반례는 기존부터 미달이며 메인테이너가 백로그·이번 범위 예외로
수용했다. 실패 PDF·전체 TSV는 로컬 증거에 남기되 정상 렌더링 fixture로 등록하지 않는다.
그 예외를 이 폴더의 등록 조건이나 전체 P5 완료로 해석하지 않는다.

[stage29 보고서](../../../mydocs/working/task_m100_7998_stage29.md)에 정확한 source/runtime
SHA·명령·전쪽 TSV·대표 compare/overlay와 직접 판정을 연결한다. ignored 증거 위치는
`output/pr-review/issue7998/p4-horizontal-metrics/anchor-native-borders/`다. 앞선 거절된
`anchor-wrap/`의 숫자상 통과를 최종 편집 구조의 통과로 대신하지 않는다.
