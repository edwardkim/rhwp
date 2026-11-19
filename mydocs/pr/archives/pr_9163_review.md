# PR #9163 검토 · 실제 Flutter GPU 메모리 압박 경로 검증

**검토 완료.** [통합 PR #9189](https://github.com/edwardkim/rhwp/pull/9189) 코드 head `5b3e013dfb49e82f24fc769f81170e2310adf612`의 Full CI와 적용 대상 검사가 모두 통과했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/44470122043) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44470122101) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44470121749) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44470122018) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44470122039) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44470121845)

이번 검토 기록을 반영한 head의 필수 검사를 확인한 뒤 병합한다.

[원 PR](https://github.com/edwardkim/rhwp/pull/9163) head `50691c622f0837580d759b8503c003b082e000df`를
`c526dd402bcf55d09cb8d0993537698a25d4740a` 기준 누적 브랜치 `review/integration-8701-seo-rii-desktop`에 반영했습니다.
로컬 검증 소스는 `64ba35858d4f511bdd993f24eae9337880a7f2b2`다. 위 원격 코드 head에는 검증 증적만 추가했다.

## 원 PR 주장

옵션으로 시스템 메모리 압박을 발생시켜 라이브 trim receipt, idle 상태, 다시 화면 출력, 자원 정리를 검사합니다.

## 체리픽 추적

| 원 커밋 | 적용 커밋 |
| --- | --- |
| `409c3bfd5c4bac161ea4970c23ffd4d556932efb` | `68c72745bb3a8f8237649bad72fc2c11f41247c9` |
| `f3f9c2844887fca4250ffa3a7493d2e8a8967111` | `fc29424c50a28ac4f9f9bc66c310c89596ff1177` |
| `438da907cba4d6aa4222146497c3e5457cb67bb5` | `ccb542de06dfaf61d36a12f4ba05ddfdab219ddc` |

## 실제 검증 범위

압박 인자와 supervisor 연결, single/two-tile 실제 소프트웨어 GL, trim 바이트·화면 재개·close 기록

Mac의 Flutter 3.47.9·Dart 3.13.7에서 strict analyze를 통과했고, 순차 전체 Flutter 테스트는 388건 통과·8건 건너뜀이다.

Xvfb 소프트웨어 GL에서 단일 화면과 두 타일에 각각 두 차례 메모리 압박을 주었다. 단일 화면 3,590,144바이트와 두 타일 19,324,416바이트가 회수되었고 각 실행의 화면 재개를 확인했다. 물리 VRAM 측정과 GPU 완료 검증은 결과에 포함되지 않는다.

[공통 보호문서 검토](../assets/issue8701-desktop-integration-20261119/protected-review.md)에서 Native·fresh WASM 전쪽 결과와 정본 TSV를 확인할 수 있다.

## Merge 후 contributor PR comment 계획

통합 PR의 병합 SHA와 최신 devel 반영을 확인한 뒤 원 PR에 링크를 남긴다. Xvfb 소프트웨어 GL에서 관찰한 단일 화면·두 타일의 두 차례 관리 메모리 회수와 화면 출력 재개를 적는다.
