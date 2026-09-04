# 최종 결과보고서 — Task #1616 (footer 독립-쪽 규칙 역공학 — 부정 결과)

**제목**: 발신명의 footer(Page+Bottom) 독립-쪽 배치 규칙 역공학
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1616 · **브랜치**: `local/task1616` (base: `local/task1612`)

## 1. 목표
잔여 −1쪽 8건(razor-thin): footer 가 기하상 page1 에 맞는데 한글이 page2 에 footer 단독 배치.
통제셋(한글 정답 보유)의 footer 문서 특징을 추출해 push/keep 임계 규칙을 역공학.

## 2. 결과 — **규칙 부재 확정 (데이터 기반)**

footer 보유 71건 특징 분석 + 한글 PDF 검증 결과, **footer 기하로 push/keep 을 분리하는
규칙은 존재하지 않음**:

- 결정적 반례(한글 PDF 검증): **36389575**(bottom 927, 선언 357) → **PUSH(2쪽)** vs
  **36393727**(bottom 929, 선언 386) → **KEEP(1쪽)**. 기하 거의 동일·PUSH 쪽 footer 가 더
  작은데 판정 반대. headroom·body_fill·valign 앵커 margin 모두 중첩(분리 실패).
- 한글 결정 요인은 footer 기하 밖(본문 line-fill / 폰트 메트릭 / 한글 내부 계산)에 있어
  stored IR 로 도출 불가.

## 3. 판정 (코드 수정 없음)

razor-thin 8건은 **알려진 한계**로 확정. 임계 기반 수정은 반례를 필연 오분류하고 전 코퍼스
footer 문서(거의 모든 정부 결재문서)에 대량 +1 회귀를 유발하므로 **수정하지 않음**. 이는
추측 회피·회귀 방지의 책임 있는 판단.

#1611 이 해소한 케이스는 footer 가 stored vpos 에서 **실제 overflow**(vpos+선언 > body)하는
surgical 케이스. 잔여 8건은 overflow 가 없어(geometry fit) 기하 역공학 범위 밖.

## 4. −1쪽 갭 시리즈 최종 정리

| 태스크 | 내용 | 통제셋 일치 |
|--------|------|------|
| #1600 | 다요인 조사·통제셋/게이트 자산 | 60 |
| #1608 | 요인 A: is_hwp3_origin 오탐지 제거 | 66 (+6) |
| #1611 | 요인 B: footer overflow page-fit | 72 (+6) |
| #1612 | 진단 메트릭(hwp_used) 정정 | 72 (불변) |
| #1616 | 잔여 8건 규칙 역공학 → **규칙 부재 확정** | 72 (수정 없음) |

**최종 통제셋 일치 72/92 (78.3%)**, +12(60→72). 잔여 −1쪽 12건(footer 8 + 다페이지 4)은
한글 내부 페이지-채움 알고리즘 재현이 필요한 하드코어로, 현 IR/기하 기반 접근의 한계.

## 5. 산출물
- 분석: `output/poc/task1616_footer_features.tsv`, `footer_rule_features.py`, `counterexample_probe.py`
- 문서: `_stage1`(상세), 본 보고서, `mydocs/tech/investigations/issue-1600/render_minus1_page_gap.md` 갱신
- 코드 변경: **없음**
