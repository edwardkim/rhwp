# #8927 Visual Sweep 실루엣 TSV 원본

`scripts/visual_sweep.py --silhouette-only` 결과를 바이트 그대로 복사했다. 96 DPI, Print, 2px raw
실루엣(`threshold_boundary_color_support_v1`, 색상 허용 32), `--compat 2022`.

| 폴더 | 산출 |
|---|---|
| `head/` | source `82d7439a98`(Native Skia `release-test`, fresh WASM `wasm-pack-locked.sh`) |
| `base/` | devel `a165c6c7d2`(같은 방식) |

| 키 | 입력 | 기준 PDF |
|---|---|---|
| `a4-inserted-HYhwpEQ` | `samples/issue8927/a4-inserted-HYhwpEQ.hwp` | `pdf/issue-8927/a4-inserted-HYhwpEQ-2022.pdf` |
| `a4-inserted-HancomEQN` | `samples/issue8927/a4-inserted-HancomEQN.hwp` | `pdf/issue-8927/a4-inserted-HancomEQN-2022.pdf` |
| `sep2022` | `samples/3-09월_교육_통합_2022.hwp` | `pdf/3-09월_교육_통합_2022-hwp-2020.pdf` |
| `sep2023` | `samples/3-09월_교육_통합_2023.hwp` | `pdf/3-09월_교육_통합_2023-hwp-2020.pdf` |
| `oct2022` | `samples/3-10월_교육_통합_2022.hwp` | `pdf/3-10월_교육_통합_2022-hwp-2020.pdf` |
| `nov2022` | `samples/3-11월_실전_통합_2022.hwp` | `pdf/3-11월_실전_통합_2022-hwp-2020.pdf` |
| `exam_math` | `samples/exam_math.hwp` | `pdf/exam_math-hwp-2020.pdf` |
| `exam_science` | `samples/exam_science.hwp` | `pdf/exam_science-hwp-2020.pdf` |
| `issue505` | `samples/issue-505-equations.hwp` | `pdf/issue-505-equations-hwp-2020.pdf` |

글꼴 공급은 Native·fresh WASM 같다: 본문 글꼴은 Native가 준비한 웹폰트 환경(`--font-environment`와 그
프로그램 경로)을 WASM에도 넘겼고, 수식 원 글꼴 `HyhwpEQ`·`HancomEQN`은 한글 2022 설치본의
TTF를 로컬 `--font-path`로 공급했다(저장소에 넣지 않는다). `exam_math`·`exam_science`·`issue505`의
90% 미만 쪽은 head와 devel이 같은 쪽이고 값 차이는 0.01%p 이하다.
