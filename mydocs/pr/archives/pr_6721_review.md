# PR #6721 검토 기록

- 원 PR: [#6721](https://github.com/edwardkim/rhwp/pull/6721)
- 기여자: `planet6897`
- 원본 head: `75ef75522844bb52b7831ac6d9fa5b1a129148b2`
- 통합 적용 commit: `0832523dd3ab19093fb44f39f2ffcb29dd8a5d72`

## 판정: 메인터너 보정 됨 수용 가능

원 PR의 native HWP5 `vpos == 0` 쪽 경계 보정은 그대로 전역 규칙으로 적용하면 다른 HWP5 문서의
정상 좌표까지 되감길 위험이 있었다. 메인터너 보정으로 실제 저장 line-segment 계약에 한정해
수용했다.

## 메인터너 보정

- private 환경 변수·경로 탐색을 제거하고 공개 정식 fixture
  `samples/issue6718/27469-child-allowance-retroactive-support.hwp`와 manifest를 등록했다.
- page reset은 native HWP5 profile에서 `vertical_pos == 0`인 모든 줄이 아니라, 현재 stored line
  segment의 `line_height=1200HU`, `baseline_distance=1020HU`, `line_spacing=1440HU`,
  `tag=0x00160000` 계약까지 일치할 때만 허용한다.
- 원 PR의 page 2 및 page 4 회귀 테스트를 유지했고, #2070 등 인접 native HWP5 제어 회귀를 함께
  확인했다.

## 실제 검증

`upstream/devel` rebase 전 통합 후보에서 다음 targeted control과 전체 회귀를 완료했다. 이 PR
head는 그 뒤 최신 `upstream/devel` 위로 rebase됐으므로, 병합 판정에는 PR CI 결과를 별도로
사용한다.

```text
# native HWP5 source/adjacent controls
16 tests run: 16 passed, 9046 skipped

cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review-planet6897-green-batch-20260904 \
  --tests --no-fail-fast

Summary [238.794s] 9016 tests run: 9016 passed (3 slow, 1 leaky), 46 skipped
```

현재 fixture의 남은 baseline은 `text-overlap=2`다. 이는 이 PR의 page reset 계약이 아닌 잔여
문서 충실도이며, 이를 해결했다고 주장하지 않는다.

## 시각 증적 범위

원 PR이 제공한 page 2 source asset은 zero-vpos reset의 국소 전후를 보여 준다. source asset은
원본 변경의 범위를 설명하는 자료이고, 최종 통합 head에서 새로 export한 이미지라고 주장하지
않는다.

| 구분 | 자산 | SHA-256 |
| --- | --- | --- |
| 수정 전 | [before_p2.png](../../report/6718-native-hwp5-zero-vpos-rewind/before_p2.png) | `22399f19935a8582b8f8bc29ecc7121ed972683974f445b6a2045938dfe30b12` |
| 수정 후 | [after_p2.png](../../report/6718-native-hwp5-zero-vpos-rewind/after_p2.png) | `e625021f7ae4e198839b3c52812877482065526f7e6a71ecca951b65a7a2eade` |

![#6721 p2 수정 후](../../report/6718-native-hwp5-zero-vpos-rewind/after_p2.png)

## 보류 범위

- 원 PR이 언급한 page 8 tail overflow와 다른 전역 layout 문제는 이 좁은 보정의 완료 범위가 아니다.
- #2070 등 기존 native HWP5 동작을 바꾸지 않는 fail-closed 조건을 유지한다.
- 원 PR을 직접 병합하지 않고, 이 통합 브랜치의 provenance-preserving 체리픽으로 수용 후보에
  반영한다.
