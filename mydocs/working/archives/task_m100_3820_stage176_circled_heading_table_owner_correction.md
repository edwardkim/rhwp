# Stage 176 - 동그라미 소제목과 RowBreak 표의 페이지 소유 보정

## 범위

#3820 브랜치를 `upstream/devel`에 rebase한 뒤에도 76076 기준 ledger에는 실제 소유권
불일치가 두 건 남아 있었다. 동그라미 소제목 `③ 대안의 선택 및 근거`가 rhwp의 p55와
p70 하단에 그려졌지만, 한컴은 소제목과 설명 표를 함께 다음 쪽에서 시작한다.

## 원본 구조

두 사례는 동일한 native HWP5 구조를 가진다.

1. 컨트롤이 없는 한 줄짜리 동그라미 소제목
2. 실제로는 비어 있는 carrier 문단 하나
3. 본문을 담은 비-TAC `TopAndBottom` 1x1 `RowBreak` 표 하나만 있는 호스트 문단

이 소제목에는 명시적인 `keep-with-next` 플래그가 없으므로, 일반적인 스타일 기반 규칙은
안전하지 않다. 따라서 보정은 이 구조에만 한정하고, 현재 쪽 꼬리에는 소제목만 들어가지만
소제목, carrier, 이미 확립된 최소 가시 표 조각까지는 들어가지 않을 때만 적용한다.

## 가드

- native HWP5와 단일 컬럼으로 한정한다. native HWP5는 보조적인 carrier `LINE_SEG`를
  생략할 수 있으므로, 그 부재를 반대 증거로 취급하지 않는다.
- 현재 페이지가 이미 가시 콘텐츠를 소유해야 한다.
- 의미상 비어 있는 carrier 문단이 정확히 하나이고, 표만 가진 호스트 문단이어야 한다.
- 양수로 선언된 표 높이, 셀 하나, 글자처럼 취급하지 않는 `TopAndBottom` `RowBreak`
  의미론을 요구한다.
- 묶음의 최소 단위가 새 페이지에 들어갈 수 있어야 한다.

이로써 일반 절 제목, 명시적 keep 속성, 다중 셀 표, synthetic carrier, 관련 없는 표
앵커는 기존 페이지 나눔 경로를 유지한다.

## 검증 목표

release-test 렌더러를 다시 빌드하고 #3820 integration suite 및 76076 텍스트 소유 ledger를
실행한다. p55->p56과 p70->p71의 조기 소유 항목이 사라져야 하며, p4/p18/p35의 RowBreak
회귀가 다시 발생해서는 안 된다.

## 검증 결과

- `issue_3820_rowbreak_rowspan_band`: 4 passed
- `issue_3820_body_top_table_border_clip`: 2 passed
- `issue_4490_4491_anchor_flow`: 2 passed
- `issue_4090_hwpx_tail_page_break`: 1 passed
- `76076-stage176-upstream-circled-owner-v2`: 기준 PDF, rhwp SVG, render tree가 모두
  82쪽이다. p55->p56과 p70->p71의 소유 후보는 없다.

남은 p6->p7과 p38->p39 소유 행은 보수적인 반복 문자 교집합 후보로 의도적으로 유지한다.
이전 PDF/SVG 시각 검토에서 각 경계 양쪽의 표와 소제목/본문 소유자가 일치함을 확인했다.
이는 페이지 경계 결함이 아니며, 이번 보정은 두 페이지 쌍 모두를 변경하지 않는다.
