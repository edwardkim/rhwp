# KAF 재인쇄 대조와 메인터너 보정

코드 `a0bca426057bb80dca3f7fcea0baf4eeee76edd2`에서 원본 표 44쪽과 부록 54쪽을 확인했다.
같은 원본과 96 DPI, compat 2022, Visual Sweep의 raw 실루엣 2px 이웃 기준을 사용했다.
[최초 인쇄본과 신규 MCP 인쇄본의 영수증](../mcp-print-reissue.json)에 두 출처를 기록했다.

- 24쪽의 독립 인라인 글상자는 폭을 줄이고 높이를 늘린 원본 변환을 가진다. 현재 내부 폭으로 글줄 경계를 다시 계산하고 기존 세로 행 위치를 유지한다. 새 MCP 2020 Print 기준 Native는 두 문서에서 97.05→100.00%다.
- 부록 36쪽은 늘어난 외부 글상자에서 `such as`와 `and Cadastre`의 줄 소속이 PDF와 같아졌다. 새 MCP 기준 Native 94.65→100.00%다.
- 부록 38쪽은 원점에서 논리 창 끝 픽셀을 포함하는 단일 DIB의 인쇄 틀을 복원했다. 새 MCP 기준 Native 96.01→100.00%다. 39쪽의 그림 폭은 새 한컴 2020·2024 모두 410.19pt이며 기존 한컴 2020 Print는 408.482pt다. 후보 Native는 두 새 인쇄본에서 100.00%, 기존 인쇄본에서 98.23%다. 기존 인쇄본의 보정 전 39쪽은 100.00%였다.

## 31쪽 기준 PDF 확인

표·부록의 새 한컴 2020 `11.0.0.9172` Print는 `/HCI#20Poppy` Type0의 하위 CID 글꼴에
`/Subtype /Type1`을 기재하고 `INPILL+Gulim` 글꼴 기술자를 연결한다. Trace에서 표는
객체 31→268→264, 부록은 31→347→343이며 MuPDF가 두 파일에서 `unknown cid font type`을 보고한다.
한컴 2024 `13.0.0.4225`로 MCP에서 다시 인쇄한 표·부록 31쪽에서도 같은 객체 참조 오류가
재현됐고, Native는 두 문서 모두 95.69%였다. [한컴 2024 객체 점검 및 점수](mcp-2024-native-comparison.json)
신규 2020 기준 Native의 31쪽은 95.73%이고, 같은 후보 PNG를 기존 한컴 `11.0.0.1623`의 독립 Print에
대조하면 두 문서 모두 98.84%다. 31쪽의 비교 기록과 기존 기준 PDF는 보존한다.

## Native 전쪽 증적

| 문서 | 새 MCP Print TSV | 최초 독립 Print TSV | 31쪽 비교 |
| --- | --- | --- | --- |
| 표 | [44쪽](kaf-table/native/mcp-2020.tsv) | [44쪽](kaf-table/native/original-print.tsv) | [MCP review](kaf-table/native/review_031.png) · [overlay](kaf-table/native/overlay_031.png) · [독립 Print review](kaf-table/native/original-print-review_031.png) · [overlay](kaf-table/native/original-print-overlay_031.png) |
| 부록 | [54쪽](kaf-appendix/native/mcp-2020.tsv) | [54쪽](kaf-appendix/native/original-print.tsv) | [MCP review](kaf-appendix/native/review_031.png) · [overlay](kaf-appendix/native/overlay_031.png) · [독립 Print review](kaf-appendix/native/original-print-review_031.png) · [overlay](kaf-appendix/native/original-print-overlay_031.png) |

- 부록 24쪽 [review](kaf-appendix/native/review_024.png) · [overlay](kaf-appendix/native/overlay_024.png)
- 부록 36쪽 [review](kaf-appendix/native/review_036.png) · [overlay](kaf-appendix/native/overlay_036.png)
- 부록 38쪽 [review](kaf-appendix/native/review_038.png) · [overlay](kaf-appendix/native/overlay_038.png)

[Native 전쪽 집계와 입력 해시](native-comparison.json)

## fresh WASM 전쪽 증적

보정 전 비교에서 사용한 1,129 face의 카탈로그를 실제 글꼴 SHA-256과 대조해 순서와 공급
바이트를 고정했다. 같은 macOS의 locked wrapper `--no-opt` 패키지로 재출력했으며 표 44쪽과
부록 54쪽의 PNG·렌더 트리·출력 완료 기록을 검사했다.

| 문서 | 새 한컴 2020 Print | 새 한컴 2024 Print | 기존 독립 한컴 2020 Print |
| --- | --- | --- | --- |
| 표 | [44쪽 TSV](kaf-table/wasm/mcp-2020.tsv) | [44쪽 TSV](kaf-table/wasm/mcp-2024.tsv) | 최저 98.38% · [44쪽 TSV](kaf-table/wasm/original-print.tsv) |
| 부록 | [54쪽 TSV](kaf-appendix/wasm/mcp-2020.tsv) | [54쪽 TSV](kaf-appendix/wasm/mcp-2024.tsv) | 최저 98.22% · [54쪽 TSV](kaf-appendix/wasm/original-print.tsv) |

