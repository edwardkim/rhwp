# PR #8184 적용·보정 실행 기록

## 2026-10-28 원 PR 추가 커밋 반영 중

재조회 head `cc6d06077013daf6090021ab317c347dbd05b236`에는 이전 체리픽 후 커밋 세 개가 추가됐다. `6fe6d9fdb01c`의 `first + 1` 호출은 기존 통합 보정 `7fd587fc52`(순서 정리 후 `da2b3ae1e6`)에 이미 있다. `ffbf463e0081`은 처음 `58063983b6`으로 체리픽한 뒤 #8149 구간으로 순서를 옮겨 `c4605f7fed`가 됐고, 충돌한 첫 조각 앵커 부분은 HWPX 저장 행 경계를 보호하는 현재 브랜치의 조건을 유지했다. 따라서 `58063983b6`의 원 커밋 메시지가 언급하는 앵커 대안 제거는 **적용하지 않았다**. 다른 두 파일의 간격 흡수·저장 원점 조건은 적용했다. `cc6d06077013`은 처음 `6506bd2aee`로 적용했으며 같은 순서 정리 후 `895f8a60bf`가 됐다. 순서 정리 리베이스는 충돌 없이 완료됐고 전후 최종 트리 SHA는 `c9722a0cc2027a69c51be9f71fcede1bca1020d9`로 일치한다.

이 세부 충돌 해소와 새 보정은 아직 Native·fresh WASM 전쪽 및 기존 사례 하락 검증 전이다. 최종 수용으로 판정하지 않는다. [원 PR head·patch 비교](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-patch-refresh-20261028.json).

- 원 head: `567ff9d18ac1f7e702e94042eb38a03a44f95943`
- 통합 branch: `review/integration-20261026-document-updates`
- base: `faa885e8b54ffc7cdf4196cd43a3103f362499a2`
- 완료한 로컬 code/test 검증 head: `2e902abacf328bccdd7780606666ac3b1c5c1ce9`
- 최종 판정·입력·명령·제한: [pr_8184_review.md](pr_8184_review.md)
- 단계 계획·독립 근거: [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)

## 실제 체리픽 순서

| 순서 | 원 SHA | 실제 적용 SHA / 처리 |
| ---: | --- | --- |
| 1 | `17e8267d99eeef5a5d32a304433ef0f84c20ecaf` | `cbf8e6100429ea3ec79a28b8cbbba87f30cb6785` |
| 2 | `5918b082bf14f5d2b8fb030087a3da35e545804f` | `d8dbbb2c94282b0b20e267e6fd22cec04112016d` |
| 3 | `90386d7de39a0092d645184c6ad783d16ae30013` | `2df0904b57c743c8876e073930bb7dc9b39613d5` |
| 4 | `f7626011e8c48a61ce589f11c93fd84608d149f7` | `622e0f5bfac20173e2fb8ec8c435fda2e1bf5358` |
| 5 | `0eab5aef81e66973001241f590af0e84186fe762` | `8f1bb1cc84e71d09d7c1f06655646bbd54992c85` |
| 6 | `553805f8615f19abaf3190216eb98d7f493b49f7` | `68329b8cdab5eab2530a692fb08c14b8255a05bc` |
| 7 | `05cef5908cf6d7b7a7580f38bfd7ae888b96e16f` | `d4056dacab50cbbfb7b804d8ede7256786cf6bcc` |
| 8 | `c255b7320d1ef4eeff93d796877d37bbe8aa99eb` | `378162362dc502dd0c1fe13b89937723c99c0cfd` |
| 9 | `add1ee699818af2a8364dee9ff051425a50da272` | `3423f6b342152483c24bd155f5477104f0eef776` |
| 10 | `2c6ebe553de79a3b21e0adaf5b2942322fd889e9` | `0673c9e37489a1d0fcfb3de9eb6bc3fbca968b09` |
| 11 | `ca6112a6c91c2eef40f1b46e88e442b7723bdc2c` | `53b7eb968aa77d57c89322699a72f4078d4dae26` |
| 12 | `29dbddf9ce78156860c2625598f4dacda0811ecc` | `b7f4667f65b467c1fa7ce0ed8745a6c9f2828f8d` |
| 13 | `62d23c09b1ff6f430484ddc46d57f2dd036e1ea1` | `4ce88676f604d4079c0e0ffa8406098d5656d5c8` |
| 14 | `78ba587d11ec911731e1c6eaa5a1a28c956536d3` | `0142af3ed8ec19c862afcd1058403c858738bc61` |
| 15 | `b9334d5e2039c66261550da0f8bd4333a982d041` | `15ff514a260dc3598378caa102801da2899edaac` |
| 16 | `5be3dd4c710f76af75ad9bd1efd92b7b1a02c9ca` | `3e225817697842c1213925793d87067b99a4a228` |
| 17 | `67525acc3c30cdc91ed78bb92e14db3dcb9f63bf` | `b1c7c0dbb27616ea56f8cf74aabef7e63e3d1c1d` |
| 18 | `821ee71a40107fdbec3ba20289e49833e87fb122` | `28a88267f4f723925769f104e321294fd1e40fd4` |
| 19 | `00cd6ef8275bc0c96a08ccbce4a1a7e4150c866a` | `2e52dd776212c133445a9beb3107d7b441d737d7` |
| 20 | `1db32c93f993d1e37bd0f204ac81eca7b86632ec` | `8262e9986dfbc173b91278133a083f2d2d171aa4` |
| 21 | `f862393e880095a1fdb8ff356e0f7aa7a3b78110` | `3a43a4f34f21316e632ab03aa74ccf69f4be1c47` |
| 22 | `d878c2ad53a5caca5d1d099fdc2d5da3a718add1` | `17f9bc221ae9da0fac3ecb2d304935750291a73b` |
| 23 | `1aa8a62489894af1b0c8adf2154e4422eb00eea9` | `0db8d21f93b849f1421fca4481f9a07503f0662c` |
| 24 | `567ff9d18ac1f7e702e94042eb38a03a44f95943` | `2cc4ef3226e377018ba328fd471382a436b5482a` |

