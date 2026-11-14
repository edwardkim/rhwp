# 원 PR의 실제 입력·페이지에 대응하는 최종 증적

[검증 영수증](validation.json)은 제품 소스·Native 실행 파일·fresh WASM·실제 공급 글꼴·
전체 TSV와 5·6쪽 review/overlay의 SHA-256을 연결한다. 기존 독립 PDF를 재인쇄하지 않았다.

- [Native 전 14쪽](native_silhouette.tsv)·[fresh WASM 전 14쪽](wasm_silhouette.tsv): 각각 모든 쪽 raw 100.00%.
- Native [5쪽 review](native/review_005.png)·[overlay](native/overlay_005.png), [6쪽 review](native/review_006.png)·[overlay](native/overlay_006.png).
- fresh WASM [5쪽 review](wasm/review_005.png)·[overlay](wasm/overlay_005.png), [6쪽 review](wasm/review_006.png)·[overlay](wasm/overlay_006.png).
- [원본·Print 내용 대조](original-content-audit.json): 문단 41의 첫 줄은 5쪽, 나머지 두 줄은 6쪽, 후속 문단 43은 7쪽.
- [저장 재시작 없는 원 PR 반례 A/B](counter-final-ownership-comparison.json): 원 PR 대비 165쪽의 페이지 배치 동일.
  이 자료는 독립 PDF 시각 측정이나 원 작성자의 전체 10k 코퍼스 재실행을 뜻하지 않는다.

두 경로의 5·6쪽 review와 overlay를 직접 판독해 위 주장을 확인했다.
100.00%는 공식 2px 실루엣 지표이며 RGB 픽셀 동일성을 뜻하지 않는다.
전체 보호 Native·편람 gate·나머지 필수 lint와 원격 CI·병합은 별도 최종 기록으로 확정한다.
