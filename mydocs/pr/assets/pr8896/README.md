# PR #8896 메인테이너 검증 증거

원 기여 head `51a960e392a3d79a4d28f78fa90a2aa88ad8c0d7`, 현재 devel `4c876654d86d071d342fc78706c51cfff6ddbde4`의 검토 병합 제품 소스로 검증했다.
실제 Native/최적화 WASM producer source는 `a22e3fbb8ff117086443c0618287c34f9c7dc5ef`다.

2026-11-11 PDFium 96 DPI·print·compat 2022·색상 경계 보정 전 2px raw 실루엣으로
공개 입력 3개 전 6쪽을 각각 비교했다. Native/fresh WASM 모두 전쪽 raw 100.00000%이며 쪽수가 일치한다.
`focus-summary.json`과 `focus/<backend>/<key>/`의 TSV·manifest·동일 쪽 PNG를 따른다.
각 폴더의 `silhouette_manifest.json`은 같은 실행의 PNG 쌍을 정본 CLI로 측정한 결과이며, `run_manifest.json`은 실제 렌더 생산자·원본·PDF·글꼴을 기록한다.

Native는 Skia 직접 PNG이며 WASM은 locked wrapper·wasm-opt 최적화 후 Chromium 152/revision1660786의 직접 Canvas PNG다.
합성 입력의 웹한글기안기 Print 기준과 비공개 실문서 보고를 구분한다. 이 수치를 RGB 전체 픽셀 동일로 해석하지 않는다.

정본 보호·새 devel 기능 보존·Rust 검사는 진행 중이며, 이 문서는 아직 최종 수용 판정이 아니다.
