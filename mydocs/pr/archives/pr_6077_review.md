---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6077 review - 어울림 그림 아래 자리차지 표 겹침 해소 (#5929)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6077](https://github.com/edwardkim/rhwp/pull/6077) |
| 작성자 | [@kevin9327](https://github.com/kevin9327) |
| base | `devel` |
| 원 head | `ca1fb30d93a09d89b6d7c0c0df4f4086b96cec7d` |
| 규모 | +149 / -1, 5 files, 1 commit |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, Build & Test success (작성 시점 참고값) |
| 원 PR CI | [run 32871768397](https://github.com/edwardkim/rhwp/actions/runs/32871768397/job/97886722528) |
| 통합 적용 | `b845acd8d` |

## 관련 이슈와 변경 범위

[#5929](https://github.com/edwardkim/rhwp/issues/5929)는 표가 이미지와 겹치는 결함이다.
이슈 첨부 HWPX를 `samples/issue5929/table_below_square_pic.hwpx`
(SHA-256 `1b4727d65fcd85bea3fa4e299b32acd6577a612458ca945be582476957c5fd03`)로 보존했다.
그림은 `textWrap=SQUARE`·`treatAsChar=0`·`allowOverlap=0`·`vertRelTo=PARA`이고 뒤따르는 표는
`TOP_AND_BOTTOM`이다.

`src/renderer/layout.rs`에서 `VisibleFloatExclusion`에 `blocks_text` 필드를 추가하고, 어울림 그림의
페인트 bbox를 `blocks_text=false` 밴드로 남긴다. 본문 텍스트는 종전대로 그림 옆을 흐르고, 후속
자리차지(T&B) 표만 밴드 아래로 밀린다.

## 렌더 영향과 시각 검증 판정

`src/renderer/layout.rs`의 float 배치 경로가 바뀌고 PR이 그림·표 겹침 해소를 주장하며 신규 HWPX
fixture가 붙어 있으므로 **직접 증적 필수** 조합이다. 이 fixture에는 한컴 기준 PDF가 없다.
[visual_fixture_evidence 3.5.1](../../manual/pr_review/visual_fixture_evidence.md)에 따르면 원본
HWPX가 있으므로 `rhwp info --json`의 저장 버전에 맞는 MCP로 기준 PDF를 산출하는 것이 정석이나,
이 검토 환경에는 Windows MCP 접근이 없다. 대신 통합 head 산출물의 기하(그림 bbox 하단 대비 표 상단)를
직접 확인한다.

## 발견한 문제와 risk

구조적 결함은 찾지 못했다. 확인한 항목은 다음과 같다.

- `visible_float_exclusions`는 단(column) 빌드 지역 변수라 쪽·단 사이로 누수하지 않는다.
- `blocks_text` 소비부 세 곳(단일 줄 전방 스냅 프로브, 문단 밴드 소비 루프, 표 floor 계산)이 모두
  갱신됐고, 기존 자리차지 표 밴드는 `blocks_text=true`로 종전 동작을 유지한다.
- `find_painted_control_bbox`는 자식을 역순 탐색해 마지막 페인트 노드를 고르며, 높이 0 노드를 배제한다.
- 회귀 테스트가 SVG의 `<image>` bbox와 셀 `clipPath` y를 직접 읽어 겹침을 판정한다. 결함 위치
  (표 y≈594.4)를 별도 단언으로 못박아 통과 조건이 느슨해지지 않는다.

적용 범위가 `Control::Picture`로 한정돼 도형(Shape)의 Square 어울림에는 같은 보정이 없다. 문서
주석이 그 범위를 명시하므로 이번 PR의 결함은 아니며, 같은 형상이 도형에서 재현되면 별도 이슈로
분리할 대상이다.

## 검증 근거 (통합 head `136a94677`)

- 이 PR의 회귀 `issue_5929_table_below_square_pic`가 통합 head 전체 회귀에 포함돼 통과했다.
- 시각 검증: `rhwp export-png samples/issue5929/table_below_square_pic.hwpx -p 0` 산출물에서 표가
  그림 아래로 완전히 내려가 겹침이 사라진 것을 직접 확인했다
  (`mydocs/pr/assets/pr_6077_table_below_square_pic_after.png`).
- 기하 실측: 그림 페인트 bbox는 `y=377.87`, `height=250.00`으로 bottom `627.87`이고, 표 상단
  괘선도 `627.87`이다. 즉 **간격이 정확히 `0.00px`**다.

## 발견한 문제 — 바깥 위 여백이 복원되지 않는다 (기록)

`rhwp dump`로 확인한 이 표의 바깥 여백은 `top=1.0mm(283 HU)` ≈ `3.8px`다. 그런데 코드 주석
"[#5929] 어울림 그림 아래 자리차지 표는 바깥 위 여백을 유지한다(한컴: 표가 그림 바닥에 붙지 않음)"이
주장하는 복원이 이 fixture에서는 관측되지 않는다. `visible_outer_top_px.max(v_off.max(0.0))` 복원
분기가 이 표에서 실제로 평가되는지 원 저자 확인이 필요하다.

이 fixture에는 한컴 기준 PDF가 없어 정답 간격을 확정할 수 없고, 이슈가 지목한 겹침 자체는
해소됐으므로 차단 사유로 보지 않는다.

## 최종 권고

**수용.** 위 여백 항목은 원 저자 확인 대상으로 PR comment에 남긴다.
