# 펼친 부모 표의 물리 상자·자식 앵커 대조군

원본은 `../current-print-inputs-v175/clickhere-wrapper-current-v175.hwpx`이며 SHA-256은
`17a174162fd3fcd1c51bd04a25a577396c44bd51ad1c8ab001f9184b16dc4de3`이다.

offset/padding/flow/inline/height는 원본에서 한 가지 조건만 바꾼 수동 대조군이다.
정상 한컴 생성본이라고 분류하지 않는다. normal-save-hwp는 같은 원본을 실제 한컴2020으로
정상 저장한 독립 대조군이며 원본 입력의 대체물로 쓰지 않는다.

모든 PDF는 실제 한컴2020 `11.0.0.9136`, `PrintMethod=0`,
`hancom2020_pdf_driver_one_up`으로 출력했다. 현재 Native/fresh WASM 전쪽은 모두100%이고
두 출력의 decoded RGB도 같다. 직접 review와 3배 확대는 누적 리뷰 진행151–154에 연결한다.
차트 내부·직접 보정3문서의 최종 전쪽·전체 회귀·backend의 수용을 이 대조군으로 대신하지 않는다.

| 대조군 | 입력 | 입력 SHA-256 | Print PDF | PDF SHA-256 | Native / fresh 최저 |
| --- | --- | --- | --- | --- | --- |
| child-offset | [child-offset-8b1ec78415f1.hwpx](child-offset-8b1ec78415f1.hwpx) | `8b1ec78415f1b35232de4a6f1d5964bd2b5039d15a2049b855a638160c4abfc5` | [child-offset-2020.pdf](/home/tsjang/rhwp/pdf/semanticist21-20261008/wrapper-anchor-counters-v289/child-offset-2020.pdf) | `5b96ff466e5f3f6a3f348448eb4d941bcf6ff56c72264c5505c16c3268519b8c` | 100.00000% / 100.00000% |
| parent-padding | [parent-padding-61e9cbd25806.hwpx](parent-padding-61e9cbd25806.hwpx) | `61e9cbd2580635e282ebc5489988152b458f3eee94f64f8aba749f6d7bc2e6ff` | [parent-padding-2020.pdf](/home/tsjang/rhwp/pdf/semanticist21-20261008/wrapper-anchor-counters-v289/parent-padding-2020.pdf) | `56d0f3b0773b1aeb243db404eddcac04bd74a2745758272cf93239bc27fe7d8f` | 100.00000% / 100.00000% |
| child-flow | [child-flow-faeb7331e8fb.hwpx](child-flow-faeb7331e8fb.hwpx) | `faeb7331e8fb9ecd81df5dc761847197c1e7c1ff99c8cfcf02e4b58d58bbe576` | [child-flow-2020.pdf](/home/tsjang/rhwp/pdf/semanticist21-20261008/wrapper-anchor-counters-v289/child-flow-2020.pdf) | `ae28e12d6665682c8040415f760d35af9f395294c4aac0656408b1d910faf454` | 100.00000% / 100.00000% |
| child-inline | [child-inline-fda0d3d71ef2.hwpx](child-inline-fda0d3d71ef2.hwpx) | `fda0d3d71ef2729fee28c0c685fac5bad5c71943f1346676fdfd99df17d1130d` | [child-inline-2020.pdf](/home/tsjang/rhwp/pdf/semanticist21-20261008/wrapper-anchor-counters-v289/child-inline-2020.pdf) | `1b82b4e57205af99268c67e77ab46540883955455e1b30d929f50ad252324000` | 100.00000% / 100.00000% |
| parent-height | [parent-height-96f2cd69236d.hwpx](parent-height-96f2cd69236d.hwpx) | `96f2cd69236d0be49f62d10c86b39cb20b082384b48946be875cf7d6e7cd4a88` | [parent-height-2020.pdf](/home/tsjang/rhwp/pdf/semanticist21-20261008/wrapper-anchor-counters-v289/parent-height-2020.pdf) | `860ee5d293ba53b51c82eb280e1d268aca5f2c5e42f2c0f86b7cebb78e515a28` | 100.00000% / 100.00000% |
| normal-save-hwp | [normal-save-hwp-bebf8d7399c2.hwp](normal-save-hwp-bebf8d7399c2.hwp) | `bebf8d7399c24fb70c545be6f65af899637d9496f3241e97fb51735957fb5056` | [normal-save-hwp-2020.pdf](/home/tsjang/rhwp/pdf/semanticist21-20261008/wrapper-anchor-counters-v289/normal-save-hwp-2020.pdf) | `cbc0c0bb52abb8cdb26c4da25279b91ea01434a94c6f1dd86fab050133a8b27a` | 100.00000% / 100.00000% |

## 재현과 증적

검증 Native source/binary는 ignored `output/pr-review/semanticist21-20261008/`의
`wrapper-cell-note-native-v315-result.json`, fresh는
`fresh-wasm-wrapper-cell-note-v323-result.json`의 source SHA-256 목록으로 고정했다.
fresh WASM SHA-256은 `f16609ba3bac35ab0292a0c15c2bc36d8becb562834f98090fb1831cc2d517f9`다.
Native CLI SHA-256은 `3d50e1311eb579afd41d0dcf3bafade916b8b763a633083c8f89d2b001340ae3`다.

원본·독립 대조군의 전체17쪽 Native/fresh TSV와 실행 명령·입력/Print 해시는
`wrapper-cell-note-native-fresh-completion-v338.json`,
`wrapper-cell-visual-v320-result.json`,
`wrapper-cell-note-fresh-visual-v326-result.json`에 보존한다. TSV·로그·중간 JSON·SVG는
Git에 포함하지 않는다. `scripts/visual_sweep.py`의 원본/동일 Print 비교와 같은
96dpi print profile / compat2022 / 실제 공급된 일곱 font path를 사용했다.
Native와fresh 명령은 `--wasm-pkg` 유무로 구분하고, 같은 이름을 주어도 입력/Print SHA를 먼저 대조한다.

한컴 job provenance와 대조군 생성 명령은 ignored
`wrapper-anchor-counters-v289-result.json`과 `wrapper-anchor-counters-v289/`에 있다.
본 디렉터리는 이미90% 조건을 충족한 입력/Print를 영구 보존하며, 임의 px golden이나
허용치 완화를 추가하지 않는다.
