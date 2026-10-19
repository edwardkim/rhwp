# 한컴 저장본의 연산자 뒤 첨자 대조군

Issue: #7642

## 입력과 독립 기준

`operator-script-saved-2024.hwp`는 합성 수식 스크립트 행렬을 한컴 2024에서 HWP로 정상 저장한
파일이다. 실제 시험지 원본을 축소한 파일은 아니다. HancomEQN·HYhwpEQ 두 face와
900·1200 HWPUNIT 크기에 각 16개 스크립트를 배치했다. 저장 전후 64개 스크립트의 동일성을
확인했다. 등호·사칙·부등호·콜론·괄호 뒤 첨자, 빈 그룹 본체, 선행 고아 첨자가 포함된다.

| 파일 | 바이트 | SHA-256 |
| --- | ---: | --- |
| `operator-script-saved-2024.hwp` | 30,208 | `d29732021cf9f4d35d41d0e230267f037ff7d9fb4675451b32feb22eb1d0ace9` |
| `operator-script-saved-2024.pdf` | 26,166 | `f3ab60bb8a049b5e53223ffc46c097deed3d3efe96a75c531b9da7bc35d4528f` |

MCP 정상 저장 job: `8cbce758-b826-4d55-9892-640c4f5dcee5`.
독립 Print job: `486e14d6-9ff5-4753-bc87-7303ea933b67`.
Windows 한컴 `13.0.0.3901`, engine `2024`, `pdf_print_method: 0`,
`pdf_output_mode: frame_print_to_pdf_ex_one_up`. 전체 8쪽, 원본 용지·한 쪽씩 인쇄다.
PDF는 RHWP 출력이나 PDF 저장 기능으로 만들지 않았다.

## 회귀 추가 전 시각 검증

검증 제품 source SHA: `221c9e8c205601dba3a2b13f304a7d13a7757da9`.
전체 8쪽을 96dpi·2px 관용 내용 실루엣으로 대조했다.
Native 최저 **99.18280%**, fresh WASM 최저 **99.16442%**, 양쪽 쪽수 8/8 일치.
7·8쪽은 양쪽 **100%**여서 해당 쪽의 독립 PDF 글리프 원점을 ±2 CSS px로 보호한다.
첨자·본체·뒤 항의 소유와 순서 검사도 유지한다.

| 페이지 | Native | fresh WASM |
| ---: | ---: | ---: |
| 1 | 99.87685 | 99.89621 |
| 2 | 99.67568 | 99.67056 |
| 3 | 99.23047 | 99.16442 |
| 4 | 99.18280 | 99.17775 |
| 5–8 | 100 | 100 |

1·2·7·8쪽의 대표 review·overlay를 만들고, 7·8쪽의 조합식·괄호 아래첨자·위아래첨자를
같은 좌표·배율로 확대 판독했다. 누락·뒤 항 소실·첨자 소속 오류가 보이지 않았다.
Native와 브라우저·PDF의 잉크 농도와 안티앨리어싱 차이는 남으며, 실루엣 100%를
픽셀 전체의 완전 동일로 해석하지 않는다. 한컴 Print가 비우는 `_{6} C`도 비워야 한다.

## 실행 증거 위치

아래 파일은 Git에 넣지 않는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/` 아래에 있다.

- `stage1642-operator-script-mcp.json`: 저장·Print의 실행 메타데이터.
- `stage1642-independent-script-print-observations.json`: 독립 PDF의 실제 글리프·원점 64행.
- `stage1670-current-native.json`, `stage1671-fresh-wasm-build.json`: source·빌드·산출물 SHA.
- `stage1672-current-native-scope/operator-script-diagnostic/operator-script-diagnostic/silhouette.tsv`:
  Native 전쪽 TSV, SHA-256 `228c02b4d296c1e5874bc50c21838a041ef5a65c82a4e59a71837f853533f1c3`.
- `stage1673-new-base-wasm-scope/operator-script-diagnostic/operator-script-diagnostic/silhouette.tsv`:
  fresh WASM 전쪽 TSV, SHA-256 `40c261fcd24cb47e24c8020e438aa73fc2bf57e8fc529a54b2068105dcc2d34b`.
- `stage1688-operator-direct-native/receipt.json`, `stage1688-operator-direct-wasm/receipt.json`:
  입력·PDF SHA, 같은 source 산출물, 실제 페이지·실행 명령과 대표 PNG.
- `stage1690-operator-detail-review.png`: 같은 좌표에서 Native/WASM/Print를 네 배로 확대한 대조.
- `stage1695-operator-regression/tests.log`: 현재 제품 코드에서 새 이름 검사 3개 통과.

대표 review·overlay와 실행 결과는
[1706단계 보고서](../../../mydocs/working/task_m100_7642_stage1706.md)에 연결했다.
`stage1699-before-operator-regression/tests.log`에서 동일 검사 소스·입력의 수정 전 3 FAIL,
`stage1695-operator-regression/tests.log`에서 수정 후 3 PASS를 확인했다. 수정 전 제품 source는
`a3ac53e07c3fe428415bfe3ad648a4abf66affb5`다. 조합식에서 11개 대신 1개만 남고
`= _{6} C`의 첨자·뒤 항이 소실된 실제 원인으로 실패했다. 빌드 실패를 그 증거로 세지 않는다.
