# HFT 기준선 편집의 내부 수식 원점 복구

기존 C429의 정확한 6개 편집 상태와 독립 한컴 2024 Print를 같은 96 DPI/Print/compat2024,
PDFium153/raw2px 조건으로 대조했다. 동일 입력/PDF SHA는 기존 정식 증거와 모두 같다.
수용에 사용하는 입력 6개와 50 MiB 미만 PDF 6개를 원래 바이트 그대로 정식 경로에 보존했다.
임시 로그·진단 프로그램·JSON은 ignored output에만 둔다. 새 HWP 보정 대상으로 확대하지 않았다.

| 상태 | 변경 전 Native | 변경 후 Native | 변경 전 WASM | 변경 후 fresh WASM |
| --- | ---: | ---: | ---: | ---: |
| existing-inline-after-edit-67 | 100.00000% | 100.00000% | 100.00000% | 100.00000% |
| existing-inline-after-edit-80 | 77.50865% | 100.00000% | 77.50865% | 100.00000% |
| existing-cell-after-edit-90 | 62.47505% | 100.00000% | 62.47505% | 100.00000% |
| existing-cell-after-edit-95 | 56.99374% | 100.00000% | 56.99374% | 100.00000% |
| current-cell-after-edit-90 | 62.47505% | 100.00000% | 62.47505% | 100.00000% |
| current-cell-after-edit-95 | 52.90698% | 100.00000% | 52.90698% | 100.00000% |

모든 상태는 각 1/1쪽이며 수식 1개와 기존 owner를 유지한다. 분자·분모·분수선의 내용·순서·
셀 내부 포함·원본 줄 기준선 갱신은 기존 Native 계약 모듈 9건이 검사한다. 기준선 편집 2 FAIL은
기대값 변경 없이 PASS로 복구했다. 집중 36건은 24 PASS / 12 FAIL, 6.245초이며 전체 회귀가 아니다.

양쪽 backend의 inline80/cell95 review와 overlay를 직접 판독했다. 두 backend 모두 정확히
같은 2개의 RGB raster 그룹이며 각 그룹이6개 상태의 별칭을 모두 포함한다. 100%는 독립 PDF와
RGB byte 동일성을 뜻하지 않는다. 메트릭을 유지하며 내부 원점의 중복 기준선 이동만 제거한다.

## 정상 대조군과 잔여

기존 sample16 HWP3/HWP5/HWPX는 각각 전 64쪽 중 수식 노드가 6쪽 2개다. 이 변경 경계의
현재 before/after Native·WASM 6쪽을 비교했다. 표 전체 쪽수는 64/64이나 TSV는 6쪽만이므로
최신 전체 64쪽 통과로 보고하지 않는다.

| backend | 입력 | 변경 전 6쪽 | 변경 후 6쪽 | 하락폭(%p) |
| --- | --- | ---: | ---: | ---: |
| native | sample16-hwp3 | 99.95109% | 99.94633% | 0.00476 |
| native | sample16-hwp5 | 99.94951% | 99.94867% | 0.00084 |
| native | sample16-hwpx | 99.94951% | 99.94867% | 0.00084 |
| wasm | sample16-hwp3 | 99.95281% | 99.94980% | 0.00301 |
| wasm | sample16-hwp5 | 99.95200% | 99.95200% | 0.00000 |
| wasm | sample16-hwpx | 99.95200% | 99.95200% | 0.00000 |

원본과 HWP5의 6쪽 review·overlay를 Native/WASM 각각 직접 판독했다. 같은 backend에서
HWPX의 RGB raster는 HWP5와 동일해 producer/hash로 판독을 연결했다. 최대 하락 0.00476%p로
이 실행 당시 0.1%p와 새 2%p 기준 모두 경계 미만이다. 이는 이전 후보 대비 집중 비교이며
upstream/devel 대비 최신 전쪽 검증을 대신하지 않는다.

기존 CJK 진단 대조군의 미달 두 쪽은 새 완료·수용으로 인용하지 않는다. Native frame p3
84.60691%/parenthesis p3 76.18441%는 PNG byte 동일이며 다른 일부 쪽은 최대3/255 색값
차이만 있다. WASM frame p3 86.35004%는 그대로, parenthesis p3는 76.36706→76.37202%다.
정상 baseline67은 양쪽 100%다. 이 진단의 미지원 복합 구문은 새 보정 대상으로 확대하지 않았다.

## 입력·빌드 출처

- `existing-inline-after-edit-67`: [입력](inputs/existing-inline-after-edit-67.hwp), 기준 PDF `pdf/issue7445-legacy-baseline-edit/existing-inline-after-edit-67-2024.pdf`
  입력 SHA256 `debf81898c3b016dfab102040f0c787a485d4b8d4ebc134f44afd2a763b389f0`, PDF SHA256 `cec6020e72bed90ac3788c84a0d1e181f2b0c8da9d10cd5c32042db51153b97e`
- `existing-inline-after-edit-80`: [입력](inputs/existing-inline-after-edit-80.hwp), 기준 PDF `pdf/issue7445-legacy-baseline-edit/existing-inline-after-edit-80-2024.pdf`
  입력 SHA256 `553b935e8dd1517fe3ed07f9ab6ec9b863bc7c5c40738fbf0dc925504bb7485f`, PDF SHA256 `722a2ab02269415f9497f45e346cb0fc95d106080d8ef80144da1fccd0a05fb3`
