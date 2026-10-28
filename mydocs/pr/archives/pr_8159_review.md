# PR #8159 리뷰 — fix(typeset): 글 host 의 양수 오프셋 자리차지 표를 밀린 저장 줄이 아니라 문단 원점에서 잰다 (#8156)

## 최종 판정

**통합 반영 완료 — 지정된 원 PR 범위의 로컬 검증·최종 CI를 통과한 뒤 PR #8228로 병합했습니다.**

통합 [PR #8228](https://github.com/edwardkim/rhwp/pull/8228)의 최종 head `b1e93d8de2d4842f725415aee19bd5b98ecfb8b3`가 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/40196953497)를 통과했고, 2026-10-25 22:58:32 KST에 merge `3e02fe2749866a8453af3eb3901b9c1a674cccaf`로 devel에 반영됐습니다. 원 contributor head와 실제 검증한 통합 head를 구분하여 기록합니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [8159](https://github.com/edwardkim/rhwp/pull/8159) / planet6897 / devel |
| 원 contributor head | `c5d49433d00bb46f2b88f30de257637c0b6d8c40` |
| 통합 검토 branch | `review/planet6897-20261024` |
| 검증 source / base | `673c1d5f7cdcb5573b793ba2d2abaaedd2602134` / `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a` |
| 현재 제출 후보 | `673c1d5f7cdcb5573b793ba2d2abaaedd2602134` / runtime `38ad63a903` |
| 적용 순서 | 현재 누적 history의 6번째 PR. 원 PR별 적용 SHA는 아래 표 참조 |
| 종료 지시 / 참조 | 원 본문 Closes ['8156'] / Refs []; 실제 종료 판정 전에는 실행하지 않음 |
| 원 PR 상태 조회 | 2026-10-25T13:01:34.007187+00:00 / OPEN / MERGEABLE |
| 원 PR reviewer | jangster77 |

## 변경과 검토 범위

양수 오프셋 자리차지 표의 원점을 밀린 저장 줄이 아닌 문단 소유 정보로 결정합니다.

### 원 기여와 메인터너 보정

#8157과 같은 4d06f563a6ba38c3d77cd8cbb81c02b9913f4c19는 중복 제외하고 고유 2개 기능 commit을 적용했습니다.

| 원 기능 commit | 통합 branch의 적용 commit |
| --- | --- |
| `630e8e9b5153f47dfe2ac907e1a195e17ec6e89d` | `2e326dbcf3000bd3f89361331b962223dccb27e9` |
| `c5d49433d00bb46f2b88f30de257637c0b6d8c40` | `172a71b80f9743e2bf58c5e1517b48f87cb64885` |

아래 적용 commit은 이 통합 branch의 실제 history입니다. 원 PR source branch는 변경하지 않았습니다.

### 검토할 실제 소비 경로와 회귀 원본

- `src/renderer/typeset/table/block/entry.rs`
- `tests/cases/issue_7687_empty_host_multi_float_gap.rs`
- `tests/cases/issue_8156_saved_host_offset_float_anchor.rs`

양수 오프셋 표는 밀려난 host 저장 글줄 대신 원 문단 원점을 기준으로 배치합니다. 앞 빈 host가 있는 K1과 없는 K4, T3/T4의 상대 오프셋과 host/후속 글줄을 대조합니다.

## 검증 입력과 결과

- code head `673c1d5f7cdcb5573b793ba2d2abaaedd2602134`, base `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`.
- 공통 통합 검증: 렌더러 집중 회귀 16/16 PASS, MCP 계약 4/4 PASS입니다. MCP 기존 실패는 제외한 sample16의 64쪽 기대값이 남은 오류였고 선택 fixture의 선언된 24쪽 기대값과 연결했습니다. 암호 stdin·비노출 검사는 유지했습니다.
- 최종 전체 회귀:      Summary [1920.075s] 11481 tests run: 11481 passed (46 slow), 45 skipped
- 필수 gate 상태 `complete`. [공통 검증 원장](../assets/integration-planet6897/final-validation-20261025/README.md).
- sample16 계열과 편람 HWP/HWPX는 사용자 지시로 제외하고 기존 검사 삭제를 유지합니다. 현재 판정 범위를 다른 #7445 항목으로 확대하지 않습니다.

### 전쪽 Native / fresh WASM TSV

| 조건 | Native 쪽수·최저 / TSV | fresh WASM 쪽수·최저 / TSV |
| --- | --- | --- |
| `saved-host` | 4쪽 / [100.00000%](../assets/integration-planet6897/final-validation-20261025/head/native/saved-host.tsv) | 4쪽 / [100.00000%](../assets/integration-planet6897/final-validation-20261025/head/wasm/saved-host.tsv) |

전체 페이지 90% 미만·누락 0쪽입니다. 같은 조건의 고정 base 대비 0.1%p 이상 하락은 0쪽이며, 쪽수 개선은 본문 소유·순서 보존을 따로 확인했습니다. [실제 입력·PDF SHA-256과 증적 바이트](../assets/integration-planet6897/final-validation-20261025/EVIDENCE.md).

## 변경 내용과 대응하는 증적

### `saved-host` · PDF p4 ↔ rhwp p4

- 입증 항목: 양수 오프셋 자리차지 표의 문단 원점.
- 실행 원본: `samples/issue8156/titled-host-hancom2020-saved.hwp`.
- 독립 Print PDF: `pdf/issue8156/titled-host-hancom2020-saved-2020.pdf`.

| 출력 | review | overlay |
| --- | --- | --- |
| Native | ![#8159 saved-host p4 Native review](../assets/integration-planet6897/final-validation-20261025/images/pr-8159/native/saved-host-p004-review.png) | ![#8159 saved-host p4 Native overlay](../assets/integration-planet6897/final-validation-20261025/images/pr-8159/native/saved-host-p004-overlay.png) |
| fresh WASM | ![#8159 saved-host p4 fresh WASM review](../assets/integration-planet6897/final-validation-20261025/images/pr-8159/wasm/saved-host-p004-review.png) | ![#8159 saved-host p4 fresh WASM overlay](../assets/integration-planet6897/final-validation-20261025/images/pr-8159/wasm/saved-host-p004-overlay.png) |

- 이 쪽 raw: Native 100.00000% / fresh WASM 100.00000%.

### `saved-host` · PDF p1 ↔ rhwp p1

- 입증 항목: 앞 빈 host와 양수 오프셋 표를 함께 둔 경우의 본문 원점.
- 실행 원본: `samples/issue8156/titled-host-hancom2020-saved.hwp`.
- 독립 Print PDF: `pdf/issue8156/titled-host-hancom2020-saved-2020.pdf`.

| 출력 | review | overlay |
| --- | --- | --- |
| Native | ![#8159 saved-host p1 Native review](../assets/integration-planet6897/final-validation-20261025/images/pr-8159/native/saved-host-p001-review.png) | ![#8159 saved-host p1 Native overlay](../assets/integration-planet6897/final-validation-20261025/images/pr-8159/native/saved-host-p001-overlay.png) |
| fresh WASM | ![#8159 saved-host p1 fresh WASM review](../assets/integration-planet6897/final-validation-20261025/images/pr-8159/wasm/saved-host-p001-review.png) | ![#8159 saved-host p1 fresh WASM overlay](../assets/integration-planet6897/final-validation-20261025/images/pr-8159/wasm/saved-host-p001-overlay.png) |

- 이 쪽 raw: Native 100.00000% / fresh WASM 100.00000%.

### 이 원 PR에 대응하는 정식 회귀

현재 전체 회귀 로그의 대응 PASS 1건입니다.

- `rhwp::regression_suite_024 issue_8156_saved_host_offset_float_anchor::saved_titled_host_measures_offset_float_from_paragraph_origin`

직접 판독: K1에서 앞 빈 host의 띠와 T4를, K4에서 앞 빈 host가 없는 T3/T4 상대 오프셋과 host/뒤 글줄을 확인했습니다.

## 원격 검토와 후속 처리

제출 자산 223개의 실제 raw 다운로드 바이트·SHA-256과 게시 본문 API·GitHub 화면의 review/overlay 표시를 확인했습니다. 최종 head의 CI·CodeQL·Render Diff·Proptest·Adapter·Gym 계약은 모두 통과했고 MERGEABLE·CLEAN 상태에서 사용자 승인으로 병합했습니다. 통합 이후 확정 기록은 사용자 지정 옵션 2의 별도 문서 PR로 보완합니다. 원 PR에는 이 통합 PR과 개별 검토 기록을 연결해 기여·보정·잔여를 설명한 뒤 통합 반영을 이유로 종료하며, contributor 원 브랜치는 변경하지 않습니다.
