# Dependabot PR 19건 로컬 최종 검토 — 2026-11-09

기준 devel `024d610633ede166ebcbb915f9c90e7a9d0649d8`. 검증 source `2320351d2325a41d900e91f4e07f559d4a924a7d`의 결합 graph가 로컬 필수 검증을 통과했다. 승인된 원격 처리로 통합했다.

추가 하네스·고지 보정 commit은 `42f82843ec5c957e4e06c414e7f9d0ade7cf052b`다. [실제 보정과 검증 source의 동일성](../assets/dependabot-20261109/source-review.md)을 별도로 확인했다.

전체 회귀 13,005건·Native 공식 4,077건·Studio 2,778건과 적용 lint·build·브라우저·편람 383쪽을 통과했다. Native/fresh WASM 전 9쪽씩의 raw 최저는 각각 100.00%이고 base/후보 픽셀은 모두 같다. [정식 검증·원 CI 원인·공급망 원장](../assets/dependabot-20261109/README.md)을 따른다.

| PR | 변경 | 원 head | 로컬 판정 |
| --- | --- | --- | --- |
| [#8811](pr_8811_review.md) | chore(deps): bump zip from 9.1.3 to 9.4.0 | `73f6b8c1f431dc36de0d4b17ab5ce2bf423be5ca` | 승인 |
| [#8812](pr_8812_review.md) | chore(deps-dev): bump vite from 8.3.5 to 8.3.6 in /rhwp-studio in the vite-stack group | `bb3d70c892bfffb5ee59d7570328ee86186ec5e4` | 승인 |
| [#8813](pr_8813_review.md) | chore(deps): bump softprops/action-gh-release from 3.0.3 to 3.0.4 | `5a98e6d56974481e831eb232f5c20631b48da087` | 승인 |
| [#8814](pr_8814_review.md) | chore(deps): bump log from 0.4.34 to 0.4.35 | `efd1a65979cfe42ced6649f7118d1d65dbecaad6` | 승인 |
| [#8815](pr_8815_review.md) | chore(deps-dev): bump vite from 8.3.5 to 8.3.6 in /rhwp-firefox | `384d609211f9caa0408d4912c1a1e8f23c872a07` | 승인 |
| [#8816](pr_8816_review.md) | chore(deps): bump @anthropic-ai/sdk from 1.3.0 to 1.6.0 in /rhwp-studio | `5d8b73a5f75482af8f1ee770f5ca698fcca1d00b` | 승인 |
| [#8817](pr_8817_review.md) | chore(deps): bump cfb from 0.15.0 to 0.16.0 | `2fb2264a8242539611a424e53c722651166301f6` | 승인 |
| [#8818](pr_8818_review.md) | chore(deps): bump github/codeql-action/analyze from 4.39.1 to 4.39.3 | `9b4ead9100f821bbcfe2331a3bf2916b96b575fb` | 메인터너 보정 후 수용 가능 |
| [#8819](pr_8819_review.md) | chore(deps): bump unicode-properties from 0.1.4 to 0.1.5 | `526ef018fa216d42808a24b6f522ac6bad4bfdbd` | 승인 |
| [#8820](pr_8820_review.md) | chore(deps): bump github/codeql-action/init from 4.39.1 to 4.39.3 | `c607ea643619fced34132144e07d43d4f9637c6e` | 메인터너 보정 후 수용 가능 |
| [#8821](pr_8821_review.md) | chore(deps): bump taiki-e/install-action from 2.88.1 to 2.90.2 | `6bf1774453978c45d954c5cd68a7977ae6f293fb` | 승인 |
| [#8822](pr_8822_review.md) | chore(deps): bump ml-kem from 0.3.3 to 0.3.4 | `4153bd5a09960fa69bf45e15d092b5b6861c985d` | 승인 |
| [#8823](pr_8823_review.md) | chore(deps): bump ml-dsa from 0.1.1 to 0.1.2 | `0de0b5e3ede6df2d257aaa62308521a25ea3aced` | 승인 |
| [#8824](pr_8824_review.md) | chore(deps): bump svgtypes from 0.16.1 to 0.16.2 | `7f26c047348a920fd2397a8b20c4edfaa0c8dd46` | 승인 |
| [#8825](pr_8825_review.md) | chore(deps): bump crc32fast from 1.5.3 to 1.5.4 | `a705ee565a3985b560db58167e70b5678285ddd6` | 승인 |
| [#8826](pr_8826_review.md) | chore(deps): bump ureq from 3.4.0 to 3.5.1 | `a55a1ed94a3c26b480418d3898c8dbd20b22ee6b` | 승인 |
| [#8827](pr_8827_review.md) | chore(deps): bump encoding_rs from 0.8.42 to 0.8.44 | `bb4981791b116b393e3f432c14cdf261304f7a69` | 승인 |
| [#8828](pr_8828_review.md) | chore(deps-dev): bump webpack-cli from 7.2.3 to 7.3.1 in /rhwp-vscode | `d65c82058e7caf6bc8e330785328563d88513475` | 승인 |
| [#8829](pr_8829_review.md) | chore(deps-dev): bump vite from 8.3.5 to 8.3.6 in /rhwp-chrome | `24ed2188056425b95b7c70b255c2e7d0ee4a447b` | 승인 |

CodeQL 두 원 head의 버전 불일치와 Native 4건의 30분 제한은 실제 로그·annotation으로 분류했다. 제품 코드·테스트 기대값·보안 탐지·검사 범위는 유지했다. 이번 갱신 범위의 서드파티 버전·라이선스 표기도 현행화했다.

원격 처리안은 단일 통합 branch·Open PR의 CI·self-review·merge, 원 PR 19건 완료 확인과 결과 댓글, 옵션 2의 별도 후속 기록 PR·devel 동기화·종료 정리다. 원격 변경은 승인 뒤 수행한다.

## 원격 처리 결과

[통합 PR #8840](https://github.com/edwardkim/rhwp/pull/8840)의 제출 head `91453509c9fb10d24bc8fb318b5437f2e80692fa`를 검증했고 [merge `153d5aede886077772349569e900fd770adddc93`](https://github.com/edwardkim/rhwp/commit/153d5aede886077772349569e900fd770adddc93)로 수용했다. 사용자 원격 승인 뒤 실행했으며 [self-review](https://github.com/edwardkim/rhwp/pull/8840#pullrequestreview-5822605316)를 공개했다. 원 기여 head 19개를 모두 보존했다.

- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/42573393012): success, attempt 1, head `91453509c9fb10d24bc8fb318b5437f2e80692fa`.
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/42573392969): success, attempt 1, head `91453509c9fb10d24bc8fb318b5437f2e80692fa`.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42573392806): success, attempt 1, head `91453509c9fb10d24bc8fb318b5437f2e80692fa`.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42573392794): success, attempt 1, head `91453509c9fb10d24bc8fb318b5437f2e80692fa`.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/42573392890): success, attempt 1, head `91453509c9fb10d24bc8fb318b5437f2e80692fa`.
- CI Impact Policy: success.

후속 기록은 CI 변경 PR의 옵션 2로 별도 review-only PR에 반영한다. 원 코드 PR에는 trailing commit·추가 CI를 만들지 않았다. 원 PR 19건 모두 MERGED임을 확인했고 결과 댓글과 종료 정리는 기록 병합 뒤 수행하며 최종 실행 결과는 해당 PR 댓글과 `target/pr-review/cleanup-pr8840-20261109.json`에서 확인한다. closing issue는 없다.