적층 선행 commit은 patch-id로 대조했다. merge commit을 다시 체리픽하지 않았고 원 저자·`-x` 출처를 보존했다.
#8227·#8237의 사전 가상 충돌은 적층 부모를 명시한 예상 tree와 실제 결과가 일치했다. 실제 체리픽 충돌은 없었다. 이후 #8116 리베이스의 line_breaking.rs 한 충돌은 두 수용 조건을 모두 유지해 해소했다. 리베이스 전후 SHA·원 저자 보존은 커밋 대조 원장에 기록했다.

## 보정·검증·후속 경계

1. 고정 원 head 적용과 예상 tree·중복을 원장으로 확인했다.
2. 승인된 기존 미달 페이지를 공통 로직에서 보정하고 단계별 일반 커밋으로 고정했다.
3. 관련 전쪽 Native/fresh WASM 선행 검증과 직접 판독 뒤 정식 소유 회귀를 보강했다. 후속 보정 뒤 기존 통합 67개 입력과 표 조각 36개 입력을 확인했다. #8319 보존 대상 197개 입력·256쪽도 Native와 fresh WASM 각각 전쪽 렌더 PNG가 보정 전과 같다. [Native](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-incoming-native-same-raster-comparison.json), [WASM](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-incoming-wasm-same-raster-comparison.json).
4. 이전 전체 회귀와 후속 보정의 선택 회귀를 구분해 기록했다. 작업지시자 지시에 따라 전체 회귀는 재실행하지 않는다. 현재 남은 전쪽 비교와 미해결 기준을 모두 마치기 전에는 제출 전 전체 검증 완료로 판정하지 않는다.
5. 최종 로컬 커밋·본문을 준비하고 작업지시자 승인 후 원격 push·통합 PR 제출을 진행한다. 이후 CI·머지·원 PR 종료는 별도 후속 조건이다.

보정 SHA: `a30b10d0967e98771dd8e90d6ad6748b2f14a52e`, `80e3cb7490e05294283dd45a545e41293bdfb4de`, `607c578e3b2cf1bca5c58a0ca0e679be97f23582`.

원 contributor head를 rewrite하거나 원격 브랜치를 변경하지 않았다. 되돌림은 필요 시 이 원장의 적용 commit과
관련 공통 보정을 검토해 별도 revert로 처리하며 다른 PR 변경까지 한꺼번에 롤백하지 않는다.

## 2026-10-28 기존 패치의 최신 원 PR 대조

기존 3개 patch-id 차이는 주로 주변 코드와 새 기반의 호출 인자에서 발생한다. 통합 후보에서 아래 캡션 높이 소비, 빈 host 표 첫 조각 예약의 진입·예산 소비, 다음 저장 줄을 이용한 단 원점 계산을 각각 확인했다. 후속 체리픽의 앵커 충돌 보존과 전쪽 시각 검증은 별개로 추적한다. [커밋별 대조](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-paired-range-diff-review.md).

## 2026-10-28 후속 보정 뒤 표 조각 전쪽 검증

독립 Print를 가진 36개 입력은 Native와 fresh WASM 각각 총 64쪽 모두 raw 100%다. 원본·PDF SHA, 각 쪽의 렌더 및 PDF 래스터 SHA와 수치는 [검증 원장](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-float-fragments/evidence.json)에 기록했다. 공식 사양서 71쪽도 이전 PDF 래스터 기준 두 경로 전쪽 수치가 보정 전과 같음을 [별도 원장](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-spec-all/evidence.json)으로 확인했다. 추가 보정과 관련된 다른 입력·최종 gate는 별도이며 이 결과만으로 일괄 완료하지 않는다.

## 2026-10-28 후속 보정 대표 쪽 전후 비교

같은 원본·독립 PDF·환경의 4쪽에서 0.1%p 이상 하락은 없다. `hcar-001.hwpx` 1쪽은 Native 74.87266% → 94.24519%, fresh WASM 75.42285% → 94.72286%다. `rowbreak-problem-pages.hwpx` 11쪽은 Native 95.74925% / WASM 95.75079%로 보정 전후 같고, 정책연구 보고서 174·175쪽도 두 경로의 수치가 각각 같다. [입력·PDF·PNG SHA와 전후 수치](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-representative/evidence.json). 기존 hcar의 90% 미달은 보정 전 관측이며 보정 후에도 100%라고 주장하지 않는다.

## 2026-10-29 후속 선택 회귀

후속 표 보정과 관련된 `issue_8149_empty_host_float_page_bottom`, `source_owned_float_fragment_frames`, `issue_8160_saved_page_boundaries` 검사만 선택해 실행했으며 24개 통과, 종료 코드 0이다. 필터에서 제외된 794개는 이 실행에서 검사하지 않았다. [실행 명령·검사명·로그 해시](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-after-targeted-nextest.json).
