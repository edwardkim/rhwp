# PR #6719 검토 기록

- 원 PR: [#6719](https://github.com/edwardkim/rhwp/pull/6719)
- 기여자: `planet6897`
- 원본 head: `78f8a5938eef60afbbb8022729cf6620fa92c615`
- 통합 적용 commit: `b8411986b2afca9546c063088a2e696fd42742c1`
- 원 PR의 closing reference: `Fixes #6550`

## 판정: 승인

원 변경은 endnote 제목의 저장 `vpos`가 직전 note tail보다 위에 있을 때 제목을 위로 snap시키지
않도록 하며, 정상 간격(`<= 1984HU`)과 정상 tail 범위만 보정한다. 대상 범위와 제외 범위가
명시되어 있고, rebase 전 통합 후보 회귀에서도 통과했다. 현재 PR head의 병합 판정에는 PR CI
결과를 별도로 사용한다.

## 실제 검증

`upstream/devel` rebase 전 통합 후보에서 다음 전체 회귀를 완료했다. 이 PR head는 그 뒤 최신
`upstream/devel` 위로 rebase됐으므로, 병합 판정에는 PR CI 결과를 별도로 사용한다.

```text
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review-planet6897-green-batch-20260904 \
  --tests --no-fail-fast

Summary [238.794s] 9016 tests run: 9016 passed (3 slow, 1 leaky), 46 skipped
```

별도 #6719 후보 검증에서는 issue fixture의 p18에 대해 `pageCount=20`, `offCanvas=0`,
`textOverlap=0`, `overflow=2`, `overlap=0`을 확인했다. 이 수치는 문서 전체의 완전한 시각
동일성을 주장하는 값이 아니라, 이 PR의 endnote 제목 범위를 확인하기 위한 layout-anomaly 결과다.

## 시각 증적

아래는 #6719 후보에서 확인한 동일 p18 범위의 Hancom 2020 기준과 rhwp 후보 렌더다. 후보 PDF와
PNG는 이 통합 브랜치에 #6716/#6721 및 메인터너 보정이 누적되기 전에 생성됐다. 따라서 #6550의
국소 증적으로만 사용하며, 최종 통합 head의 새 export 결과라고 표기하지 않는다.

| 구분 | 자산 | SHA-256 |
| --- | --- | --- |
| Hancom 2020 기준 p18 | [hancom-2020-p18.png](../assets/pr_6719_planet6897_20260904/hancom-2020-p18.png) | `ce05afb0e32cf2056dae578463ace84c28e81749df0f4cab494f08da633c1dc1` |
| rhwp 후보 p18 | [rhwp-p18.png](../assets/pr_6719_planet6897_20260904/rhwp-p18.png) | `620ddf6de6911e7bf93ee1c31a95b7835c2f0ebe1eaf0703090ad657583ab1a2` |
| rhwp 후보 PDF | [rhwp-3-09월_교육_통합_2023.pdf](../assets/pr_6719_planet6897_20260904/rhwp-3-09월_교육_통합_2023.pdf) | `438d1a3cf28fbe4023c2eaf50fe7c396d526852f84d327ff6d1c60defc624593` |

![#6719 Hancom 2020 p18](../assets/pr_6719_planet6897_20260904/hancom-2020-p18.png)

![#6719 rhwp 후보 p18](../assets/pr_6719_planet6897_20260904/rhwp-p18.png)

## 후속 처리 경계

- `Fixes #6550`의 issue close는 통합 PR이 병합되고 devel CI가 성공한 뒤에만 처리한다.
- 원 PR을 직접 병합하지 않고, 이 통합 브랜치의 provenance-preserving 체리픽으로 수용 후보에
  반영한다.
