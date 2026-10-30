# #8153 최종 후보 전쪽 TSV

2026-10-30 KST, 96dpi, 독립 한컴 Print의 기존 PDF 래스터와 같은 쪽을 `scripts/visual_sweep.py --silhouette-only --png-pair`로 비교했다. 표의 raw 값은 소수 둘째 자리 표시다.

- 고정 base: `ad4bf269ffef5adf2142b33004a9981fad7ce804`.
- Native producer: `5620ff0475`, 실행 파일 SHA-256 `f73d1a3416524ab8e022f394459cba20dc6f096f36cebb82f42b1837d159b6da`.
- fresh WASM producer: `50cdf6701fcd9b646a403f03f61e8c5fd8c2bd74`, 최적화된 WASM SHA-256 `f9e11ac702fc94a0f9abd4033ba27104b86166983085b9c659730a0658b196c9`.
- 이후 `962fd34ce9`까지 제품 로직 변경은 없으며 회귀 검사와 한국어 주석만 변경했다. 생산 커밋과 증거를 추가한 커밋은 구분한다.
- 편람 HWP·HWPX 각 383쪽의 Native·WASM 최저 99.96%는 130쪽이다. 사용자가 승인한 WMF 차이 예외를 적용하며 TSV 값을 변경하지 않는다.
- #8188의 HWP 2022는 각 64쪽 표시 최저 100.00%, HWP 2024·HWPX는 각 64쪽 표시 최저 99.99%다.
- #7905는 이번 범위를 1쪽 개선으로 한정한다. Native 91.44% → 98.98%, WASM 91.23% → 98.91%다. 기존 base부터 3~6쪽이 90% 미만인 사실과 원 수치를 유지하며, 그쪽 개선은 이 브랜치 범위에 포함하지 않는다.
- #8040 두 입력은 각 4쪽이며 fresh WASM 최저 98.82%, base 대비 0.1%p 이상 하락한 쪽이 없다.
- 원본 입력과 PDF의 경로·SHA, 상세 비교·기준 출처는 기존 `rebase-ad4-validation-20261030/README.md`, `mydocs/pr/assets/issue_8160/rebase-ad4-validation-20261030/README.md`, `mydocs/pr/assets/pr_7905_maintainer_20261029/README.md`와 PR 본문을 함께 참조한다.

| TSV | SHA-256 |
| --- | --- |
| [handbook-hwp-native.tsv](handbook-hwp-native.tsv) | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` |
| [handbook-hwp-wasm.tsv](handbook-hwp-wasm.tsv) | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` |
| [handbook-hwpx-native.tsv](handbook-hwpx-native.tsv) | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` |
| [handbook-hwpx-wasm.tsv](handbook-hwpx-wasm.tsv) | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` |
| [sample16-hwpx-native.tsv](sample16-hwpx-native.tsv) | `804fedcc655075b2739abaf7adfb4764c1938357fcaeaaceb62dca421ecb4f49` |
| [sample16-hwpx-wasm.tsv](sample16-hwpx-wasm.tsv) | `b63dc2408d611890f7d966e88f036d930fb514dcaf3e7d82e017d10da28b0ffa` |
| [sample16-hwp2024-native.tsv](sample16-hwp2024-native.tsv) | `804fedcc655075b2739abaf7adfb4764c1938357fcaeaaceb62dca421ecb4f49` |
| [sample16-hwp2024-wasm.tsv](sample16-hwp2024-wasm.tsv) | `b63dc2408d611890f7d966e88f036d930fb514dcaf3e7d82e017d10da28b0ffa` |
| [sample16-hwp2022-native.tsv](sample16-hwp2022-native.tsv) | `418d4bcbe4f906eea1d18e539b57a245dfaa26a79203c693a59af622ffa3caf7` |
| [sample16-hwp2022-wasm.tsv](sample16-hwp2022-wasm.tsv) | `418d4bcbe4f906eea1d18e539b57a245dfaa26a79203c693a59af622ffa3caf7` |
| [issue7905-native.tsv](issue7905-native.tsv) | `3d7f99f46297f0d21f8cc159decd5245b654f7730093909d7c5d4226beb0a95c` |
| [issue7905-wasm.tsv](issue7905-wasm.tsv) | `a5383687e7e1ce0851b3ad33b1db4174296dc97bc6900f5cb09a1a2d0781adbc` |
| [issue8040-insert-native.tsv](issue8040-insert-native.tsv) | `9f894b9dc7b78efff1d152b090a924b611630b4e94aefb20d2dc27aca6451c26` |
| [issue8040-insert-wasm.tsv](issue8040-insert-wasm.tsv) | `0e9ef18698dd82c10d23b762b0278f191f7fe5e99d493c79fa89cf6441402ef3` |
| [issue8040-insert-delete-native.tsv](issue8040-insert-delete-native.tsv) | `9f894b9dc7b78efff1d152b090a924b611630b4e94aefb20d2dc27aca6451c26` |
| [issue8040-insert-delete-wasm.tsv](issue8040-insert-delete-wasm.tsv) | `0e9ef18698dd82c10d23b762b0278f191f7fe5e99d493c79fa89cf6441402ef3` |
| [base-issue7905-native.tsv](base-issue7905-native.tsv) | `8187c52a91cebd22284862746e18682c1086de5e7aaca329fe770c8b288ea426` |
| [base-issue7905-wasm.tsv](base-issue7905-wasm.tsv) | `9afe3da6bf4e67affa9ee10d8ebe39fd620bcb7b9996db3d8d165dac39a6d002` |
| [base-issue8040-insert-native.tsv](base-issue8040-insert-native.tsv) | `9f894b9dc7b78efff1d152b090a924b611630b4e94aefb20d2dc27aca6451c26` |
| [base-issue8040-insert-wasm.tsv](base-issue8040-insert-wasm.tsv) | `0e9ef18698dd82c10d23b762b0278f191f7fe5e99d493c79fa89cf6441402ef3` |
| [base-issue8040-insert-delete-native.tsv](base-issue8040-insert-delete-native.tsv) | `9f894b9dc7b78efff1d152b090a924b611630b4e94aefb20d2dc27aca6451c26` |
| [base-issue8040-insert-delete-wasm.tsv](base-issue8040-insert-delete-wasm.tsv) | `0e9ef18698dd82c10d23b762b0278f191f7fe5e99d493c79fa89cf6441402ef3` |

