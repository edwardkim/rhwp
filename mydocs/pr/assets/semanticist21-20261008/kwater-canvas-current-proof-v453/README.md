# 새 패키지 actual Studio Canvas2D 검증

생산 `ddf17f864`의 루트 pkg/Studio public 파일과 실제 브라우저 응답 해시를 확인했다.
WASM `4299eb204ec8283d75446b61f2183d3c4f3cd41acc0acabfd7be7dc065f01ae0`.
Native K-water24 SVG의 검증된 실제 폰트 바이트를 공급하고 Canvas metric 경로를 활성화했다.
Chrome140, renderer canvas2d/fallback없음, 문서27쪽, pageError0, 실행exit0·21.817초, 임시 서버 종료.
현재24/25쪽을 독립 Print와 직접 열었다. 2px 실루엣99.94940/99.66933%, 표 외곽·내부행·끝글자·다음서식 보존.
글자획·작은 불릿크기·미세선 래스터 차이는 남는다.

원본 `samples/hwpx/k-water-rfp.hwpx`, 기준 `pdf/hwpx/k-water-rfp-hwpx-2020.pdf`다.
#7773에 등록된 `samples/k-water-rfp-2024.hwp`의 동일 원본 검증으로 대신하지 않는다.
ignored v453/current-kwater-canvas-result.json, studio-kwater-canvas/diagnostics.json과
independent-print-comparison.json에 명령/실제 응답·기능 경로·PNG/기준 SHA·직접 판독을 보존했다.

- `canvas-p024-review.png` — `ad62929ecf59831a1302b32c82aafb598e7261639bacd569055d31aa0ab0be72`
- `canvas-p024-overlay.png` — `3e0668ad4e04980a64375f30aa63066ec904367b61a7e42141b25ff09d8c65ae`
- `canvas-p025-review.png` — `407e352f4cf4c0467c571bed837dc2043249a27a8f7591a56c6a87b0ac8b1e9a`
- `canvas-p025-overlay.png` — `3e586f79667c56dec27b61a52c16fac705cac8943c412cbb1c05c1febf2e70a0`
