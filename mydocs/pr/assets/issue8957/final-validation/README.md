# #8957 최종 코드 검증

검증한 코드 head는 `87a4acee56c4d39c290777a9b6c4bd6f3edbbe57`, base는
`80b1c354c29b7a02fb2b3c14a274da7f5b653ada`다. 이 디렉터리를 추가하는 커밋은 결과 기록만 바꾼다.
명령·실행 시간·바이너리/JS/WASM 해시와 실제 실행 head는 `results.json`에 있다.

| 필수 항목 | 결과 |
| --- | --- |
| fmt, manifest 계약·base 대비 정책 | 통과 |
| Native/WASM/workspace Clippy, workspace build | 통과 |
| 최소 글꼴 집중 회귀 | 37개 통과 |
| 최소 글꼴 전체 회귀 | 13,320개 통과, 기존 설정의 45개 제외 |
| Native Skia 라이브러리 | rhwp 3,852개 통과·기존 12개 ignored, 내부 crate 182개 통과 |
| Native Skia 그림 미지정 / 직접 PDF | 2개 / 36개 통과 |
| locked fresh WASM / wasm-opt | 통과, release codegen-units 16 |
| 표지 굵게·보통·전진폭·정상 그림 혼재 | 각 전체 1쪽, Native/fresh WASM 모두 raw 100.00% |
| 편람 Native `--handbook-gate` | 전체 383쪽 통과, 최저 99.96%, 정본 대비 하락 0쪽 |

표지 TSV 원본은 `native/`·`wasm/`, 편람 TSV는 `handbook-native.tsv`다. PDFium 96 DPI,
보정 전 2px 실루엣이며 마스크·쪽 제외·글꼴 예외를 사용하지 않았다. 편람의 130쪽만 99.96%,
나머지 382쪽은 100.00%다. `handbook-130-review.png`는 왼쪽 독립 PDF, 오른쪽 최종 Native다.
본문 줄과 그림 경계를 직접 판독했다.

최종 표지 PNG 8개는 앞서 직접 판독한 [`linked-print`](../linked-print/)의 대응 PNG와
바이트까지 같다. 최종 Native/WASM의 14자 문구와 WASM 12/18/24pt 보통·굵게도 다시 확대 판독했다.
전진폭·가운데 정렬·기준선·공백이 독립 Print와 맞고 정상 그림·내용 순서가 보존된다.
100.00% 점수는 PDF와의 RGB 픽셀 동일을 뜻하지 않는다.

`alignment.json`은 최종 Native SVG에서 14자 원점·기준선·크기와 Print 행렬을 다시 추출한 결과다.
최신 devel 원점과 같고 v0.9.0 대비 −4.26pt를 유지한다. Microsoft Print의 마스크 내부 이동과
획 확장을 공통 조판에 추가하지 않았다.

최소 글꼴 검증은 사용자·공유 글꼴과 font registry 접근을 제한했고 개인 OTF 읽기 거부를 확인했다.
CoreText에 노출된 기본 OS 글꼴 URL은 528개다. 새 회귀는 저장소의 MIT 합성 글꼴 또는 직접 생성한
PNG를 사용하며 설치된 상용 글꼴에 의존하거나 글꼴 부재 때문에 조용히 건너뛰지 않는다.

초기 전체 실행의 입력 누락은 sparse 검증 사본의 Git 원본 파일과 JSON 참조를 복원해 해결했다.
기존 #1197의 정상 그림 대용 외부 경로 표식은 원 해결 PR을 추적한 뒤 실제 합성 PNG로 바꾸고
모든 z-order 관계 검사를 유지했다. 이후 같은 head의 전체 회귀를 다시 실행해 전부 통과했다.
초기 실패 로그와 실행 로그·브라우저 자료는 비공개 `output/issue8957-stage2/`에만 둔다.
