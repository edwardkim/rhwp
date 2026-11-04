# 표시 치환 뒤 원본 결측 기호의 전진 보존

원본은 [영월 임대차 서식](../../../../../samples/issue5699/20099369_yeongwol_forms.hwp), 독립 기준은 [기존 한컴 Print](../../../../../pdf/issue7642-failed-inputs-followup-20261011/20099369_yeongwol_forms-2020.pdf)다.
새 PDF를 만들거나 입력을 바꾸지 않았다. 두 파일은 기존 저장소 경로를 재사용한다.

| 자료 | SHA-256 |
| --- | --- |
| HWP | `6d723789148699479983fbcb6fa6368a252c1187fe0a7b3c922b583f6727ec09` |
| Print PDF | `a04da624238d77da4832dd302017c7d2f559a01da02e4c1b4ae12cfd3f77b123` |
| Native 실행 파일 | `71a6e065770d741971ad618e614a2d0bc1759cf2eefd4ce9b72ad395032f8d8f` |
| 최적화 fresh WASM | `0fcc2a5d219ade9de64ce9657de95ce8769ce21d3bed4062835dc15e205ae3e1` |

제품 source `4b149b6cdfc514b9858ed644be2cf8120c731cd4`, WASM 실제 빌드 head
`7bb9e66fd006109ca9f7ee4c4d3f752c82e60fae`다. 두 head의 src·crates·Cargo·assets tree가 같다.
2026-11-04, 96 DPI, Print 2022, PDFium, 색상 경계 보정 전 2px raw를 측정했다.
전체 시스템 글꼴과 [승인된 기존 대체 설정](../yeongwol-font-environment.json)을 양쪽에 공급했다.
회귀 실행은 저장소 허용 글꼴 5개만 노출하는 최소 환경과 구분한다. 글꼴 파일은 추가하지 않는다.

| 쪽 | Native raw | fresh WASM raw |
| ---: | ---: | ---: |
| 1 | 99.98384% | 99.98228% |
| 2 | 99.99067% | 100.00000% |
| 3 | 99.96664% | 99.97763% |
| 4 | 99.99505% | 100.00000% |

전쪽 TSV 원문: [Native](yeongwol-native.tsv) · [fresh WASM](yeongwol-wasm.tsv).
원본·두 경로·독립 PDF 모두 4쪽, 누락·90% 미달 0쪽이다. 원 PR #8389·#8397의
표시 기준 99.60 / 99.99 / 99.97 / 100.00%를 모두 충족한다.

| 같은 입력의 쪽 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| 1: `민․형사상의 모든` | [review](yeongwol-p001-native-review.png) / [overlay](yeongwol-p001-native-overlay.png) | [review](yeongwol-p001-wasm-review.png) / [overlay](yeongwol-p001-wasm-overlay.png) |
| 3: 본문·표시 face 보존 | [review](yeongwol-p003-native-review.png) / [overlay](yeongwol-p003-native-overlay.png) | [review](yeongwol-p003-wasm-review.png) / [overlay](yeongwol-p003-wasm-overlay.png) |

1쪽의 dot·뒤 본문·줄바꿈·표 프레임을 직접 대조했다. 공급 폭을 후보 PNG에서 역산하지 않고
HBATANG.TTF의 독립 hmtx 178/1024em와 원본 글자 크기를 사용한다. 프로그램 SHA-256은
`35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05`다.
해당 프로그램의 글꼴 바이트를 회귀나 저장소에 넣지 않는다.

[새 회귀](../../../../../tests/cases/issue_8419_display_substitution_supplier_advance.rs)는 표시 face를
공개 Noto로 명시하면서 원본 메트릭을 유지한다. 로컬 private/OTF 설치를 요구하지 않는다.
원본 source의 dot 존재·모양 소속·공급자·1HU 양자화 이내 전진·실제 run 폭과 뒤 본문의
연속 관계를 검사한다. `바탕→Haansoft Batang`만 지정하면 수정 전에도 통과하므로 그 설정을
결함 검출 반례로 쓰지 않는다. 표시용 Noto 치환에서 보정 전 실제 WASM API는 1200HU
`heuristicFullwidth`, 보정 후 실제 fresh WASM API는 208HU `metricGlyphFallback`다.

검증 head `e8b36a41aa`에서 새 Rust 회귀를 저장소 허용 글꼴 5개만 노출한 최소 환경으로
실행해 1/1 PASS, exit 0을 확인했다. 컴파일은 공용 `target/pr-review`·빌드 잡 8개다.
공개 API의 보정 전 FAIL / 보정 후 PASS와 Rust의 보정 후 PASS를 구분한다. Rust의 보정 전
실행으로 바꿔 쓰지 않는다. [공통 보정 분석](../../../../working/task_m100_yeongwol_source_punctuation_advance.md)에 실행 결과를 연결한다.
