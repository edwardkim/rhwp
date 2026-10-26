# 편람 139쪽 그림의 독립 Print 기준

- 입력: `samples/2025 행정업무운영 편람(최종).hwpx`.
- 입력 SHA-256: `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`.
- 독립 한컴 2024 Print PDF SHA-256:
  `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e`.
- 기존 PDF 139쪽을 `scripts.visual_sweep_pdf.rasterize`의 정확한 96dpi
  `pdfium_smooth_rgb_exact_dpi_v2` 행렬로 출력했다. PDF를 새로 변환하지 않았다.
- 이 PNG는 위 기준의 `[139,658,586,853)` 영역을 RGB로 자른 것이다.
  bitmap에 포함된 글자도 원본 그림이므로 시스템 글꼴을 공급하지 않는다.
- PNG SHA-256: `4ced8bfd470f18ed4dd12ed22ecf2e2ba63d7e3731b0ebdad4c4c4ab820b01b7`.
- `ddfacbc9fa305175a6e0237a7f6e14800a3cede4`의 전체 페이지 검증에서 두 원본
  139쪽의 Native raw는 각각 99.94994%, 표준 최적화 fresh WASM raw는 각각
  99.94958%다. 신규 회귀의 전쪽 90% 선행 조건도 같은 source에서 통과했다.
- 검사는 실제 두 원본의 그림 소유·순서를 보존하고 해당 bitmap만 공통 Skia로
  출력한다. 독립 PNG와 양방향 2px 내용 실루엣을 비교하며 소수 좌표를 고정하지 않는다.
- 수정 전 frozen library에서 2건 모두 실제 FAIL했다. 수정 후와 최종 Native/fresh
  WASM 결과는 작업 계획의 제품 검증 결과에 기록한다.
