# 크기 보호 수식의 한컴 저장·Print 대조군

Issue: #7642

## 생성과 독립 근거

이 입력은 시험지 원본이 아닌 합성 대조군이다. 정상 한컴 2024 저장본
`../operator-script-saved-2024.hwp`를 RHWP로 HWPX 내보내기한 뒤, 64개 수식의
`hp:sz@protect`만 0 또는 1로 지정했다. 스크립트·글꼴·글자 크기·양의 저장 폭과 높이·
문단 순서·저장 줄은 그대로 유지했다. HancomEQN·HYhwpEQ 두 face와 900·1200 HWPUNIT,
각 16개 스크립트를 사용한다. 원본 자체의 정상 저장 정보와 수동 변경 범위를 구분한다.

로컬 HWPX의 저장 제품 메타데이터에 따라 MCP `engine: 2020`을 명시했다.
실제 한컴은 `11.0.0.9136`, `pdf_print_method: 0`,
`pdf_output_mode: hancom2020_pdf_driver_one_up`다. 모든 PDF는 원본 용지·한 쪽씩·전체 8쪽의
독립 Print 출력이며, RHWP PDF 또는 Save As PDF가 아니다. raw HWPX Print와 정상 한컴
HWP 저장 후 Print를 각각 보존한다.

정상 한컴 HWP를 다시 읽으면 미보호 0/64, 보호 64/64이며 속성 차이는 크기 보호 비트뿐이다.
스크립트·글꼴·글자 크기·저장 크기가 동일하다. raw 두 입력의 Print는 96dpi 전쪽 PNG가
바이트 단위로 동일하다. 정상 보호 HWP의 2020 Print 글리프·글꼴·원점·크기도 기존 2024
Print와 전쪽에서 대조했다. 보호 속성이 표시 글리프 종류를 바꾸지 않는다는 독립 근거다.

## 파일과 Print 작업

| 파일 | 바이트 | SHA-256 |
| --- | ---: | --- |
| `operator-unlocked.hwpx` | 14,373 | `af767a3a114510aa536b2cff6b2156528417419afdd903a981bf4a9bbe457ab7` |
| `operator-unlocked-raw-2020.pdf` | 31,327 | `57a779e31923b06c1dc8cf9420af49f4d62342450a412546607d2291ed2fb736` |
| `operator-unlocked-saved-2020.hwp` | 30,208 | `84a1a5f5fabf3e3ee242ba1ac45fc842abed36baa16ea460a8f2c2b3c03bb3ca` |
| `operator-unlocked-saved-2020-2020.pdf` | 31,327 | `796f949eaedbcc4ec2eeea6547667e6d896782cecce31e49ac9d2abe77ee5a93` |
| `operator-locked.hwpx` | 14,373 | `601639654bd1bcd04f49bfd53cc602bdebc984b23bf1890683a05aa18126f85a` |
| `operator-locked-raw-2020.pdf` | 31,327 | `75358eb109b44400468ba2d353fdb364fe78887c944e9def838689bedbe67c14` |
| `operator-locked-saved-2020.hwp` | 30,208 | `9a6f692b3196439cff9d41304a53d69c64657825858e98b7d6b111972cfec5e0` |
| `operator-locked-saved-2020-2020.pdf` | 31,327 | `1e5dcd09284cfe7d5c582fb4a455b989361ede7a20312a2783e719b486d145d6` |

| MCP 출력 | 형식 | Job |
| --- | --- | --- |
| `operator-unlocked-raw-2020.pdf` | `pdf` | `9ead9273-65bb-46b7-9007-2e34df199a44` |
| `operator-unlocked-saved-2020.hwp` | `hwp` | `4c273cee-8acd-43b1-9807-611d7cacb80b` |
| `operator-unlocked-saved-2020-2020.pdf` | `pdf` | `17ce0541-abfb-4e4b-99fa-fc888ac13783` |
| `operator-locked-raw-2020.pdf` | `pdf` | `a38e09bd-fc30-4f21-92e0-a77a30223423` |
| `operator-locked-saved-2020.hwp` | `hwp` | `c6bfe7b8-5a45-4907-b3f6-066b09503b91` |
| `operator-locked-saved-2020-2020.pdf` | `pdf` | `34cba24e-600b-4545-9ba7-a351e54d4b95` |

## 회귀 추가 전 검증

