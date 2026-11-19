# P136 한컴 인쇄본 대조

공개 가족관계등록 예시 문서의 두 쪽을 96 DPI, 한컴 2022 호환, Print 프로필로 대조했습니다. 기준 PDF의 생성자는 `Hwp 2022 12.0.0.4547`, 인쇄기는 `Hancom PDF 1.3.0.547`입니다. 원문과 기준 PDF는 기존 #8554 자산을 재사용했고, 실제 PDFium 출력과 각 렌더 결과를 비교했습니다.

- [공개 원본 HWP](../../../../../../samples/issue8554/2913671-appendix1-family-register.hwp)
- [한컴 Print PDF](../../../../../../pdf/issue8554/2913671-appendix1-family-register-2022.pdf)
- [실행 입력과 산출물 해시](evidence.json)

| 쪽 | 수정 전 Native | 수정 후 Native | fresh WASM Canvas |
| --- | ---: | ---: | ---: |
| 1 | 95.49% | 98.15% | 97.99% |
| 2 | 100.00% | 100.00% | 100.00% |

표는 2px 관용 raw 실루엣 점수입니다. 출력 쪽수는 모두 기준의 두 쪽과 같았습니다. Native 전후의 두 렌더 트리도 바이트 단위로 일치했습니다. 첫 쪽의 두 이름에 빠졌던 한자 여섯 자가 나타난 것과 표·문단 경계가 유지된 것을 비교 이미지에서 확인했습니다. 같은 영역에 글꼴 모양의 차이는 남아 있습니다.

Native는 복원이 재현된 Serif-only Fontconfig와 검증된 오프라인 웹폰트 캐시에서 CLI의 실제 Print PNG 출력을 사용했습니다. WASM은 `739f9589c74d4b65013b5284c50676c7c913c17c`의 최적화된 새 web package를 Canvas2D에 로드하고 같은 공개 Noto Serif CJK 글꼴을 공급했습니다. 이번 변경의 세 Rust 소스는 Native Skia에 속하며 WASM 빌드에서 제외됩니다.

[수정 전 Native TSV](native-base.tsv) · [수정 후 Native TSV](native-fixed.tsv) · [fresh WASM TSV](wasm.tsv)

[Native 1쪽 검토](native-p001-review.png) · [Native 1쪽 겹침](native-p001-overlay.png) · [WASM 1쪽 검토](wasm-p001-review.png) · [WASM 1쪽 겹침](wasm-p001-overlay.png)