- `existing-cell-after-edit-90`: [입력](inputs/existing-cell-after-edit-90.hwp), 기준 PDF `pdf/issue7445-legacy-baseline-edit/existing-cell-after-edit-90-2024.pdf`
  입력 SHA256 `fe2bc3ac3cc32482b9c0065d45cea0c808b0ff0aea119e3dd5722af888046824`, PDF SHA256 `5b3a653f905878d800233978c9bfd519173a893bc878f235ac3edae330ee38ba`
- `existing-cell-after-edit-95`: [입력](inputs/existing-cell-after-edit-95.hwp), 기준 PDF `pdf/issue7445-legacy-baseline-edit/existing-cell-after-edit-95-2024.pdf`
  입력 SHA256 `b57b6162d29e931f1d5912692d409212e8dee0cdc49e3be370e9d1c4f2663686`, PDF SHA256 `bcf6e1d49537bc0afc863c71e017e90310a3749d0ff41c3f12fb0b33bdc50272`
- `current-cell-after-edit-90`: [입력](inputs/current-cell-after-edit-90.hwp), 기준 PDF `pdf/issue7445-legacy-baseline-edit/current-cell-after-edit-90-2024.pdf`
  입력 SHA256 `8a48f0fd6114bce20576d8615215379a8c42ffbfa02d2d32d6c2d69241264c18`, PDF SHA256 `71c59c00b8bab14320b8f3f2df4ba2dffc862033749579b01c3bd8512715b612`
- `current-cell-after-edit-95`: [입력](inputs/current-cell-after-edit-95.hwp), 기준 PDF `pdf/issue7445-legacy-baseline-edit/current-cell-after-edit-95-2024.pdf`
  입력 SHA256 `07821b8b148ff7356cfdb26889208fc65033e9ab4be78a4b72e3b6e82085bfcb`, PDF SHA256 `3322b57c82692f7fc40abbf1d1afd896b3d1c0e3942be5883830f18e443dfc64`

- parent code head `657aa5fa13ad6e896a564eb8b23825b37094cd29` + `src/renderer/equation/mod.rs` SHA256 `5a3bdf9a92f90e34cfc84b8ba19ec98a4a00d20875cf0cb6d8eab24bce78c587`
- Native SHA256 `466f4a7e6ab116560298a2f5ec6826a36c91336aaa4566f6b5d56b93eb6ac8ff`
- fresh WASM SHA256 `354f1c2b58cb938ed4abdc1b18a87bbb59dbb890a8008b903887307fb830903f`
- Mac locked wrapper web/pkg/no-opt 로컬 대체 빌드 230.189초, exit 0. Docker 최적화 통과가 아니다.
- root pkg/Studio public/frozen JS·WASM SHA 일치. 실제 Studio 초기화 응답·최종 소비 경로는 미검증이다.
- 로그·전쪽/집중 원본과 producer는 ignored `rebase-8155-20261022/stage63-legacy-inner-paint-frame/`에 보존한다.

## 대표 review·overlay

| 출력 | 상태 | review | overlay |
| --- | --- | --- | --- |
| before-native | existing-inline-after-edit-80 | [review](before-native-existing-inline-after-edit-80-review.png) | [overlay](before-native-existing-inline-after-edit-80-overlay.png) |
| before-native | current-cell-after-edit-95 | [review](before-native-current-cell-after-edit-95-review.png) | [overlay](before-native-current-cell-after-edit-95-overlay.png) |
| before-wasm | existing-inline-after-edit-80 | [review](before-wasm-existing-inline-after-edit-80-review.png) | [overlay](before-wasm-existing-inline-after-edit-80-overlay.png) |
| before-wasm | current-cell-after-edit-95 | [review](before-wasm-current-cell-after-edit-95-review.png) | [overlay](before-wasm-current-cell-after-edit-95-overlay.png) |
| after-native | existing-inline-after-edit-80 | [review](after-native-existing-inline-after-edit-80-review.png) | [overlay](after-native-existing-inline-after-edit-80-overlay.png) |
| after-native | current-cell-after-edit-95 | [review](after-native-current-cell-after-edit-95-review.png) | [overlay](after-native-current-cell-after-edit-95-overlay.png) |
| after-wasm | existing-inline-after-edit-80 | [review](after-wasm-existing-inline-after-edit-80-review.png) | [overlay](after-wasm-existing-inline-after-edit-80-overlay.png) |
| after-wasm | current-cell-after-edit-95 | [review](after-wasm-current-cell-after-edit-95-review.png) | [overlay](after-wasm-current-cell-after-edit-95-overlay.png) |
| focus-after-native | sample16-hwp3 p6 | [review](focus-after-native-sample16-hwp3-p006-review.png) | [overlay](focus-after-native-sample16-hwp3-p006-overlay.png) |
| focus-after-native | sample16-hwp5 p6 | [review](focus-after-native-sample16-hwp5-p006-review.png) | [overlay](focus-after-native-sample16-hwp5-p006-overlay.png) |
| focus-after-wasm | sample16-hwp3 p6 | [review](focus-after-wasm-sample16-hwp3-p006-review.png) | [overlay](focus-after-wasm-sample16-hwp3-p006-overlay.png) |
| focus-after-wasm | sample16-hwp5 p6 | [review](focus-after-wasm-sample16-hwp5-p006-review.png) | [overlay](focus-after-wasm-sample16-hwp5-p006-overlay.png) |

TSV는 원본 그대로 보존했다. 남은 12건·전체 nextest·필수 게이트·승인된 문서의 최신 전쪽·실제 소비 검증을 계속한다. PR 준비 완료가 아니다.
