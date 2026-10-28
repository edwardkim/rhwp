# #7419 Stage7 — 사용자 요청에 따른 MCP Print PDF 재산출

## 범위와 실행 계약

2026-10-16 사용자 지시에 따라 `mcp_hwp2024Convert_usage.md`의
`start → status → download`로 작은 용지·partial 폭 부족·wrapper 세 입력을 다시 변환했다.
기존 PDF도 MCP Print 산출물이었으며 이번에는 새 job의 실제 결과를 별도로 검증했다.

- 검토 head: `3bfeef22af90f46e62f5dd76e387bb15dfac100a`.
- 렌더링 source: `f5a4774049fa10cc661f7220de0127d61fa081ba`.
- 원격 devel: `0953598d8e08af5e9149b3e4cdbbbf2f5f2bae64`로 변동 없음.
- 입력의 `rhwp info --json`: 세 파일 모두 metadata 기준 `hancom-office-2024`,13.0.0.1408.
  합성 입력을 실제 한컴 저장본이라고 재분류하지 않으며 이 값은 engine 선택 근거다.
- 요청: `--target pdf --engine 2024 --timeout-seconds 1800`, 파일명은 `<stem>-2024.pdf`.
- 비공개 설정은 기존 `--env-file`로 전달했다. 주소·토큰은 기록하지 않았다.
- 실제 결과: 세 job 모두 succeeded/download success. `engine=2024`,
  `hancom_version=13.0.0.3901`, `worker_bits=32`, `backend=hwp-managed-direct-dll-host`,
  `pdf_print_method=0`, `pdf_output_mode=frame_print_to_pdf_ex_one_up`.
  Print 경로를 실제 status/result에서 확인했으며 PDF Creator만으로 판정하지 않았다.

## 새 기준 PDF

| 입력 | job UUID | PDF 쪽수/바이트 |
| --- | --- | --- |
| page-height-28000.hwpx | 8543d52b-01b2-436b-9d47-513baa1cc020 | 3 / 14,366 |
| partial-nested-width-wrap.hwpx | 6b26616a-158c-4e25-98b0-598600a5146e | 2 / 14,037 |
| partial-nested-wrapper.hwpx | 1b864fe4-9b5a-498f-8912-faa29482e38f | 2 / 13,965 |

다운로드 파일은 기본 작업공간의 `pdf/issue7419-mcp-recheck-20261016/`에 보존했다.
해당 세 로컬 사본만 정확한 `.git/info/exclude` 경로로 제외하고, 동일 바이트의 새 PDF를
검토 worktree의 기존 추적 경로에 반영했다. 기존 PDF는 이전 commit과 Stage6의 frozen
`print-pdfs/`에 보존하며, 옛 PDF 해시를 새 PDF의 해시로 바꾸어 쓰지 않는다.

| 새 추적 PDF | SHA-256 |
| --- | --- |
| [page-height-28000-print-2024.pdf](../../pdf/issue7419/page-height-28000-print-2024.pdf) | `42e48d3c85a14186dd2b9b7a7fada8cfcbdd88906552df9de998c798d31d31f6` |
| [partial-nested-width-wrap-print-2024.pdf](../../pdf/issue7419/partial-nested-width-wrap-print-2024.pdf) | `c159d38e6a2acadd70f295e1c02323168b71e9edd5722b548446d8e9c77557ff` |
| [partial-nested-wrapper-print-2024.pdf](../../pdf/issue7419/partial-nested-wrapper-print-2024.pdf) | `be1e12d3736b35869804451be7a59b972547f40de6a204cb81fe2d5b4d78a6b8` |

## 비교와 검증 범위

다운로드의 client/server SHA·크기·PDF 서명을 확인했다. 새 Print를96dpi,
`pdftoppm -thinlinemode shape`로 전쪽 raster화했고, 기존 기준 PDF의7쪽과 크기·RGB 픽셀이
전부 동일했다. 파일 해시는 다르지만 이번 비교 해상도에서 인쇄 결과 차이는 없다.

코드·원본 입력·글꼴·생산자는 변하지 않아 Stage6의 실제 Native/host fresh WASM 출력 PNG를
재사용했다. 새 Rust/WASM 빌드나 새 head 렌더라고 보고하지 않는다. 새로운 PDF PNG와
같은 Visual Sweep 지표로 Native/WASM 전쪽 TSV를 각각 다시 산출했다.

| 문서 | 현재 Native/Print | 현재 host WASM/Print | 양 경로 전쪽 최저 |
| --- | --- | --- | --- |
| page-height-28000 | 3/3 | 3/3 | 100% |
| partial-nested-width-wrap | 2/2 | 2/2 | 100% |
| partial-nested-wrapper | 2/2 | 2/2 | 100% |

새 Print의 대표 Native/WASM review·standalone overlay도 생성했다. wrapper2쪽의 양 경로
두 종류 이미지를 직접 열어 자식 두 표의 줄 구성·전체 이월·후속 본문을 확인했다.
획 raster 차이는 남으며 관용100%를 엄격 픽셀 동일성으로 보고하지 않는다. 다른 두 입력도
대표 PNG를 생성했고 전쪽 PDF 픽셀이 기존 직접 판독한 기준과 같음을 별도로 확인했다.

같은 새 PDF에 대한 순수 devel 출력은 여전히2/3,1/2,1/2쪽이다. 존재하는 쪽의 측정값도
직전 진단과 같다. 새 PDF 산출로 현재 출력의 Print 일치를 재확인했지만, 기준 devel에 없는
쪽의 일대일 비교를 만들어내지는 못한다. 사용자 선택에 따른 기존 보류는 해제하지 않았다.
표준 Docker WASM 미검증 선택과 PR/push 별도 승인 조건도 유지했다.

## 재현과 보존

기본 작업공간의 ignored
`output/pr-review/pr7433-current-20261016/rebased-0953598/mcp-recheck/`에 아래를 보존한다.

- `convert.py`, `start.log`, `status.log`, `jobs.json`: 실제 요청·job·Print 실행 증거.
- `<stem>-info.json`: engine 선택 근거와 입력 정보.
- `compare.py`, `compare.log`, `comparison.json`: 전쪽 비교·생산자 provenance·PDF/PNG 해시.
- `preserved-pdfs.json`: 추적 PDF의 이전/새 해시와 다운로드 사본의 일치.
- `candidate-native/<stem>/silhouette.tsv`, `candidate-wasm/<stem>/silhouette.tsv`: 각각7쪽.
- `base-native/<stem>/silhouette.tsv`, `base-wasm/<stem>/silhouette.tsv`: 공통 쪽의 진단만.
  manifest는 전체 쪽수 차이와 누락을 유지하며 전쪽 통과로 사용하지 않는다.
- 후보 경로의 `review/`, `overlay/`: 작은 용지3쪽, 폭 부족2쪽, wrapper2쪽의 대표 이미지.

스크립트는 기본 작업공간의 `venv/bin/python`으로 실행했고 변환·비교가 exit0으로 끝났다.
Rust 코드/검사·입력·새 Print 픽셀이 불변이므로 전체 Rust 회귀를 다시 실행하지 않았다.
Stage6의 전체10,454 PASS·lint·Native 결과는 그 검증 source의 결과로 유지한다.
