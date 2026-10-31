# 합성 팔레트 기존 회귀의 독립 Print 대조

2026-10-29, 렌더링 소스 `ffae46e8a6`을 분리 worktree에서 빌드했다. 기존 회귀 실패의 전제 확인에 사용한 합성 대조 입력이며 행정업무운영 편람의 시각 점수를 대신하지 않는다. 2024 Print PDF와 같은 입력의 Native 및 fresh WASM 출력 전체를 `scripts/visual_sweep.py`로 96dpi PDFium 래스터화하여 2px raw 실루엣을 비교했다. 모든 입력은 독립 PDF와 출력이 각 1쪽이다.

| 입력 | Native 1쪽·전체 최저 | fresh WASM 1쪽·전체 최저 | 원본 TSV | overlay |
| --- | ---: | ---: | --- | --- |
| 흑백 예약색 | 100.00000% | 100.00000% | [Native](reserved-native-silhouette.tsv) / [WASM](reserved-wasm-silhouette.tsv) | [Native](reserved-native-overlay_001.png) / [WASM](reserved-wasm-overlay_001.png) |
| 색상 팔레트 | 100.00000% | 100.00000% | [Native](indexed-native-silhouette.tsv) / [WASM](indexed-wasm-silhouette.tsv) | [Native](indexed-native-overlay_001.png) / [WASM](indexed-wasm-overlay_001.png) |

TSV는 표시 정책에 따라 소수 둘째 자리까지 기록한다. 각 overlay를 직접 대조했고 위치 차이는 발견하지 못했다. 2024 Print에서 두 입력의 그림 18개를 기존 독립 Print 추출 JPEG와 동일 decoder로 복호화해 RGB 전 픽셀이 일치함을 확인했다. 색 확장 여부와 Print 면적 표본 경로는 다른 계약이므로 기존 테스트의 둘을 동일시한 전제를 수정한다. 원본 바이트·색과 2022/화면 프로필, 소유 및 출력 payload 대조 검사는 유지한다.

입력과 기준 PDF의 SHA-256:

- `pdf/issue7642-metafile-palette-visibility-20261017/bitmap-white-palette-controls.hwpx`: `0460afdbe6ed8f9a4272cac1828645760038421a6aa03338c96d6e730a8cab55`
- `pdf/issue7642-metafile-palette-visibility-20261017/bitmap-white-palette-controls-2024.pdf`: `e9b268117253193197fd8c2de694165bf681fb0f8ea28ad5659a9117b19fa6cf`
- `pdf/issue7642-metafile-palette-visibility-20261017/bitmap-indexed-color-controls.hwpx`: `b6e326a5a7b486a85cb49c599611ec80f39e516b3ac2c6a93efaf6241542e04b`
- `pdf/issue7642-metafile-palette-visibility-20261017/bitmap-indexed-color-controls-2024.pdf`: `0ed6c7157fc01b3d33213793561a217353d89e8c24635902ffd4d712062fb418`

Native 실행 파일 SHA-256 `9425abb8a2d6fb981992573aa618eced426f0ca745cf5d01cb3cc0c5e2970563`, fresh WASM SHA-256 `6894bcc9978476aa432b71a55ea149e821efcd27041a5ae137fe406b98ac888c`. 시스템 글꼴은 내부 실행에만 사용했으며 새 글꼴은 이 증거에 포함하지 않았다.