제품 source는 `03073384fdf94e73f9328358cf7e90dbc8a1439e`에 표시·저장 보정 두 파일을
적용한 작업 트리다. 제품 diff SHA-256은 `5ad06ca539a053f2cd32a3373c68b94fa02e68a3dcbd8fdf27248ff6005064f3`이며,
Native 1745와 fresh WASM 1746의 전 source hash가 동일하고 빌드 중 불변임을 확인했다.
커밋 이후 제품 hash 연결은 ignored 실행 기록으로 보존한다.

네 입력 모두 전체 8쪽이 Print와 일치하고 Native 최저 **99.18280%**,
fresh WASM 최저 **99.16442%**다. 보호 입력의 Native 최저는 보정 전 **97.94494%**,
WASM 최저는 **98.62899%**였다. 미보호 대조군은 보정 전후 같은 점수를 유지했다.
RHWP로 다시 저장한 보호·미보호 HWPX도 각각 MCP Print로 독립 재출력하고 전쪽 대조해
동일한 최저값과 쪽수 일치를 확인했다.

| 페이지 | Native | fresh WASM |
| ---: | ---: | ---: |
| 1 | 99.87685 | 99.89621 |
| 2 | 99.67568 | 99.67056 |
| 3 | 99.23047 | 99.16442 |
| 4 | 99.18280 | 99.17775 |
| 5 | 100.00000 | 100.00000 |
| 6 | 100.00000 | 100.00000 |
| 7 | 100.00000 | 100.00000 |
| 8 | 100.00000 | 100.00000 |

두 face·두 크기의 조합식과 7·8쪽 첨자를 같은 영역·배율로 직접 확대 판독했다.
보호 속성 때문에 바뀌던 아래첨자·위첨자·뒤 항의 원점이 Print와 맞도록 개선됐다.
Native/WASM/Print의 잉크 농도와 안티앨리어싱 차이는 남는다. 실루엣 100%를 모든 색상 픽셀의
동일성으로 해석하지 않는다. 7·8쪽은 양쪽 100%이고 독립 Print 원점 대조·확대 판독을 마쳐
핵심 조합식과 위아래첨자의 좌표를 ±2 CSS px로 보호한다. 소유·순서·누락·중복 검사도 유지한다.

## 정식 검사와 증적

새 이름의 `tests/cases/verified_hancom_size_protection.rs`에서 보호/미보호 두 입력 사이의
글리프·기준선·원본 소유 관계, HWPX 저장·재열기 후 보호 속성과 내용 보존, 완료 페이지의
독립 Print 좌표를 검사한다. 전체 SVG 해시나 현재 실측값에서 만든 golden을 사용하지 않는다.

다음 파일은 Git에 넣지 않는
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/` 아래에 있다.

- `stage1731-size-protect-mcp.json`: 원본 Print·정상 저장·정상 저장본 Print 6개 작업.
- `stage1733-size-protect-print-pair/receipt.json`: 전쪽 독립 Print의 동일 raster.
- `stage1737-size-protect-model/`: 정상 한컴 저장본의 실제 속성·내용·크기 비교.
- `stage1745-current-native.json`, `stage1746-fresh-wasm-build.json`: 제품 source·diff·producer hash.
- `stage1747-current-native-scope/receipt.json`, `stage1748-new-base-wasm-scope/receipt.json`:
  네 입력의 전쪽 Native/fresh WASM TSV와 전체 쪽수.
- `stage1749-size-protect-review/receipt.json`: 전쪽 TSV 산출 후 대표 페이지 review 생성.
- `stage1752-size-protect-reexport-mcp.json`: RHWP 저장본의 독립 MCP Print.
- `stage1754-current-native-scope/receipt.json`, `stage1755-new-base-wasm-scope/receipt.json`:
  RHWP 저장본 2개의 전쪽 검증.
- `stage1756-size-protect-independent-origin-check.json`: 2020/2024 독립 Print 원점 대조.
- `stage1760-size-protect-detail-review.png`, `stage1764-size-protect-detail-review.png`: 확대 판독.
- `stage1767-protected-regression/receipt.json`: 같은 새 검사와 원본의 수정 전후 실행 기록.

이 보정은 시험지 8쪽 하락 원인의 해결 증거가 아니다. 조사한 시험지 538개 수식은 보호가
꺼져 있어 별도로 수식 원점과 표시 경로의 하락 원인을 검토한다. 이 대조군의 통과를 전체
브랜치 검증 완료 또는 PR 준비 완료로 확대하지 않는다.
