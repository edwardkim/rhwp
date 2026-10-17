# PDF 가져오기 공동 처리 시각 증거

Runtime source: `9c6c48c091e0780f805d91a045561b5994c7a6aa`.
Native CLI SHA-256: `f7ab7531f2beab29f3a28f5f50631479db2a4984dfbe3c1da4b3e2774df5503f`.
Fresh Docker WASM SHA-256: `01f4831e486ef0cb7f195d4b0fae3c17ee6e90848409e6b607d94d3df67ba962`.

96 DPI, Print/compat 2022. 원본 PDF 자체의 독립 Poppler 출력과 Native Skia / WASM Canvas를
같은 쪽에서 비교했다. 실제 실행한 Native CLI는 runtime commit 직전의 동일 소스를 컴파일한
것이며 관련 source bytes를 commit과 대조했다. 이후 test·sample·문서·PNG만 추가했다.
전체 페이지 TSV 원문은 최종 PR 본문에 보존하며 원본 TSV/로그/중간 JSON을 Git에 넣지 않는다.

- `image-count`: pdf/issue7966/exceeds-32.pdf 6·7쪽. 연결선·가림 순서·글꼴·affine clip을 판독한다.
- `operators`: pdf/pr7704/6778-square-2020.pdf 12쪽. PostScript font 정체성과 단어 간격을 판독한다.
- `image`: samples/issue7959/bounded-import-path.pdf 1쪽. 색 공간의 q/Q·Form 범위와 검정 배경 누출을 판독한다.

공동 처리의 359쪽은 양쪽 raw 90% 이상이다. 16 MiB 생성본 1쪽은 사용자가 자원 정책
검증으로 범위를 한정했으며 기존 Helvetica 차이는 백로그 B1이다. 이 예외를 다른 원본의
출력 gate에 확대하지 않는다. parserAllocationBoundsProven=false를 유지한다.

입력 SHA·생성 절차는 samples/issue7957/README.md와 pdf/issue7966/README.md,
최종 검증은 mydocs/working/task_m100_7957_stage11.md에 연결한다.

텍스트 가림은 `text-paint-order-before-after.png`, 연속 글자색 회귀는
`text-paint-style-before-after.png`의 실제 이전/보정 출력으로 구분한다. 실루엣 점수와
색상·소속/순서의 정식 검사 및 직접 판독을 함께 사용한다.
