# #8978 채우기 무늬 종류 시각 검증

HWP 무늬 종류를 0부터 5까지로 읽도록 고친 변경을 Native와 fresh WASM으로 비교한 기록이다.

## 입력과 기준 PDF

| key | 원본 | 기준 PDF |
| --- | --- | --- |
| fill-patterns-0-6-hwp | `samples/issue8978/fill-patterns-0-6.hwp` | `pdf/issue8978/fill-patterns-0-6-2024.pdf` |
| fill-patterns-0-6-hwpx | `samples/issue8978/fill-patterns-0-6.hancom2024.hwpx` | `pdf/issue8978/fill-patterns-0-6.hancom2024-hwpx-2024.pdf` |
| hwpspec | `samples/hwpspec.hwp` | `pdf/hwpspec-2024.pdf` |
| 59043-regulatory-analysis | `samples/issue1921/59043_regulatory_analysis.hwp` | `pdf/issue1921/59043_regulatory_analysis-2022.pdf` |

- `fill-patterns-0-6.hwp`는 7행 표의 오른쪽 칸에 무늬 종류 0부터 6까지를 하나씩 넣은 합성 입력이다.
  `setCellProperties({fillType:'solid', fillColor:'#ffffff', patternType:N, patternColor:'#000000'})`로 만들었다.
- `fill-patterns-0-6.hancom2024.hwpx`는 한글 2024(13.0.0.1408)가 위 HWP를 HWPX로 저장한 파일이다.
- 두 PDF는 한글 2024에서 Microsoft Print to PDF로 문서 전체를 1-up 출력했다.
- hwpspec와 59043은 무늬 2·3을 쓰는 기존 한컴 저장 문서이고, 기준 PDF도 저장소에 있던 것이다.

| 파일 | SHA-256 |
| --- | --- |
| fill-patterns-0-6.hwp | `26062124ea6184245e967b0dbba5a2ee129a076188a0a1d8e6ff65ece5321b2b` |
| fill-patterns-0-6.hancom2024.hwpx | `4733e537608f22f6c9b6a144f97830baae998705bd610d0338807cf668075429` |
| fill-patterns-0-6-2024.pdf | `be829f0f4647f895ca9e293bad8c36ad2ea95c14442dfe8cfe4fa86d2bb68ee3` |
| fill-patterns-0-6.hancom2024-hwpx-2024.pdf | `0493000b0572a3f1de68d694fc52181d74397aadeaaa2cdad014d2be8ea288f1` |
| hwpspec.hwp | `64df877f3c4accda0111ecf39d837da889741bb821b220cf61b6c126dc88364d` |
| hwpspec-2024.pdf | `9c8110347efd719c75098b97509bd4ddf4c5eb6c838d4a6d78982da5f610512b` |
| 59043_regulatory_analysis.hwp | `da9f9c9862475d1710f6ecf93e3ce134726054fe0be0d9523b9d60d1a90c82ef` |
| 59043_regulatory_analysis-2022.pdf | `fade38ba20113ab52eaf42c55eec207943fe66e1b18efdeee06bfa65a1d64fa1` |

## 실행 환경

- base: devel `41c889cfd`. head 코드: `7de77727c`(`src` tree `094f14c0b933c8d42889f25abfd11090beee72d2`).
- Windows 11, Rust 1.93.1. 두 쪽 모두 같은 환경에서 빌드했다.
  - Native: `cargo build --locked --release --features native-skia --bin rhwp`
  - fresh WASM: `scripts/wasm-pack-locked.ps1 --target web --out-dir pkg`, wasm-pack 0.15.0

| 산출물 | head SHA-256 | base SHA-256 |
| --- | --- | --- |
| `rhwp.exe` | `7a02036c4e5b3a9d8324a3eb679e50f3e7c332c513e2ea33cf5b544fc6f92cc1` | `827089f26e6b58974750f76c4cfee0b2cef6a2cc419fb205d84ee4f48868f70a` |
| `rhwp_bg.wasm` | `f6e0be02dd7c8326520c5d95043bc0f0e7c7e863eb8dd670ced0f191bc94aaa9` | `cd08982023472cf72bccf946e835a1c1ab4407dbd97c97701b4964c3d74ce50d` |

- Chrome 156.0.8078.74, Node 24.22.1, Python 3.12.10, pypdfium2 5.15.2, Pillow 13.0.0, fontTools 4.68.0, Poppler 25.07.0.
- `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96 --page-workers 4`.
  `--font-path`는 Windows 글꼴 175개(읽을 수 없는 `mstmc.ttf` 제외)와 한컴오피스 2024 `Shared/TTF`다.
  WASM은 같은 입력, PDF, 글꼴에 `--wasm-pkg`만 더했다.

## 결과

`silhouette_raw_match_percent` 기준이다. 4개 문서 모두 쪽수는 rhwp와 PDF가 같다.

