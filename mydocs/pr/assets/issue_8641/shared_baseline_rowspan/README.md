# 이슈 #8641 — 같은 줄 TAC 표 공유 기준선과 rowspan 초과분

- 후보 source: `7f85c9f4fa5ee95a9d58eeb569794ad89c9e1dc5`. 측정 빌드는 제품 코드가 같고 테스트 머리 주석만 다른 사전 커밋에서 만들었다.
  `src`·`samples`·`pdf`·`tests/fixtures` 는 동일하며, 이 커밋에서 fmt·clippy 3종·전체 회귀를 다시 통과했다.
- 비교 base: `4db4c75ae4eff93b08e37a5e4ee02d90a3e2d435`. `upstream/devel`(`170b886f16`)에 PR #8645·#8649 head 를 쌓은 커밋이다.
  두 PR 이 병합되면 최신 devel 로 rebase 하고 재검증한다.
- 빌드 SHA-256: 후보 Native `191012b78a3f25e4f9200c69c125de15db0172e8e726106cce106105d4014a0f`, 후보 WASM `634bb17fc79293b2470a53526816a80f50c72ea15b1d118b99c12e66dbb84966`,
  base Native `dc66035ed75271d77eef69c81cef7c757ea8d4bc1ca159b95ea56996fbfc2b2b`, base WASM `6ce5854f24cb27c1c819ac97621dafbddacf1f1b44d85a8954d73fca8265091c`.
- 측정: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96 --font-path ttfs/opensource`,
  판정값 `silhouette_raw_match_percent`.
- 기준 PDF: `pdf/issue8641/<입력>-2020.pdf`(한/글 2022 Print, Creator Hwp 2022 12.0.0.4547).

| 입력 | 전체 쪽수(한컴=rhwp) | 대상 쪽 | base Native | 후보 Native | base WASM | 후보 WASM | 전쪽 최저 Native / WASM |
| --- | --- | --- | --- | --- | --- | --- | --- |
| inspection_request_stamp_a | 1 | 1 | 89.06 | 98.02 | 88.80 | 98.00 | 98.02 / 98.00 |
| inspection_request_stamp_b | 1 | 1 | 89.04 | 97.67 | 88.83 | 97.66 | 97.67 / 97.66 |
| labor_standards_report | 53 | 13 | 88.08 | 99.92 | 87.98 | 99.90 | 94.57 / 95.61 |

직접 판독: 결재표 위끝·표 높이·행 경계·뒤 문단 위치는 Native·fresh WASM 모두 PDF 와 맞는다.
남은 차이는 일부 글자 굵기(수정 전후 잉크 양 동일)와 보고서 13쪽의 U+2024 모양이며 이 변경과 무관하다.

코퍼스 10,000건 base/후보 SVG 대조: 109건 변경, PDF 대조 40건, PDF 없음 69건(한컴 대조 불가).
가장 큰 저하는 05859 3쪽 74.75→74.44 다. 쪽수 변화는 07302 하나로 242→241 이며 PDF 241쪽과 같다.
