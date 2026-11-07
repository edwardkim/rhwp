# #7640 fresh WASM 전쪽 검증과 전체 회귀 보류

코드 `e9c789b7ab`의 Mac locked web `--no-opt` 로컬 대체 빌드다. Docker 최적화 결과가 아니다.
루트 pkg·Studio public·실제 새 탭 초기화 응답의 JS/WASM hash를 대조했다.

- 재저장본 21/21쪽, 최저 99.02457%(18쪽), 미달·누락 0.
- 정확한 PR7600 원본 Print를 쓴 원본 21/21쪽, 최저 98.77736%, 미달·누락 0.
- #7375 대조군 50/50쪽은 고정 devel 및 이전 후보와 PNG 50개 모두 같다.
- 같은 고정 devel의 전쪽 WASM과 비교해 두 입력 모두 raw 0.1%p 초과 하락 0쪽이다.
- 11·18·19·20쪽의 단 소속·줄 분리·비겹침·후속 그림과 원본 18쪽을 review/overlay로 직접 판독했다.

TSV는 정본 표시 자릿수를 유지하며 하락 비교는 JSON의 원래 raw 소수값으로 수행했다.
입력·PDF·producer·쪽수·비교의 상세 근거는 validation 및 producer JSON을 따른다.

전체 nextest는 12,640 PASS / 1 FAIL / 45 skipped다. `exam_science.hwp`의 빈 수식 줄에
별도의 글자 상자가 있는 반례를 첫 보정이 놓쳤다. 검사를 완화하지 않고 공통 점유 생산을
보정하며, 전체 해결·PR 준비 완료로 판정하지 않는다. 이후 lint/build gate는 실행하지 않았다.

로그·진단 스크립트와 실패 원본은 ignored output에 보존하며 이 폴더에는 로그를 넣지 않는다.