| key | 쪽수 | head Native | head WASM | base Native | base WASM |
| --- | --- | --- | --- | --- | --- |
| fill-patterns-0-6-hwp | 1 | 99.97 | 99.99 | 96.68 | 88.20 |
| fill-patterns-0-6-hwpx | 1 | 99.97 | 99.99 | 88.98 | 99.99 |
| hwpspec | 178 | 97.08 | 97.08 | 97.08 | 97.08 |
| 59043-regulatory-analysis | 37 | 37.53 | 36.74 | 37.53 | 36.74 |

값은 문서별 최저값이다. base와 head를 쪽마다 비교하면 두 경로 모두 떨어진 쪽이 없다. 값이 바뀐 쪽은 아래뿐이다.

| key | 쪽 | 경로 | base | head |
| --- | --- | --- | --- | --- |
| fill-patterns-0-6-hwp | 1 | Native | 96.68 | 99.97 |
| fill-patterns-0-6-hwp | 1 | WASM | 88.20 | 99.99 |
| fill-patterns-0-6-hwpx | 1 | Native | 88.98 | 99.97 |
| hwpspec | 23 | WASM | 99.80 | 99.99 |
| 59043-regulatory-analysis | 9 | WASM | 99.71 | 99.93 |

원본 TSV: [native](native/), [wasm](wasm/), [base-native](base-native/), [base-wasm](base-wasm/).

## 59043의 90% 미만 쪽

59043은 Native와 WASM 모두 12, 14, 15, 16, 20, 25, 28, 37쪽이 90% 미만이다. 이 8쪽은 base에서도
두 경로 모두 같은 값이라 이번 변경과는 관계가 없다. 무늬가 있는 쪽은 9쪽이고, 9쪽은 Native 99.87,
WASM 99.93이다.

이 문서의 미달은 이미 #7778로 넘어가 있다([#7445 이관 기록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5970120438)).
미달 쪽을 잡아내던 `regulatory_59043_page36_keeps_nested_source_and_following_headings`는 그때 빠졌다.
쪽수 검사와 8·11·12쪽 그림 소속 검사 4개는 `tests/issue_1921_59043_pagination_pin.rs`에 남아 있고,
이번 head의 nextest에서도 통과한다. 이 PR은 59043의 원본, PDF, 검사를 바꾸지 않는다.
그래서 새로 넘길 검사도 없고, 이 문서는 무늬 쪽의 대조 자료로만 쓴다.

## 직접 판독

실루엣 점수로는 칸 안 빗금의 방향을 구분할 수 없어서 같은 영역을 확대해 비교했다.

- 시험 문서: Native와 WASM 모두 위 행부터 ─ │ ＼ ／ ＋ × 순서이고, 7행(무늬 6)에는 무늬가 없다.
  무늬 간격도 PDF와 같다.
  [Native review](png/fill-patterns-0-6-hwp-native-review.png) ·
  [Native overlay](png/fill-patterns-0-6-hwp-native-overlay.png) ·
  [WASM review](png/fill-patterns-0-6-hwp-wasm-review.png) ·
  [WASM overlay](png/fill-patterns-0-6-hwp-wasm-overlay.png)
- hwpspec 23쪽: 한컴은 「RECORD_HEADER」(무늬 3)를 ／로, 「RECORD_DATA (RD)」(무늬 2)를 ＼로 그린다.
  base Native는 둘을 반대로 그렸다. head Native와 WASM은 PDF와 같다.
  [확대 비교(PDF / base Native / head Native / head WASM)](png/hwpspec-p23-zoom-pdf-base-head-wasm.png) ·
  [Native review](png/hwpspec-p23-native-review.png) ·
  [WASM review](png/hwpspec-p23-wasm-review.png)

## 편람 gate

`python scripts/visual_sweep.py --handbook-gate --rhwp-bin <head rhwp>`, 지원 웹폰트 사용.

- Ubuntu 24.04(WSL2)에서 head `7de77727c`를 빌드했다
  (`cargo build --locked --release --features native-skia --bin rhwp`, SHA-256
  `785baffbba3567362ea294f85db5e70a44f332c61b5d8e9d47ae99c8fb8d1395`).
- 결과는 `passed`다. 383/383쪽, 최저 99.96%(130쪽, 알려진 WMF 차이)이고 나머지 382쪽은 100.00%다.
  기준 대비 떨어진 쪽은 없다.
- 원본 TSV: [native/handbook-hwpx_silhouette.tsv](native/handbook-hwpx_silhouette.tsv)
  (SHA-256 `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308`, 이전 head 결과와 같다).

Windows `rhwp.exe`로 같은 gate를 돌리면 9쪽만 100.00에서 99.99로 나온다(제목 글자 안티앨리어싱).
이전 head `b14268b8f`와 base `eb84afc03`의 Windows 출력은 383쪽 PNG가 SHA-256까지 같았다.
그래서 이 차이는 이번 변경이 아니라 Windows 글꼴 래스터 차이로 본다.
