# 같은 수식 저장본의 독립 Print 재확인

Issue: #7642

기존 구조 수식과 정렬 칸의 첫 칸 정렬 관측이 달라, 같은 정상 저장 HWP를 내용 수정 없이
한컴 MCP로 다시 Print했다. 기존 입력·기준 PDF를 덮어쓰지 않았다.
엔진은 한컴 Office 2020 `11.0.0.9136`, `pdf_print_method=0`,
`pdf_output_mode=hancom2020_pdf_driver_one_up`다.

| 범위 | Print job | 전체 쪽수 | 기존 Print와 DPI 96 raster가 다른 쪽 |
| --- | --- | ---: | --- |
| structural | `0eada7b1-273d-44aa-94a0-47a871ff1b34` | 30 | [] |
| alignment | `d9610439-6e9b-49ed-8866-eb26b225ebfc` | 6 | [] |
| catalog | `ca90887f-5889-4aa9-ab57-6ab956ac5f15` | 18 | [] |

세 범위 모두 기존 Print와 전쪽 raster가 정확히 같다. 새 출력으로 미달 쪽을 대체하지 않는다.
EQALIGN의 서로 다른 관측은 출력 재현 실패로 해소되지 않았으며, 소유 범위·문법 해석을
추가 대조해야 한다. 이 검사는 RHWP 일치율 또는 PR gate 통과를 뜻하지 않는다.

## 고정 입력과 출력

- structural 입력: `pdf/issue7642-structural-command-diagnostic/structural-hancom-2020.hwp`
  SHA-256: `9cb3eef2652d2caf9c786c7c4cc69ecdb1b1621eb0d749ca8c89e3fbd93cf3fd`
  새 PDF: `structural-current-print-2020.pdf`
  SHA-256: `77b00a2a03fafbbdc4b13abff923313f82ae2387762dfcc59dc7bc640473c336`
- alignment 입력: `pdf/issue7642-script-ownership-diagnostic/alignment-column-hancom-2020.hwp`
  SHA-256: `eb71e976a22b8f33ad052e4ba3a2475de5b65734d6114f88211b08267766ee3d`
  새 PDF: `alignment-current-print-2020.pdf`
  SHA-256: `b757e074e691ae11a46d21890d915e541fd08485d8271456a0821f9289ffdc7c`
- catalog 입력: `pdf/issue7642-equation-catalog-print/equation-catalog-hancom-2020.hwp`
  SHA-256: `5821913b3311590a0a724f93a8f0c5d84966c06bbf42cdd3d8e30fe5664ad3b5`
  새 PDF: `catalog-current-print-2020.pdf`
  SHA-256: `2bcf25a83df1668aefe86810775d0f4973d9479e3b1faad37c3286bdbc5d4e9a`

MCP start/status/download의 JSON·log와 페이지별 raster SHA 비교는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage347-*`에 보존한다.
접근 정보는 private env file로 전달했고 증거에 기록하지 않았다.