## 같은 쪽 overlay

색상은 파랑 RHWP만, 빨강 PDF만, 주황 양쪽 색 차이, 회색 일치다. 숫자는 2px 실루엣 표시값이며 이미지 픽셀의 완전 동일을 뜻하지 않는다.

- [handbook-hwp-wasm-p130-overlay.png](handbook-hwp-wasm-p130-overlay.png): `9c0c0d40a03a4bbc91b2eea7da41b2ffc40e2a919082234f4cab8c7350a7f75c`
- [issue7905-wasm-p001-overlay.png](issue7905-wasm-p001-overlay.png): `39ba6e39f9f922b4ecd7706db644d662cf191b580135c48a894ab3a0950ea66e`
- [sample16-hwpx-wasm-p018-overlay.png](sample16-hwpx-wasm-p018-overlay.png): `1a2ced75def8af29db724e42f1f7785684bfb6523def74c7d7d88cf1547f20ed`

## 회귀 검사의 글꼴 환경 확인

- 2026-10-30 KST, 검토 소스 `23acf787e316d5b9adbce0f952c5104b6af315fd`의 관련 회귀 32개 모듈·89건을 Linux에서 실행하여 89건 통과, 실패·무시 0건을 확인했다. 이 모듈들에는 추가·변경한 ±1px 고정형 69개 함수가 포함된다. [함수별 목록](px-regression-inventory.md).
- Fontconfig 목록을 비우고 `RHWP_FONT_PATH`를 해제했다. 이어서 `bwrap`으로 `/usr/share/fonts`, `/usr/local/share/fonts`, `~/.local/share/fonts`, `/opt/hnc`를 빈 tmpfs로 가린 상태에서 관련 테스트 이름과 건수가 일치하는 기존 suite 바이너리를 다시 실행하여 같은 89건이 통과했다. 바이너리는 이름과 건수로 선택했으므로 각각이 해당 소스에서 빌드된 것인지는 별도로 확정하지 않았다. 저장소에 포함된 기존 테스트 자원은 이용할 수 있으며 새 글꼴은 배포하지 않는다.
- 이는 위 Linux 환경에서 시스템 글꼴이 없는 조건의 실행 결과다. 다른 운영체제나 모든 글꼴 조합에서 동일한 결과를 보장한다는 의미는 아니다. 시각 TSV는 별도로 기록된 Print 비교 환경에서 측정했다.
