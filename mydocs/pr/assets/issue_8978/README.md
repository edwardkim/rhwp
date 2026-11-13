# #8978 채우기 무늬 종류 시각 검증

HWP 무늬 종류를 0~5로 해석하는 수정의 Native/fresh WASM Visual Sweep 기록이다.

## 입력과 기준 PDF

| key | 원본 | 기준 PDF |
| --- | --- | --- |
| fill-patterns-0-6-hwp | `samples/issue8978/fill-patterns-0-6.hwp` | `pdf/issue8978/fill-patterns-0-6-2024.pdf` |
| fill-patterns-0-6-hwpx | `samples/issue8978/fill-patterns-0-6.hancom2024.hwpx` | `pdf/issue8978/fill-patterns-0-6.hancom2024-hwpx-2024.pdf` |
| hwpspec | `samples/hwpspec.hwp` | `pdf/hwpspec-2024.pdf` |
| 59043-regulatory-analysis | `samples/issue1921/59043_regulatory_analysis.hwp` | `pdf/issue1921/59043_regulatory_analysis-2022.pdf` |

- `fill-patterns-0-6.hwp`는 7행 표의 오른쪽 칸 BorderFill에 무늬 종류 0~6을 그대로 저장한 합성 입력이다.
  `setCellProperties({fillType:'solid', fillColor:'#ffffff', patternType:N, patternColor:'#000000'})`로 만들었다.
- `fill-patterns-0-6.hancom2024.hwpx`는 한글 2024(13.0.0.1408)가 위 HWP를 HWPX로 저장한 파일이다.
- 두 PDF는 한글 2024로 Microsoft Print to PDF, 전체 문서, 1-up 출력했다.
- hwpspec와 59043은 무늬 2·3을 쓰는 기존 한컴 저장 문서와 기존 기준 PDF다.

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

- base: devel `eb84afc03`. 후보 코드: `src` tree `ec10b16b72cdbc0f02511b20c7d90efef985f004`,
  `rhwp-studio/src` tree `4bc52f011cf252d26085fd8d91ed616a97e20f82`.
- Native: Windows 11, Rust 1.93.1, `cargo build --locked --release --features native-skia --bin rhwp`.
  후보 `rhwp.exe` SHA-256 `21bdc956b61a7927c1fbe11ff43ebe4bb008553788bf87b907ef8ae996dfa065`,
  base `3fcddbcd76eb678f1db52498bce66220a0071e11746e9ad4ea1994ab85510535`.
- fresh WASM: WSL2 Ubuntu, Rust 1.93.1, wasm-pack 0.15.0, `scripts/wasm-pack-locked.sh --target web --out-dir pkg`.
  `rhwp_bg.wasm` SHA-256 `5f8334a21ffda826c981e3c7c7379ec0b6d460a045e844209268b0f540908b37`.
  Chrome 156.0.8078.74, Node 24.22.1.
- Python 3.12.10, pypdfium2 5.15.2, Pillow 13.0.0, fontTools 4.68.0, Poppler 25.07.0.
- `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96 --page-workers 4`,
  `--font-path`는 Windows 글꼴 175개(읽을 수 없는 `mstmc.ttf` 제외)와 한컴오피스 2024 `Shared/TTF`.
  WASM은 같은 입력·PDF·글꼴에 `--wasm-pkg pkg`를 더했다.

## 결과 (`silhouette_raw_match_percent`)

| key | 쪽수(rhwp/PDF) | Native 최저 | fresh WASM 최저 | base Native 최저 | 90% 미만 |
| --- | --- | --- | --- | --- | --- |
| fill-patterns-0-6-hwp | 1/1 | 99.97 | 99.99 | 96.68 | 없음 |
| fill-patterns-0-6-hwpx | 1/1 | 99.97 | 99.99 | 88.98 | 없음 |
| hwpspec | 178/178 | 97.08 | 97.08 | 97.08 | 없음 |
| 59043-regulatory-analysis | 37/37 | 37.52 | 36.74 | 37.52 | 9쪽 |

- hwpspec 23쪽(인쇄 13쪽): Native 99.98762%, fresh WASM 99.99085%, base Native 99.98778%.
- 59043은 Native 37쪽 모두 base와 같다. 90% 미만 9쪽(3·12·14·15·16·20·25·28·37쪽)은 base에도 같은 값으로
  있으며 무늬 쪽(9쪽, Native 99.87% / WASM 99.93%)이 아니다. 문서 전체 피델리티는 #7778에서 추적한다.
- 원본 TSV: [native](native/), [wasm](wasm/), [base-native](base-native/).

## 직접 판독

실루엣 점수는 칸 안 빗금의 방향을 구분하지 못하므로 같은 영역을 확대해 대조했다.

- 시험 문서: Native·WASM 모두 행마다 ─ │ ＼ ／ ＋ × 순서이고 7행(무늬 6)은 무늬가 없다. 무늬 간격도 PDF와 같다.
  [Native review](png/fill-patterns-0-6-hwp-native-review.png) ·
  [Native overlay](png/fill-patterns-0-6-hwp-native-overlay.png) ·
  [WASM review](png/fill-patterns-0-6-hwp-wasm-review.png) ·
  [WASM overlay](png/fill-patterns-0-6-hwp-wasm-overlay.png)
- hwpspec 23쪽: 한컴은 「RECORD_HEADER」(무늬 3)를 ／, 「RECORD_DATA (RD)」(무늬 2)를 ＼로 그린다. base Native는 둘을
  반대로, base SVG는 무늬 2를 세로줄로 그렸다. 후보 Native·WASM은 PDF와 같다.
  [확대 비교(PDF / base Native / 후보 Native / 후보 WASM)](png/hwpspec-p23-zoom-pdf-base-head-wasm.png) ·
  [Native review](png/hwpspec-p23-native-review.png) ·
  [WASM review](png/hwpspec-p23-wasm-review.png)

## 편람 gate

`python scripts/visual_sweep.py --handbook-gate --rhwp-bin <후보 rhwp.exe>` (Windows 11, 지원 웹폰트 공급).

- 383/383쪽, 최저 99.96%(130쪽, 알려진 WMF 차이), 100.00%는 365쪽. 90% 미만 없음.
- `pr_review_gate`: `re_review_required`. 기준 대비 하락은 9쪽 하나(100.00 → 99.99)다.
- 같은 명령의 export 인자로 base `rhwp.exe`도 383쪽을 출력했다. 383쪽 모두 후보와 PNG SHA-256이 같다.
  9쪽 차이는 제목 글자의 안티앨리어싱이며 이 변경과 무관하다. 편람에는 `hatchStyle`이 없다.
- 원본 TSV: [native/handbook-hwpx_silhouette.tsv](native/handbook-hwpx_silhouette.tsv)
  (SHA-256 `9410ba06506c6fc30c084c6e5233efa4611025f9a9d9a3312fcc6ae92ba81d22`).
