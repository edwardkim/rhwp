# R41 리베이스 후 Native 전쪽 실측

- 제품·Rust 검증 source: `851e4074514935e4c6a7377f71a97f0cca454191`.
- 새 base: `e4ffabea38062e716f6c333cc950fa31b42b4dd7`.
- Native producer SHA-256: `45f9b5a2fe5bf0a4eede663b22841146f496d58cb02cfd3f9fbf069104a3b222`.
- 실제 완료: 56개 입력·662쪽, 독립 Print 쪽수 일치, 전쪽 최저 92.99%.
- 96 DPI, Print profile, 색상 경계 보정 전 2px 내용 실루엣. 실제 실행은 `visual_sweep.py --silhouette-only --resume`.
- [입력·Print·TSV 해시와 전쪽 결과](inventory.tsv). 원본·PDF 경로는 저장소 루트 기준이다.
- [주장 70쪽의 직접 판독 재사용 근거](claim-reading-reuse.tsv): 새 실행의 RGBA와 동일 독립 PDF 래스터를 이전 직접 판독 자료와 대조하여 70쪽 모두 같음을 확인했다. 연결한 PNG는 실제 동일 입력·동일 쪽의 이전 자료이며 새 PNG로 가장하지 않는다.
- `contract_review_pending=True`인 기존 관측은 그대로 보존했다. 동일 픽셀 재사용은 해당 관측의 해결 판정이 아니다.
- R38 TSV와 내용이 같은 입력 56개 / 56개. R38의 과거 실행과 R41의 새 실행은 source와 producer를 분리했다.

이 자료는 완료된 Native 경로의 결과다. fresh WASM·최신 base 전쪽 대조·전체 Rust 회귀 및 최종 PR별 판정을 대신하지 않는다.
