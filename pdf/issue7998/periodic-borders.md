# #7998 반복 경계의 속성·편집 대조군

## 입력 생성과 독립 기준

`generate_periodic_border_sources.py --output <dir>`는 엔진의 배치 결과를 복사하지 않고
PDF MoveTo/LineTo와 명시적 글자 좌표를 작성한다. 부모 외곽은 Solid, 내부는 1.2pt on /
1.8pt off·0.84pt pen이며 자식은 Solid다. 위상 0.12pt 이동, 마지막 stroke 0.05pt 부족,
0.8pt 펜과 후행 흰 가림은 각각 다른 반례다. 다섯 PDF의 바이트 재현을 확인했다.

`native-dot-widths.hwpx`는 이전 B16의 합성 Solid 부모·자식 표 저장본에서 실제
`set_cell_properties_native`로 부모 네 셀의 사방을 Dot(API 3), width index 3/5/7/11로
설정한 결과다. LineSeg를 수동으로 고치지 않았다. 작성 글꼴은 Arial 명시 대조군이며
폰트 바이너리를 이 fixture에 포함하지 않았다. API 생성본과 한컴 작성본을 혼동하지 않는다.

메인테이너가 한컴에서 인쇄한 `native-dot-widths-print-2020.pdf`는 독립 기준이다.
Creator **Hwp 2020 11.0.0.9166**, Producer HancomPDF 1.3.0.550, 1쪽·A4 가로 용지다.
이외 점선/긴 점선/혼합선 최초 Print 3개와 실제 행·열·스타일 편집 Print 5개는 stage7의
로컬 증거에 보존했다. 실제 MediaBox를 사용하고, 내용 실루엣의 비교를 임의로 crop하지 않았다.

| 파일 | bytes | SHA-256 |
|---|---:|---|
| `compatible-inner-dot.pdf` | 7278 | `e53a8b6c65cf1b6cf9e92a85e35cc7cc22a234ad80c122a6a6230d37b669c187` |
| `native-dot-widths-print-2020.pdf` | 26616 | `96b3f8c753c8f671ff00c12efba9a2f612c769cdb408e4d9991c4a165df67343` |
| `native-dot-widths.hwpx` | 12779 | `1fbd2b9026d07fed1f9bddd30bbe8d24bfb93975d5d3037d5dfb543ff6a038d4` |
| `occluded-inner-dot.pdf` | 7308 | `056b00ebec12bfc86b36c5adbf865277790683f6ada907c44bec5fe8e1c9e12a` |
| `phase-shifted-dot.pdf` | 7278 | `b4e19df761df0f3f3704b74eb7cbe4052a14303f23d4ba046597a165e4f60961` |
| `underreach-dot.pdf` | 7278 | `40730d6ed2028eb473d019d0da39815cf3a65fbe53460cc5d466ff7a15530956` |
| `wrong-width-dot.pdf` | 7272 | `b3ff78468415d91c0fd15745fbfba10d2f306af2cf2ad96aaa66e6fd8c481c06` |

## 확인한 계약과 미확인 범위

600dpi Print에서 측정한 0.2/0.3/0.5/1.5mm의 on/off/pen tick은 각각
7/10/5, 10/15/7, 17/25/12, 52/78/35다. source stroke의 실제 pen도 1 HWP unit
반올림까지 일치해야 한다. 최근접 굵기 index나 시각적 겹침만으로 원래 source를 지우지 않는다.
OWPML `DASH`와 IR `Dot`(API 3)의 이름 차이도 구분한다.

`issue_7998_pdf_periodic_borders`는 실제 두 표의 소유·내용 순서·누락/중복·셀 포함,
Cell BorderFill의 Dot 및 HWP/HWPX 재열기, 행·열 삽입/삭제·명시적 스타일 변경과
네 메트릭의 공개 paint JSON 소비를 보호한다. 기대 on/off는 위의 독립 Print에서 왔으며
제품 helper 값을 다시 호출해 기대값으로 삼지 않는다.

`canvaskit-periodic-borders-issue7998.test.mjs`는 실제 CanvasKit PathEffect와 명시적
무채움·무테두리 사각형의 잉크 부재를 검사한다. 이 작은 stroke 계약은 글자를 제외한다.
글꼴이 포함된 전쪽 화면은 실제 Arial을 공급한 별도 CanvasKit Print로 대조했고,
기본 Studio Canvas2D의 실제 편집·저장 검증과 구분했다. 일반 Arial 번들 배포를 새로 주장하지 않는다.

새 fixture를 등록하기 전에 Native/fresh WASM의 관련 전체 쪽·저장 경로 90% 이상과
2026-10-18 메인테이너 **시각 판정 통과**를 확인했다. producer SHA·전체 TSV·대표 이미지·
실행 명령·수치·수정 전 FAIL 증거는 [stage7](../../mydocs/working/task_m100_7998_stage7.md)에 연결했다.
원본 PDF에서의 의미 표 개수와 저장 이후 편집 동작을 픽셀 점수와 따로 판정했다.

완전히 Dot인 외곽의 부모 인식, 다른 폭/패턴/위상·명시적 PDF dash·bitmap/alpha와
가림 경계의 일반 스타일 대체, merged/unequal span의 독립 출력 검증은 남는다.
이 합성 대조군의 통과를 임의 PDF의 표 복원 완료로 보고하지 않는다.
