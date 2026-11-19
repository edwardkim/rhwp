# 2026-11-19 원 PR 9건 통합 검증

9종을 Native와 fresh WASM으로 출력해 독립 데스크톱 Hancom PDF Print와 대조했다. 각 입력의 후보·고정 devel 전쪽 TSV와 대표 raw·기준 PDF·review·overlay를 함께 보관한다.

- [원본·기준 Print의 경로·해시](visual-inputs.json)
- [원 PR 커밋 대응표](source-commits.json)
- [최소 글꼴 Rust·Studio·리베이스 검증](validation.json)
- [주장 문서 전쪽 점수](claim-comparison.json)
- [정본 보호 문서 4,046쪽 비교](protected-validation.json)
- [실제 Studio Canvas2D·CanvasKit 확인](actual-consumer-validation.json)
- [증적 파일 SHA-256](artifacts-sha256.json)

Helvetica 스타일 보호 문서는 Arial 네 스타일을 공급한 조건에서 Native와 fresh WASM 모두 100.00%였다. 파일과 공급 방법은 [Visual Sweep 가이드](../../../manual/verification/visual_sweep_guide.md#helvetica-스타일-보호-문서의-arial-공급)에 기록했다.
