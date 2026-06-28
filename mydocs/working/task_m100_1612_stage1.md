# Stage 1 완료보고서 — Task #1612 (조사)

**단계**: 잔여 −1쪽 특성화 + 메트릭 혼동 확정 · **브랜치**: `local/task1612`

## 1. dump-pages `hwp_used` 메트릭 다페이지 혼동 (진단 버그)

`compute_hwp_used_height`(rendering.rs:3700)는 마지막 항목의 **stored vpos 기반 누적 위치**를
반환. vpos 는 다페이지에서 누적값:
```
36398709 page별 첫/끝 vpos(px): p1 0→872, p2 888→1701, p3 1735→2471, p4 2501→3273, p5 3305→3931
```
→ per-page `used` 와 비교 시 `diff` 가 페이지마다 ~800px 누적 증가(−1.0→−815.9→...→−3300.9)해
**"대형 tac 표 과소측정"으로 오판**. 페이지 시작 vpos 차감 시 정상 per-page 높이(872/814/736/
771/626px) 도출 → base 차감으로 정정 가능.

## 2. 잔여 12건 실제 특성 (post-#1611)

| 군 | 건수 | 특성 |
|----|------|------|
| 단일페이지 footer | 8 | used 959~985px < body 990.2, footer(Page+Bottom) vpos 에서 fit, 한글 2쪽. 마지막 item=Table → 현재 메트릭 None |
| 다페이지 | 4 | 메트릭 혼동으로 −1857~−5457px 과대 표시(실제 per-page 차 수십~166px) |

footer declared/vpos 점검(36390093/36394966/36392061): footer 가 stored vpos+선언높이에서
body 안에 fit(예: 36390093 vpos 607.3 + 357.2 = 964.5 < 990.2) → #1611 이 안 미는 게 정상.
8건의 −1 은 본문 누적 ~20~43px 부족 = **inter-paragraph gap/spacing 미세차**(본문 per-line 은
저장 LINE_SEG 라 한글 동일). Task #1600 하드코어 — **단일 surgical fix 부재 가능성 높음**.

## 3. 결론

- **"대형 tac 표 과소측정" 은 메트릭 아티팩트**(실제 과소측정 아님).
- 잔여 −1 은 razor-thin 본문 누적 갭(8건 ~20~43px) — 고위험·저마진, net>0 보장 어려움.
- **본 태스크 수정 범위: 진단 메트릭 정정**(저위험, 페이지수 불변). razor-thin 본문 갭은
  특성화·보존(코드 수정 안 함).
