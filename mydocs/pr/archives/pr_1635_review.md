# PR #1635 처리 보고서 — #1622 robustness 감사 + #1626 tolerance 조사 보고서 회수 (문서 전용)

- PR: https://github.com/edwardkim/rhwp/pull/1635
- 제목: `docs: Task #1622 robustness 감사 + #1626 tolerance 조사 보고서 (코드 무)`
- 작성자: planet6897 (collaborator)
- 연결: #1622, #1626 (둘 다 OPEN — 조사·감사 성격)
- base ← head: `devel` ← `planet6897:pr/devel-docs-1622-1626`
- 처리일: 2026-06-29

## 1. 처리 결정

**admin merge.** 코드 변경 없는 투자-only 보고서 2건이 PR 미생성으로 devel 누락된 것을 회수.
MERGEABLE/CLEAN + 충돌 0 + mydocs 전용 + 기존 파일 무중복.

## 2. devel 진전 상황 (선행 확인)

처리 착수 시 origin/devel 이 `f824a3c1`→`d3627f55` 로 진전돼 있었다. 작업지시자(Taesup Jang)
본인이 다른 세션에서 planet6897 PR 다수(#1618/#1620/#1624/#1627 통합 #1631/#1632, #1623
#1629/#1630)를 직접 통합한 것으로, 본 작업과의 분업이다(`project_pr_merge_collaborator` 정합).
`local/devel`(54112151)은 origin 의 조상이라 ff 로 동기화(d3627f55) — 내 이전 작업 손실 없음.

## 3. 변경 범위 (3 files +95/-0, src/tests/tools 0)

| 파일 | 내용 |
|---|---|
| `mydocs/report/task_m100_1622_report.md` | Robustness 전수 감사 — `C:/Users/planet/hwpdocs` 18k 코퍼스에 dump-pages(parse+pagination+layout) 18,647건 + export-svg(SVG 렌더) 18,732건 전수 스캔, **크래시 0건**. #1620 패닉(`36396650`) fix 후 전 경로 무패닉 확인 |
| `mydocs/plans/task_m100_1626.md` + `_report.md` | 잔여 +1쪽 조사 — universal last-line tolerance 재도입이 **단조 net-negative**(0px=75→3px=74→5px=73→8px=72→12px=71)임을 실험으로 확정. **#1608 의 tolerance 제거가 옳았고 재도입은 잘못된 방향**임을 데이터로 기록 |

세 파일 모두 devel 에 부재(중복/충돌 0). 신규 단일 커밋 `cbf20c50`.

## 4. 검증

| 항목 | 결과 |
|---|---|
| GitHub CI | preflight pass, build/test/CodeQL/Canvas는 코드 무변경이라 skip(정상) |
| 충돌 시뮬레이션 | 0건 (CLEAN) |
| src/tests/tools 변경 | 0건 (mydocs 전용 확정) |
| 자기검열 | 비교/최상급/공공기관 오인 표현 없음. 컨트리뷰터 로컬 코퍼스 기반 명시 |

## 5. 의의

- robustness: #1620 fix 후 18k 코퍼스 무패닉 — 후속 fix 대상 없음(회귀 감시는 스캔 스크립트 재사용).
- tolerance: #1608 의 tolerance 제거가 옳았음을 +1쪽 양방향 데이터로 재확인 — -1쪽 시리즈
  종결(#1617, 78.3%)과 정합. 재도입 유혹 차단용 영구 기록.
- #1622/#1626 은 조사·감사라 이슈 close 는 작업지시자 판단에 위임(보고서 회수가 목적).

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1635_review.md`
