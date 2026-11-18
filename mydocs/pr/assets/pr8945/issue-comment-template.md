[PR #8945](https://github.com/edwardkim/rhwp/pull/8945)를 [merge `4ab860651b`](https://github.com/edwardkim/rhwp/commit/4ab860651b857bfe6d28b22b4565b2855bebb042)로 devel에 반영했습니다.

첫 조각에 실을 첫 행과 바깥 아래 여백이 함께 들어가지 않는 RowBreak 자리 차지 표에서, 표만 다음 쪽으로 옮기고 호스트/뒤 본문의 흐름을 보존하는 부분 개선입니다. 첫 행이 들어가면 기존 행 경계 분할을 유지합니다.

새 3개 입력과 기존 대조군 총 8쪽은 Native/fresh 최적화 WASM 각각 raw 100.00%, 최소 글꼴 집중 회귀 14건과 필수 편람 gate를 통과했습니다. [검토 기록](https://github.com/edwardkim/rhwp/blob/{RECORD_SHA}/mydocs/pr/archives/pr_8945_review.md)에 전쪽 TSV·입력/기준·직접 비교를 보존했습니다.

이 이슈는 **OPEN으로 유지**합니다. 앞선 [#7644](https://github.com/edwardkim/rhwp/pull/7644)/[#7681](https://github.com/edwardkim/rhwp/pull/7681) 이후 남은 footer·CellBreak 등 전체 잔여 범위를 이번 PR이 해결했다고 표시하지 않습니다.
