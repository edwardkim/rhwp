# 새 base 영향 범위의 Native 실제 대조

후보 `c183a1ac61c2aa4188989fd93032cae68d3538c1`, 추가 기능 base `22947f6a90c998e95fa51238d5b907a0ebd37162`.

3입력·7쪽을 실제 실행해 0.1%p 초과 하락 0쪽을 확인했다. 원 PR #8498의 기존 기능은 리베이스 전 후보 4b와, 새로 반영한 기능은 실제 새 base 제품과 비교했다. 원본·Print가 같은 문서끼리 대조했다.

| 원 PR · 입력 · 독립 PDF | 비교 대상 source | 원 TSV · 후보 TSV | 실제 페이지별 before → after |
| --- | --- | --- | --- |
| [PR #8498](https://github.com/edwardkim/rhwp/pull/8498) · [156736331-press-logo.hwpx](<../../../../../samples/issue8491/156736331-press-logo.hwpx>) · [156736331-press-logo-2022.pdf](<../../../../../pdf/issue8491/156736331-press-logo-2022.pdf>) | `4b149b6cdf` | [원 TSV](<rebase-scope-native/reference/pr8498-156736331-press-logo-31768298.tsv>) / [후보 TSV](<rebase-scope-native/candidate/pr8498-156736331-press-logo-31768298.tsv>) | 1쪽 99.68048 → 99.68048%; 2쪽 99.94894 → 99.94894%; 3쪽 99.97134 → 99.97134%; 4쪽 99.92386 → 99.92386%; 5쪽 92.45232 → 92.45232% |
| [PR #8585](https://github.com/edwardkim/rhwp/pull/8585) · [textbox_tac_picture_rows.hwp](<../../../../../samples/issue7524/textbox_tac_picture_rows.hwp>) · [textbox_tac_picture_rows-webhwp-print.pdf](<../../../../../pdf/issue7524/textbox_tac_picture_rows-webhwp-print.pdf>) | `base-22947f6a90` | [원 TSV](<rebase-scope-native/reference/pr8585-textbox_tac_picture_rows-1d8665a4.tsv>) / [후보 TSV](<rebase-scope-native/candidate/pr8585-textbox_tac_picture_rows-1d8665a4.tsv>) | 1쪽 99.95891 → 99.95891% |
| [PR #8122](https://github.com/edwardkim/rhwp/pull/8122) · [textbox_valign_no_lineseg.hwp](<../../../../../samples/issue8122/textbox_valign_no_lineseg.hwp>) · [textbox_valign_no_lineseg-webhwp-print.pdf](<../../../../../pdf/issue8122/textbox_valign_no_lineseg-webhwp-print.pdf>) | `base-22947f6a90` | [원 TSV](<rebase-scope-native/reference/pr8122-textbox_valign_no_lineseg-db64ab49.tsv>) / [후보 TSV](<rebase-scope-native/candidate/pr8122-textbox_valign_no_lineseg-db64ab49.tsv>) | 1쪽 99.9275 → 99.9275% |

이 집중 대조는 80입력의 최신 base 전체 재출력이라고 표시하지 않는다. 기존 [리베이스 원점 증거](../rebase_scope_22947f6a90/README.md)와 구분한다. fresh WASM의 같은 범위도 별도 실제 대조가 필요하다.
