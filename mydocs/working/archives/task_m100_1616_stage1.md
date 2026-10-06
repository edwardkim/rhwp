# Stage 1 완료보고서 — Task #1616 (footer 규칙 역공학 — 부정 결과)

**단계**: 규칙 역공학(조사, 코드 수정 없음) · **브랜치**: `local/task1616`

## 방법
통제셋(92건, 한글 정답 보유)의 footer 보유 문서 71건에서 rhwp 특징 추출
(footer_vpos·선언높이·footer_bottom·headroom·body_fill, `output/poc/task1616_footer_features.tsv`).
한글 push(footer 독립쪽) vs keep(1쪽 유지)를 분리하는 임계 탐색.

## 결과 — 분리 규칙 부재 (결정적 반례)

footer 가 page1 에 기하상 fit 하는 문서들에서 push/keep 이 **모든 footer 기하 특징에서 중첩**:

| 특징 | PUSH(한글 독립쪽) | KEEP(한글 유지) | 분리 |
|------|------|------|------|
| headroom (px) | 25.7 ~ 69.8 (page1-fit 중) | 45.2 ~ 326.7 | ✗ 중첩 |
| body_fill (%) | 32 ~ 72 | 32 ~ 58 | ✗ 중첩 |

**결정적 반례 (한글 PDF 직접 검증, COM 노이즈 배제)**:
| 문서 | footer_bottom | 선언높이 | headroom | 한글 |
|------|------|------|------|------|
| 36389575 | 927px | 357 | 63.3 | **PUSH(2쪽)** |
| 36393727 | 929px | 386 | 61.3 | **KEEP(1쪽)** |

footer 기하 거의 동일(bottom 927≈929)·PUSH 쪽 footer 가 **더 작음**(357<386)인데 판정 반대.
valign=Bottom 앵커 모델(footer_top=available−footer_h, body_end 과 비교)로도 두 문서 여유
margin 62~63px 동일·판정 반대.

검증된 한글 판정(PDF): 36389575=2쪽·36398767=2쪽·36392061=2쪽(PUSH), 36393727=1쪽·
36387735=1쪽(KEEP) — 통제셋 라벨 전부 정확.

## 결론

**footer 기하(위치·크기·headroom·body_fill·valign 앵커)로 push/keep 을 분리하는 규칙은 없다.**
한글의 결정 요인은 footer 기하 밖(본문 line-level fill / 폰트 메트릭 / 한글 내부 line-breaking
계산의 sub-px 차)에 있어 **stored IR 로 도출 불가**. 임계 기반 수정은 반례(36389575↔36393727)를
필연적으로 오분류하며, 전 코퍼스 footer 문서에 대량 회귀를 유발한다.

→ **코드 수정 없음.** razor-thin 8건은 **알려진 한계**로 확정·문서화. #1611 이 해소한 것은
footer 가 vpos 에서 **실제 overflow** 하는 surgical 케이스였고, 잔여 8건은 overflow 없는
한글 내부 휴리스틱이라 본 접근(기하 역공학)으로는 불가.

## 향후 (참고)
규칙 도출 가능성이 남는다면 footer 기하가 아닌 **본문 마지막 줄의 정확한 baseline/하단
위치 + 한글 line-fill 임계**를 per-line 으로 모델링해야 하나, 본문 per-line 이 저장 LINE_SEG
(한글 동일)인데도 판정이 갈리므로 한글 페이지-채움 알고리즘 자체의 재현이 필요(고난도).

산출물: `output/poc/task1616_footer_features.tsv`, `footer_rule_features.py`,
`counterexample_probe.py`(한글 PDF 검증).
