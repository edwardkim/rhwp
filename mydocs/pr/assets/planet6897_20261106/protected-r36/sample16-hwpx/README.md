# sample16-hwpx 전쪽 검증

- 입력: `samples/hwp3-sample16-hwp5.hwpx`, SHA-256 `49e3e809eb41e22b2c059383db32b0cf038787269b5c523d1ff59d1a52b4340c`.
- 독립 Print: `pdf/hwp3-sample16-hwp5-hwpx-2024.pdf`, SHA-256 `be4a0286d3ad5209a1a0787fd4ff691e110ddbcf7bde3a52a4547958e452a1ee`.
- Print 프로필, compat 2024, 96 DPI. 원본과 기준 PDF는 다시 만들지 않았다.
- 승인된 기존 글꼴 환경: `mydocs/pr/assets/planet6897_integration_final/pr8389_8397/hwp3-2024-font-environment.json`와 시스템 전체 글꼴.



## R36 전 64쪽 검증 완료

2026-11-07에 같은 입력·독립 Print의 전 64쪽을 대조했다. Native 전쪽 최저
**99.99%**, 실제 R36 fresh WASM **99.99%**, 90% 미만·쪽수 불일치 0건이다.
[Native 전쪽 TSV](native-full.tsv) · [fresh WASM 전쪽 TSV](wasm-full.tsv).

WASM 제품 source는 `3f2a7c96ee1ec86128b6ca1c3a6b700aa2eaeb3b`, 프로그램 해시는
`5783f16d661696184aed7c8f1d41a0f057ffa4b0d3d8f97234e277b411c106b9`다.
Native는 `ef77d740e1`의 실제 출력을 재사용했다. R33→R36 제품 변경이 wasm32 전용 메서드·모듈에만
있고 Native 코드·입력·환경이 불변인 것을 diff와 파일 해시로 확인했다.
원 PR #8389·#8397의 해당 입력 64행은 표시값 하락 0행이다. 나머지 14개 보호 입력과
현재 통합 PR의 최종 게이트를 이 결과로 대신하지 않는다.

전쪽 명령은 `scripts/visual_sweep.py --silhouette-only --resume`에 위 입력·PDF·글꼴 환경,
`--compat 2024 --dpi 96` 및 R33 Native CLI·R36 fresh package를 지정한 것이다.
집중 명령의 `--pages` 없이 전체 64쪽을 출력했다.
