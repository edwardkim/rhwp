# 최신 리베이스 Native의 필수 편람 전쪽 게이트

**383쪽 passed·실제 exit 0**. 전쪽 최저 표시 raw **99.96%**, 소수 두 자리 표시에서 382쪽이 100.00%다.
기준 대비 표시값 하락·누락·웹폰트 fallback은 모두 0건이다. 이번 실행의 TSV 원문은 공식 기준 TSV와 바이트도 같다. 평균·대표 쪽으로 미달을 상쇄하지 않았다.

제품 source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`, Native SHA-256 `296a0d3f68dcb29f5bd91f3f5411bb78e42c53d04a51f526a3473281ba090cc6`.
실제 Native build head `a2c6d75ec81d9c155dbfe2e58ba814a53c5a33a7`의 제품 입력 해시는 고정 source와 같다.
전체 nextest·fresh WASM·새 base 자체 대조의 완료로 확대하지 않는다.

- 입력: `samples/2025 행정업무운영 편람(최종).hwpx`.
- 독립 Print: `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`.
- 입력 SHA-256: `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`.
- PDF SHA-256: `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e`.
- 공식 비교 기준: `mydocs/pr/assets/issue_8153/final-50cdf-validation-20261030/handbook-hwpx-native.tsv`.
- 기준 TSV SHA-256: `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308`.
- [실제 Native 전383쪽 TSV](handbook-native-silhouette.tsv), SHA-256 `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308`.
- 96 DPI·Native Skia·Print profile·색상 경계 보정 전 2px raw, 기준 계약의 0.01%p 표시 정밀도.
- 시작 UTC `2026-11-05T01:31:36.488170+00:00` → 종료 UTC `2026-11-05T02:10:48.962694+00:00`.
- 원시 로그 SHA-256: `df13a0553d7bd8da95a4ce74dbe462c5fa5acfb3b59dfab3bf6dd5d4c19f8e68`.

독립 PDF·문서·baseline·허용값은 변경하지 않았다. 기존 지정 글꼴 계약과 시스템 글꼴을 사용하며
검증용 글꼴 프로그램을 제출·재배포하지 않는다. 실제 실행 명령은 다음과 같다.

```bash
/home/tsjang/rhwp/venv/bin/python scripts/visual_sweep.py --handbook-gate --out /home/tsjang/rhwp/output/pr-review/planet6897-20261103/handbook-gate-rebase8609-dda217dba0 --rhwp-bin /home/tsjang/rhwp/output/pr-review/planet6897-20261103/rhwp-rebase8609-dda217dba0 --page-workers 1 --font-environment mydocs/pr/assets/planet6897_integration_final/pr8389_8397/handbook-font-environment.json
```

이 입력은 최종 필수 편람 게이트로 실행한 대조다. 현재 보정 대상 10입력·43쪽의 원 PR 개선
증거로 대체하거나 새로운 편람 개선 PR 범위로 확장하지 않는다. 원시 JSON·로그는 로컬 output에
유지하며 [통합 상태](../README.md)에 완료·진행 중 검증을 구분한다.
