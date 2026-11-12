# #8957 — 연결 그림과 굵기 전진폭의 전쪽 검증

한글 2024 `13.0.0.3901`의 독립 Print와 최신 devel `80b1c354…`/후보 `5cc77a0d…`를
비교했다. 후보는 Native Skia와 최적화 fresh WASM 모두 네 입력의 전체 1쪽에서 raw 100.00%다.
PDFium 96 DPI, 색상 경계 보정 전 2px `silhouette_raw_match_percent`이며 픽셀 동일을 뜻하지 않는다.

| 입력 키 | Native 전 → 후 | WASM 전 → 후 |
| --- | ---: | ---: |
| cover-bold | 9.66 → 100.00 | 74.12 → 100.00 |
| cover-regular | 8.42 → 100.00 | 74.76 → 100.00 |
| tracking | 89.84 → 100.00 | 89.84 → 100.00 |
| mixed | 12.65 → 100.00 | 80.93 → 100.00 |

각 `native/`·`wasm/` 하위에 전후 TSV·원본 PNG, 독립 PDF PNG, review·overlay·확대 판독 PNG를
보존했다. 정확한 생산자·입력·기준 PDF 해시는 `provenance.json`에 있다. 실행 로그와 중간 파일은
비공개 `output/`에만 보관한다. 글꼴 프로그램은 포함하지 않는다.

## 입력과 독립 기준

- `tests/fixtures/issue8957/cover-linked.hwpx`: 기존 sample10 HWPX의 앞 24개 문단을 보존한 1쪽
  표지다. 원래 manifest의 외부 그림 3개는 해결되지 않은 상태다. `cover-regular.hwpx`는 해당
  표지의 글자모양 23번 굵게만 해제했다. 저장 줄·그림 크기·여백·배치를 보정하지 않았다.
- `cover-mixed-images.hwpx`: 같은 굵게 표지에서 첫 그림만 기존 `samples/oracle.gif`로 내장했다.
  다른 두 연결 그림과 section XML은 유지했다. 한컴은 정상 그림 하나만 출력하고 나머지 자리를
  비웠다. 전체 그림을 끈 인쇄 설정이 아니라 미해결 연결 그림의 출력 계약임을 구분하는 반례다.
- 기준 PDF는 `pdf/issue8957/cover-{linked,regular,mixed-images}-2024.pdf`다. 혼재본의 Print job은
  `8663c286-3140-4b3a-a11a-9396b0b3d36a`, method 0, `frame_print_to_pdf_ex_one_up`, preprocess none이다.
- tracking은 기존 `strawberry-tracking.hwpx`와 같은 이름의 2024 기준 PDF다. 12/18/24pt의
  보통·굵게 여섯 행을 대조했다.

## 직접 판독과 보호 계약

양쪽 경로에서 제목과 14자 본문의 기준선·간격·가운데 정렬을 같은 영역으로 확대해 확인했다.
보통 글자의 위치·간격은 유지되고 굵게 글자의 전진폭이 복원됐다. 정상 Oracle 그림의 위치·내용을
보존하면서 미해결 연결 그림의 화면용 대체 상자만 인쇄에서 없어졌다. 누락·겹침·줄바꿈 변화는 없다.

`alignment.json`의 14자 원점은 최신 devel SVG와 같으며 v0.9.0 대비 −4.26pt를 유지한다.
첫 원점 `(211.62, 708.82)pt`, 크기 `18pt`와 기존 Print 행렬을 유지했다. Microsoft Print의
비트맵 내부 획 확장·반올림 차이를 공통 조판에 추가하지 않았다. 그 분리 근거는 이전
[`tracking/`](../tracking/) 기록을 따른다.

새 연결 그림 검사는 화면/인쇄 표시, 정상 그림 하나의 출력, 같은 입력 내용·문단 소유·배치·쪽수
보존을 검사한다. 기존 보안 검사는 편집 표시를 명시해 basename만 노출하는 계약을 유지하고,
인쇄에는 경로와 대체 안내문이 모두 없는지 검사한다. 절대 px 핀을 추가하지 않았다.

전체 회귀·lint·Native Skia 공식 3종·최종 편람 383쪽 게이트는 작업 기록에서 별도로 판정한다.
이 표지 측정만으로 PR 제출 준비 완료를 주장하지 않는다.