새 한컴 2020 Print의 원 PR 대비 하락은 Native·fresh WASM 각 0쪽이다. 두 새 인쇄본에서
98% 미달은 양 문서 모두 CID 참조가 어긋난 31쪽이다. 기존 독립 인쇄본 기준 Native 최저는
표 98.33%, 부록 98.23%이며 31쪽은 Native·fresh WASM 모두 98.84%다.

- 부록 24쪽 97.10→100.00% [review](kaf-appendix/wasm/review_024.png) · [overlay](kaf-appendix/wasm/overlay_024.png)
- 부록 36쪽 94.39→100.00% [review](kaf-appendix/wasm/review_036.png) · [overlay](kaf-appendix/wasm/overlay_036.png)
- 부록 38쪽 96.01→99.93% [review](kaf-appendix/wasm/review_038.png) · [overlay](kaf-appendix/wasm/overlay_038.png)
- 부록 39쪽 새 2020 Print 99.97% 유지, 기존 Print 99.98→98.22% [review](kaf-appendix/wasm/review_039.png) · [overlay](kaf-appendix/wasm/overlay_039.png)
- 부록 31쪽 새 2020 Print 95.75% [review](kaf-appendix/wasm/review_031.png) · [overlay](kaf-appendix/wasm/overlay_031.png), 기존 Print 98.84% [review](kaf-appendix/wasm/original-print-review_031.png) · [overlay](kaf-appendix/wasm/original-print-overlay_031.png)

[WASM 전쪽 집계](wasm-comparison.json) · [패키지·글꼴 카탈로그·쪽별 출력 해시](wasm-capture-identity.json)

## Studio 실제 출력 경로

macOS Chrome에서 24·36·38·39쪽을 screen과 print로 열어 실제 backend를 확인했다.
검증 호스트가 Arial과 한양신명조의 글꼴 바이트를 공급한 실행에서 Canvas2D와 CanvasKit
software compat/default가 각각 8회 선택됐다. 초기화 요청 24회의 실제 WASM 해시가 동일하다.
소비 경로와 글꼴 해시는 [실행 기록](studio-consumer.json)에 남겼다.

| 쪽 | Studio Canvas2D Print | Studio CanvasKit software Print |
| --- | --- | --- |
| 24 | [줄 경계](studio-print/p024-canvas2d.png) | [줄 경계](studio-print/p024-canvaskit-default.png) |
| 36 | [글상자 끝](studio-print/p036-canvas2d.png) | [글상자 끝](studio-print/p036-canvaskit-default.png) |
| 38 | [DIB 인쇄 틀](studio-print/p038-canvas2d.png) | [DIB 인쇄 틀](studio-print/p038-canvaskit-default.png) |

CanvasKit은 36쪽의 screen·print와 38·39쪽의 screen에서 지원되지 않는 이미지
명령과 재생 실패를 보고했다. 같은 글꼴을 공급한 보정 전 패키지와 비교하니 실패한
쪽·프로필·모드·오류 종류가 모두 같았다. 양쪽 실행의 실패 기록을 보존했다.

## 최종 후보 재검증

변경된 Mac 글꼴 목록을 사용한 Native 재실행에서 부록 41·42·43·45·46쪽이
이전 환경의 원 PR 측정값보다 낮았다. 이번 후보 실행에 기록된 1,106개 파일의 순서와
SHA-256을 고정하고 원 PR `d53cf0751b`를 그 조건으로 다시 실행했다. 동일한 한컴 2020
재인쇄 PDF로 비교한 54쪽에서 5쪽이 상승했고 하락은 0쪽이었다.
[원 PR Native TSV](final-verification/appendix-original-pr-native.tsv) ·
[최종 후보 Native TSV](final-verification/appendix-final-native.tsv) ·
[글꼴 및 실행 파일 대조](final-verification/native-original-pr-same-fonts.json)

최종 Native Skia와 macOS의 `wasm-opt` 포함 fresh WASM으로 표 44쪽과 부록 54쪽을 다시
검증했다. 최초 인쇄본의 Creator 문자열에는 2018이 표시되지만 빌드 `11.0.0.1623`은
실측 및 MCP 영수증에서 한컴 2020으로 확인됐다. 최초 인쇄본에서 두 문서 모두 전쪽
98.00% 이상이다. 신규 인쇄본의 31쪽에는 앞서 확인한 CID 오류가 계속 있으며, 유효한
최초 인쇄본에서 31쪽의 98.84%를 다시 확인했다. 같은 신규 한컴 2020 인쇄본을 원 PR과
후보에 적용한 페이지별 비교는 두 경로 모두 하락 0쪽이다.
[두 문서의 전쪽 비교](final-verification/full-comparisons.json)

Studio가 초기화한 WASM 24회의 바이트 해시는 최적화 패키지와 같았다. 후보의 대표 인쇄
이미지 6장은 기존 증적과 동일하며 CanvasKit 재생 진단도 같은 조건의 앞선 실행과 일치했다.
[패키지·브라우저·인쇄 이미지 신원](final-verification/optimized-wasm-and-studio.json)
