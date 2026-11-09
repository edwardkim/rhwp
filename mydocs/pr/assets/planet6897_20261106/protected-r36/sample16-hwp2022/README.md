# 2022 HWP 51쪽의 등록 프로그램 소비 복구

이는 PR #8389·#8397 보호 입력 중 실제 실패 쪽의 집중 검증이다. 원본 64쪽 전체의
새 WASM 검증이나 전체 PR의 최종 통과를 이 한 쪽으로 대신하지 않는다.

- 입력: `samples/hwp3-sample16-hwp5-2022.hwp`, SHA-256 `fec3671c9a83cb8388cbdaab38754cb17a8da54d0314ca7e3f56860e871506fe`.
- 독립 한컴 Print: `pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf`, SHA-256 `c4e05635726869d737f1640db4e4f29c868df9e1c31e350b0b06e115fd210b33`.
- 원본과 기준 PDF는 그대로 사용했다. 글꼴 프로그램을 저장소에 추가하지 않았다.
- 2026-11-07, Print, compat 2022, 96 DPI, 보정 전 2px raw. 승인된 기존
  `hwp3-2020-font-environment.json`과 시스템 전체 글꼴을 공급했다.
- 제품 source: `3f2a7c96ee1ec86128b6ca1c3a6b700aa2eaeb3b`.
- 실제 최적화 WASM SHA-256: `5783f16d661696184aed7c8f1d41a0f057ffa4b0d3d8f97234e277b411c106b9`.
- 실제 R35 출력은 99.90%, R36은 **100.00%**, 기존 기준은 100.00%다.
- [같은 51쪽 TSV](wasm-selected-p051.tsv), [review](wasm-review-051.png), [overlay](wasm-overlay-051.png).

원본 HWP5의 미정의 비트를 TrueType 선언으로 재분류하지 않는다. 기존 `f70a8c937d3`의
정책을 유지하면서, 실제 등록된 같은 family의 CFF 윤곽을 합성 굵기 경로에서도 소비한다.
실제 TTF 별칭이 그 글자를 소유하면 기존 선택을 유지한다. 원점·전진폭·조판과 PDF를
수치에 맞춰 변경하지 않았다. 원 기호 두 개·제목·밑줄·뒤 표와 본문을 같은 배율로 판독한다.
raw는 2px 이웃 판정이므로 픽셀 동일이라는 뜻으로 사용하지 않는다.

기본 재현 명령은 `scripts/visual_sweep.py --silhouette-only --resume --hwp <위 원본>
--pdf <위 Print> --key sample16-hwp2022 --compat 2022 --dpi 96 --pages 51
--font-environment mydocs/pr/assets/planet6897_integration_final/pr8389_8397/hwp3-2020-font-environment.json`
이며, Native CLI는 `rhwp-batch-corrections-r33`, 실제 fresh package는
`wasm-cff-source-selection-r36`을 지정했다. 원 PR #8605의 다른 권고문 증거로 대체하지 않는다.


## R36 전 64쪽 검증 완료

2026-11-07에 같은 입력·독립 Print의 전 64쪽을 대조했다. Native 전쪽 최저
**100.00%**, 실제 R36 fresh WASM **100.00%**, 90% 미만·쪽수 불일치 0건이다.
[Native 전쪽 TSV](native-full.tsv) · [fresh WASM 전쪽 TSV](wasm-full.tsv).

WASM 제품 source는 `3f2a7c96ee1ec86128b6ca1c3a6b700aa2eaeb3b`, 프로그램 해시는
`5783f16d661696184aed7c8f1d41a0f057ffa4b0d3d8f97234e277b411c106b9`다.
Native는 `ef77d740e1`의 실제 출력을 재사용했다. R33→R36 제품 변경이 wasm32 전용 메서드·모듈에만
있고 Native 코드·입력·환경이 불변인 것을 diff와 파일 해시로 확인했다.
원 PR #8389·#8397의 해당 입력 64행은 표시값 하락 0행이다. 나머지 14개 보호 입력과
현재 통합 PR의 최종 게이트를 이 결과로 대신하지 않는다.

전쪽 명령은 `scripts/visual_sweep.py --silhouette-only --resume`에 위 입력·PDF·글꼴 환경,
`--compat 2022 --dpi 96` 및 R33 Native CLI·R36 fresh package를 지정한 것이다.
집중 명령의 `--pages` 없이 전체 64쪽을 출력했다.
