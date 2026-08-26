---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6075 review - 다단 미주 문단-사이 되감김 순차 적층 (#5886)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6075](https://github.com/edwardkim/rhwp/pull/6075) |
| 작성자 | [@kevin9327](https://github.com/kevin9327) |
| base | `devel` |
| 원 head | `d4bead470d741d16b79a49e5c35ac434c8189c51` |
| 규모 | +170 / -6, 4 files, 4 commits |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, Build & Test success (작성 시점 참고값) |
| 원 PR CI | [run 32888826754](https://github.com/edwardkim/rhwp/actions/runs/32888826754/job/97940894289) |
| 통합 적용 | `a25584ec3`, `618830bdf`, `41c3df5f5`, `21a690b4c` |

## 관련 이슈와 변경 범위

[#5886](https://github.com/edwardkim/rhwp/issues/5886)은 2단 본문이 용지 밖까지 그려져 줄이
소실되는 결함이다. `samples/3-09월_교육_통합_2022.hwpx`
(SHA-256 `b3ed3d5c4f0f95f9a8990b17720da3bef566b5a36174153aa3def28fd46e6029`) 12쪽에서
`[알짜 풀이]`·`ㄴ. [참]`·`ㄷ. [참]`이 y=1162~1290 (용지 1122.5)에 그려져 어느 쪽에도 남지 않는다.

변경은 `src/renderer/typeset.rs` 한 파일이다. HWPX 저장 레이아웃·다단·미주 흐름에서 문단-사이
compact 되감김이 나오면, 이후 문단을 렌더와 같은 순차 적층으로 시뮬해 단 하단을 읽고, 그 결과가
용지 밖으로 나가면 단/쪽을 전환한다. 누적 `acc`는 건드리지 않아 1139·1375 질문 흐름을 보존한다.

## 렌더 영향과 시각 검증 판정

`src/renderer/typeset.rs`의 pagination 경로가 바뀌고 PR이 특정 문서의 쪽·줄 소실 해소를 주장하므로
[intake_and_review 2.6](../../manual/pr_review/intake_and_review.md)의 **직접 증적 필수** 조합이다.
저장소에 한컴 기준 PDF `pdf/3-09월_교육_통합_2022.pdf`가 있어 통합 head 산출물로 대조 가능하다.

## 발견한 문제와 risk

1. **시뮬 비용이 기본 경로로 내려왔다.** `page_offcanvas_sim`이 참이면
   `simulate_endnote_column_bottom_y`가 scratch `LayoutEngine`으로 단 전체를 재렌더한다.
   종전에 이 경로는 `ssot_level >= EnSsotLevel::A3`(비기본) 전용이었다. 게다가
   `column_had_compact_endnote_rewind`는 단 전환까지 유지되므로, 되감김이 한 번 나오면 그 단의
   남은 미주 문단마다 전체 재렌더가 돌아 문단 수에 대해 제곱으로 늘어난다.
2. **가드 상수가 특정 문서 실측에서 역산됐다.** `ENDNOTE_PAGE_OFFCANVAS_GUARD_PX = 56.0`의 주석은
   "80px 는 663번 문단(69.7px)을 놓쳐"라고 근거를 밝히고 있어, 값 자체가 이 문서의 잔여
   +69.7px과 CI 회귀 사이에서 선택됐음을 인정한다. 다른 문서에서 이 임계가 맞는다는 근거는 없다.
3. **테스트 단언 하나가 약하다.** `tests/cases/issue_5886_column_offcanvas.rs`의
   `visible.contains("ㄴ")`은 두 쪽 전체 텍스트에서 자모 한 글자를 찾는 수준이라 사실상 항상 참이다.
   `max_y <= page_h + 24.0` 단언이 실질 게이트다.

1·2는 이번 통합에서 보정하지 않고 관찰로 남긴다. 성능은 공개 샘플로 전후 비교가 가능하므로
[CONTRIBUTING의 성능 검증 책임](../../../CONTRIBUTING.md#성능-검증-책임) 경계에 따라 메인터너가
통제된 환경에서 재확인할 대상이다.

## 검증 근거 (통합 head `136a94677`)

공통 게이트(전체 nextest·clippy·WASM·Native Skia·원장)는
[통합 구현 기록](pr_6075_6077_6079_6080_6084_review_impl.md#검증)에 한 번만 적었다.

- 이 PR의 회귀 `issue_5886_column_offcanvas`가 통합 head 전체 회귀에 포함돼 통과했다.
- 시각 검증: `rhwp export-svg "samples/3-09월_교육_통합_2022.hwpx" -p 11` 산출물에서 용지 높이는
  `1122.51`, 최대 text baseline은 `1141.56`으로 **`+19.05px`가 용지 밖에 남는다**. 원 PR이 보고한
  수정 전 값(~1290px, `+168px`)보다 크게 줄었지만 소실이 완전히 없어지지는 않았다.
  `ENDNOTE_PAGE_OFFCANVAS_GUARD_PX = 56.0` 임계 아래 잔여는 설계상 남는다.
- 한컴 기준 PDF `pdf/3-09월_교육_통합_2022.pdf`(23쪽)와 대조하면 rhwp의 쪽 구성이 오라클과
  어긋나 있다. 이는 이 문서의 선행 pagination 격차이며 이 PR이 만든 차이가 아니다. 따라서 이번
  판정은 **`#5886`이 지목한 용지 밖 소실의 감소**에 한정한다.

## 최종 권고

**수용.** 다만 아래 두 항목은 이 PR의 범위 밖 후속 대상으로 남긴다.

1. `+19.05px` 잔여와 `56.0px` 가드 임계의 근거. 임계 아래 소실은 여전히 인쇄에서 잘린다.
2. `page_offcanvas_sim`이 참인 동안 미주 문단마다 scratch `LayoutEngine` 전체 재렌더가 도는 비용.
   공개 샘플이 있으므로 통제된 환경에서 전후 비교로 확인할 수 있다.
