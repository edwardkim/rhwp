# #8957 — 딸기 HFT 굵기 전진 검증

기준 devel: `4c876654d86d071d342fc78706c51cfff6ddbde4`. 측정일: 2026-11-12 KST.

## 독립 기준과 입력

- 입력: `tests/fixtures/issue8957/strawberry-tracking.hwpx`.
- 기준: `pdf/issue8957/strawberry-tracking-2024.pdf`, 한글 2024 `13.0.0.3901`의 실제 Print 출력.
- blank HWPX의 기본 용지·여백을 유지하고 가운데 정렬 6개 문단을 만들었다.
  `가나다라마바사 아자차카타파하`를 12/18/24pt에서 보통·굵게로 각각 출력했다.
  모든 언어 슬롯의 선언은 딸기 HFT이며 저장 LineSeg는 없는 재조판 대조 입력이다.
- 제공 `HGSTRBRY.otf`의 SHA와 독립 Print job·입력·PDF·실행 바이너리 SHA는 `provenance.json`에 있다.
  글꼴 파일은 포함하지 않는다. 등록 글꼴 상태만으로 실제 선택을 단정하지 않고
  별도 표지 Type3 경로와 제공 CFF의 윤곽도 대조했다.

## 전쪽 비교와 판독

PDFium, 96 DPI, `silhouette_raw_match_percent`의 색상 경계 보정 전 2px 실루엣이다.
원본과 PDF 모두 1쪽이고 표의 최솟값은 전쪽 최솟값이다.

| 출력 | 수정 전 | 수정 후 |
| --- | ---: | ---: |
| Native Skia | 89.84% | 100.00% |
| 최적화 포함 fresh WASM Canvas | 89.84% | 100.00% |

각 경로의 TSV와 실제 PNG를 그대로 보존했다. `tracking-review.png`에서 같은 영역을
2배로 확대해 글자 순서, 획, 글자·단어 사이 간격, 기준선, 가운데 정렬을 직접 판독했다.
점수 100.00%는 2px 허용 실루엣 지표이며 PDF와 픽셀 또는 파일이 동일하다는 뜻이 아니다.

실제 제공 표지의 글자 영역은 Native 85.47659%, WASM 85.47250%에서 각각 100.00%로 개선됐다.
`fixed-cover-crop.png`와 `fixed-output-check.json`은 **국소 진단**이다. 전체 페이지 게이트를 대신하지 않는다.

## 보존과 적용 경계

- 원본 표지 14자의 기존 SVG 대비 x `-4.26pt`, 기준선과 크기 보존을 재확인했다.
  현재 첫 논리 원점 x `211.62pt`, 기준선 `708.82pt`, 크기 `18pt`다.
- Print 프레임 적용 후 첫 원점은 `(211.50624, 708.04541)pt`이며 SVG PDF의 기존 원점과 같다.
- 보통 표지의 Native 및 WASM PNG는 각각 수정 전후 바이트가 같다.
- 기존 sample10 HWP 509/757쪽은 Native 및 fresh WASM 모두 99.90/99.55%이고
  각 경로의 두 PNG가 각각 수정 전후 바이트까지 같다. 이 두 쪽의 TSV를 전체 763쪽 검증으로 승격하지 않는다.
- `driver-separation.json`: Microsoft 보통 마스크의 오른쪽 8px 확장과 굵게 마스크가 14자 모두 같다.
  이는 관측된 출력 관계이며 드라이버 내부 알고리즘을 확정하는 증거가 아니다.
- `position-separation.json`: Microsoft 타일 원점과 논리 글자 상단의 600dpi 올림을 대조했다.
  남은 비트맵 내부 세로 차이와 가로 획 확장을 공통 조판 상수로 추가하지 않았다.

## 남은 게이트

제공 표지는 연결 그림을 사용할 수 없을 때 한컴 PDF와 rhwp의 placeholder 표시가 다르다.
수정 후 전쪽 raw는 굵게/보통 Native 9.24/8.42%, WASM 76.63/74.76%다.
이 입력의 전쪽 게이트는 `re_review_required`다. 위 독립 텍스트 대조의 통과로 대체하지 않으며
표지 전체 또는 #8957 전체 해결·PR 제출 가능으로 판정하지 않는다.
이 작업의 회귀는 새 텍스트 대조에서 검증한 선언/실제 공급 프로그램/배치 계약만 보호한다.
원본 표지를 활성 golden으로 추가하거나 기존 실패의 기대값을 낮추지 않았다.

## 실행 회귀

새 관계형 2개를 포함해 실제 주 프로그램 소유, 저장 줄·그림 여백, 원본 HFT Print 프레임,
Native 글꼴 등록과 윤곽의 집중 회귀 **57개가 모두 통과**했다. 원본 기대값은 변경하지 않았다.
실행 대상과 개별 결과는 `focused-regressions.txt`에 있다. `cargo fmt --all -- --check`와 수정 라이브러리/신규 suite의 Clippy(`-D warnings`)도 통과했다.

최소 글꼴 환경에서 사용자·공유 글꼴 파일과 Font Registry 조회를 프로세스에 한정해 차단했다.
CoreText 격리 목록 528 URL, system_profiler 237개 기록 모두 OS 기본 글꼴 경로였다.
공개 CFF 읽기는 허용되고 제공 HGSTRBRY.otf의 명시 경로 읽기는 차단되는지 확인했다.
일반 프로세스의 CoreText 목록 810 URL은 유지했다. 공개 Preferred fixture 하나만 추가 허용했다.
격리 프로필 해시는 `provenance.json`에 보존했다. 실제 OTF 공급 시각 검증과 이 회귀 환경은 별개다.
전체 회귀·Native Skia 공식 3종·편람 383쪽 최종 게이트의 완료를 주장하지 않는다.
