# Task M100 #2217 Stage 7 - PR 시각 검증 준비

## 목표

Task #2217의 renderer/WASM/Studio 변경을 PR 준비 전에 시각 검증한다.

## 범위

1. `samples/issue2217/20200830.hwp`를 HWP 2020 MCP CLI로 PDF로 변환해 `pdf/issue2217/`에 보존한다.
2. 기준 PDF와 rhwp CLI SVG/render tree 산출의 대표 페이지를 같은 해상도로 비교한다.
3. visual sweep 결과와 대표 비교 PNG를 PR review 문서에 기록할 수 있도록 보존한다.

## 검증 기준

- MCP CLI 결과의 `status`, `run_status`, `validation`, 출력 PDF SHA-256, 페이지 수를 확인한다.
- PDF와 rhwp 결과의 자동 지표는 참고값으로 기록하고, 08서울한강체 다국어 이름 해석과 초기 편집 진입이
  실제 Chrome에서 확인됐는지를 최종 판단의 중심으로 둔다.

## 기준 PDF 생성 결과

- 명령: `hwp2020-mcp-convert --input samples/issue2217/20200830.hwp --target pdf`
  (MCP CLI 환경 파일을 사용하고, 결과는 `pdf/issue2217/`에 보존)
- 결과: `pdf/issue2217/20200830-2020.pdf`, 4쪽, 928,693 bytes.
- PDF SHA-256: `ea0110e5c6325f7d2b22620017791b8ab5e53768f84dfa2c0656390d931c6563`.
- MCP 응답: `status=success`, `run_status=0`, `validation=ok`.

## visual sweep 결과

- 명령: `python3 scripts/task1274_visual_sweep.py --key issue2217-20200830 --hwp samples/issue2217/20200830.hwp --pdf pdf/issue2217/20200830-2020.pdf --pages 1-4 --dpi 144 --rhwp-bin target/release/rhwp --out output/task2217_visual_sweep`.
- 4쪽 PDF, 4쪽 SVG, 4쪽 render tree, 4쪽 비교/overlay/review 패널을 생성했다.
- overlay 자동 참고값: 평균 pixel match `85.10261%`, 평균 ink match 및 visual accuracy proxy `6.05756%`.
- 대표 검토 패널: `output/task2217_visual_sweep/issue2217-20200830/review/review_001.png` 및 `review_002.png`.
- 자동 분석은 2쪽에 `render_tree_frame_tail_overflow`, line/column band drift, large ink region drift를,
  3쪽에 content-bottom drift를 후보로 표시했다. 이는 이 변경이 직접 다루는 로컬 글꼴 별칭/CanvasKit 등록과
  별개인 기존 HWP 2020 PDF 대 CLI SVG 조판 차이다.
- 따라서 이 sweep은 기준 PDF와 잔여 fidelity 축을 보존하는 증적이며, 픽셀 일치율을 Task #2217의 합격 기준으로
  사용하지 않는다. Task #2217의 직접 검증은 Stage 6의 Chrome CanvasKit 실측(4쪽 완료, textarea 활성화,
  글꼴 목록 열기 전 로컬 option 0개 및 열 때 670개 생성)으로 유지한다.
