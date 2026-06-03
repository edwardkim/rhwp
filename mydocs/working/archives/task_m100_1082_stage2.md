# Stage 2~3 보고서 — Task #1082: 다단 드리프트 수정 시도 → 실패, 재진단

- 브랜치: `local/task1082` (시도 후 전량 revert, 소스 clean)

## Stage 1 재확인 (engine.rs 정상, typeset.rs 드리프트)
- 5 파일 모두 engine.rs(RHWP_USE_PAGINATOR=1)=0 overflow, typeset.rs 만 드리프트.
- typeset.rs 20페이지 vs engine.rs 11페이지 — 두 엔진이 본 문서를 **근본적으로 다르게**
  페이지네이션. typeset 은 페이지를 더 많이 만드는데도 다단 페이지 col 1 에서 overflow.
- 문서: 1 섹션, IR 488 문단(`dump` 기준)인데 **page item / LAYOUT_OVERFLOW 의 para_index 는
  818~882** — 렌더 문단 수가 IR(488)보다 많음(합성/분할 추정). pi 인덱싱 불투명이 정밀
  per-문단 분석을 차단.

## Stage 3 시도 — 다단 누적 정책 정합 (실패)
가설: typeset 다단이 `height_for_fit`(=total−trailing_ls) 누적(L1901/1949/1969) → 문단마다
trailing_ls 과소 → col 과충전. engine.rs 는 `current_height += para_height`(full) 누적 + fit
판정만 trailing 관대(L946-964).

→ typeset 다단 누적을 `total_height` 로 변경(engine.rs 정합) 후 측정:
- C군 5파일 max overflow **불변**(626.9/277/158.5/**579.5↑**/626.9). 3-11 은 오히려 악화.
- **수정 무효** → C군 드리프트는 이 누적 경로가 아님. revert.

## 재진단 (남은 미스터리)
- col 1 content: x=670~738(우측 단 정상)인데 y=1711(드리프트). typeset col-1 used=929(<body
  1001, fit 판정)인데 렌더는 1621px(문단당 ~180px) → "빈 문단"이 실제로는 키 큰 줄이거나
  렌더가 다른 높이 사용. typeset↔렌더 측정 공간 불일치이나 **단일 누적점이 아님**.
- C군 다단 문단은 `typeset_multicolumn_paragraph`(단일 문단 내 col_break)도, 표준 fit/place 도
  아닌 조합으로 보이며, pi 인덱싱 불투명으로 정확한 처리 경로 미특정.

## 결론
- engine.rs 가 정답(oracle)이라는 점은 확정. 그러나 typeset.rs 의 정확한 divergence 점을
  **광범위 시도에도 미특정** — pi 인덱싱(IR 488 vs 렌더 882) + 다단/2-pass 처리 복잡도.
- 누적-정책 수정은 실증적으로 실패. C군 은 typeset.rs 다단 페이지네이션을 engine.rs 와
  line-by-line 대조하는 **전담 심화 과제**가 필요(단일 세션 내 안전 수정 범위 초과).
- 잘못된/회귀 유발 수정 강행 대신 정직히 보류 유지. 완료 4건(#1068/#1070/#1073/#1079) PR
  (#1069/#1083/#1084, #1070=cherry-pick 0facb1b4)이 세션 성과.

## 권고
C군 = "typeset.rs 다단 페이지네이션 ↔ engine.rs 대조" 전담 과제로 분리. 착수 시: (1) pi 인덱싱
정합(IR vs 렌더 문단 매핑) 먼저 해소, (2) 두 엔진의 섹션-1 다단 컬럼 assign/flush 단계별 dump
비교, (3) 차이점 1개씩 typeset 정합 + 전수 sweep.
