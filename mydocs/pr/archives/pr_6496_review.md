# PR #6496 검토 - 한양신명조 SVG/Studio 별칭

- 원 PR head: 5088d1b00df5763088b6ebdb4cd6f8830ea73e1d
- 통합 cherry-pick: 0e846c3d02d8bceedce3c79eca441957223b7f52, e25e1681d93b157ffdefc49726f8227c3aaeee93
- 통합 기준: 76532b4da0e720026fb24211ad0c382884d3b970

## 판정: 메인터너 보정 됨 수용 가능

## 확인한 범위

renderer font alias와 두 SVG golden의 체인을 변경한다.

## 검증 및 증적

공통 전체 회귀, Native Skia, 배포용 WASM이 통과했다.

원 PR에는 Chrome과 설치 글꼴 비교 증적 및 golden diff가 있다.

## 다음 조건

설치 글꼴이 있는 검증 호스트에서 current-head SVG/Studio 출력의 실제 font-family 선택과 raster 차이를 기록한다.

공통 검증 세부 내용은 pr_6489_6517_planet6897_integration_evidence.md를 따른다.
## 2026-08-31 메인터너 보정 검증

**최종 판정: 메인터너 보정 됨 수용 가능.**

- 비공개 원본 `156573118`의 저장 메타데이터는 product 미상, version `9.6.1.10097`로 2024 저장본이 아니다. 따라서 Hancom `2020` profile로 기준 PDF를 생성했다: SHA-256 `877868a985aa66b68fc16dd37962e584e0ad6fa90bdec6c375bd738b8efb6f22`.
- 이 문서는 `printMethod=4` N-up이므로 rhwp 논리 9쪽과 Hancom 물리 5쪽을 1:1 픽셀 비교하지 않았다. sweep은 이 불일치를 안전하게 거부했다.
- Windows font registry의 `H2MJSM.TTF` 내부 family는 `HYSinMyeongJo-Medium`이고, 현재 SVG는 `HYSinMyeongJo`/`HYSinMyeongJo-Medium` fallback chain을 실제 방출한다.
- 8쪽 table `LAYOUT_OVERFLOW` 82.7px은 base와 현재 후보에서 위치와 수치가 동일하므로 이 alias 보정의 신규 회귀가 아니다.
