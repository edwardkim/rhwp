---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6084 review - CELL 분할 평가표의 빈 밴드 해소 (#6035)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6084](https://github.com/edwardkim/rhwp/pull/6084) |
| 작성자 | [@kevin9327](https://github.com/kevin9327) |
| base | `devel` |
| 원 head | `0146e8d66922422a609a5bb0520a306efb798479` |
| 규모 | +83 / -3, 7 files, 4 commits |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, Build & Test success (작성 시점 참고값) |
| 원 PR CI | [run 32892124164](https://github.com/edwardkim/rhwp/actions/runs/32892124164/job/97951570014) |
| 통합 적용 | `fc9e12fee`, `c221f2c18`, `dcb15f638`, `5ba7bc27e` |

## 관련 이슈와 변경 범위

[#6035](https://github.com/edwardkim/rhwp/issues/6035)는 `pageBreak="CELL"` 표의 행을 셀 안에서
나누지 않고 통째로 밀어 쪽 하단을 비우는 결함이다. 식약처 고시 별표 2(147행 3열)를
`samples/issue6035/cgmp_evaluation_table.hwpx`
(SHA-256 `0f03119d6ac3e19f74fc9fcec5315ac6ed436bab5abffd0804e9cf3350dd747a`)로 보존했다.

`src/renderer/layout/table_layout.rs`의 `direct_hwpx_cell_has_declared_stored_frame`에서
`reset_count == 1`일 때의 판정을 세로 정렬별로 나눈다. CENTER 셀은 리셋 앞 조각
(`preceding_frame_end`)이 선언 높이의 4/5~1.0에 들고 뒤 조각이 선언 높이 안일 때만 물리 프레임
소유로 본다. TOP 셀은 종전 합(`sum`) 계약을 유지한다.

## 렌더 영향과 시각 검증 판정

표 분할·쪽 배치 경로가 바뀌고 신규 HWPX fixture와 쪽 배치 개선 주장이 함께 있으므로
**직접 증적 필수** 조합이다. 이 fixture에는 한컴 기준 PDF가 없고 검토 환경에 Windows MCP 접근이
없다. 통합 head 산출물에서 PR이 주장한 쪽 구성(헤더 행과 하위 항목이 같은 쪽)을 직접 확인한다.

## 발견한 문제와 risk

**CENTER 분기가 좁히기가 아니라 조건 교체다.** 기존 판정은 `preceding + trailing ∈ [0.8h, h]`인데
새 CENTER 판정은 `preceding ∈ [0.8h, h] && trailing ∈ (0, h]`로 **대체**된다. 두 조각이 모두 큰
형상, 예를 들어 `preceding = 0.9h`, `trailing = 0.9h`이면 합이 `1.8h`라 종전 판정은 거짓인데 새
판정은 참이 된다. 즉 일부 CENTER 셀에서는 판정이 오히려 넓어진다.

#6035 해소에 필요한 방향은 좁히기이므로, 교집합
(`sum_fits_declared && preceding_frame_end >= …`) 형태가 의도에 더 맞는다. 다만 이 형상이 저장소
코퍼스에 실재하는지는 확인되지 않았고 원 PR CI와 통합 회귀가 모두 통과하므로, 이번 통합에서는
보정하지 않고 원 PR 저자 확인 대상으로 기록한다.

회귀 테스트는 헤더 행과 하위 항목이 같은 쪽에 있는지, 헤더만 남은 유령 쪽이 없는지를 페이지
텍스트로 판정한다. 결함 상태를 직접 재현하는 형태라 통과 조건이 느슨하지 않다.

## 검증 근거 (통합 head `136a94677`)

- 이 PR의 회귀 `issue_6035_cell_split_empty_band` 2건이 통합 head 전체 회귀에 포함돼 통과했다.
- 신규 fixture이므로 [local_validation 4.3.1](../../manual/pr_review/local_validation.md#431-새-hwphwpx-fixture의-baseline-등록--ir-sweep--overflow-cell-원장)의
  두 원장을 통합 head에서 재산출했다. IR field sweep은 baseline과 정렬 diff 차이가 없었고,
  overflow-cell 원장에도 이 fixture는 행을 만들지 않는다(쪽 밖 소실 줄 0).
- 시각 검증: `rhwp export-png samples/issue6035/cgmp_evaluation_table.hwpx` 36쪽에서
  `다. 물 공급 설비는 다음을 만족하는가?`와 하위 `1) 2) 3)`이 같은 쪽에 채워지고 빈 밴드·헤더만
  남은 유령 쪽이 없음을 직접 확인했다 (`mydocs/pr/assets/pr_6084_cgmp_p36_after.png`).

## 최종 권고

**수용.** 다만 CENTER 분기가 기존 `sum` 계약을 대체해 일부 형상에서 판정이 넓어지는 점은 원 저자
확인 대상으로 PR comment에 남긴다.
