# Stage 1 보고서 (POC/타당성) — Task #1257: typeset vpos 통일

- 이슈: edwardkim/rhwp#1257 · 브랜치: `local/task1257`
- 방법: `RHWP_EN_VPOS_UNIFY=1` env 게이트로 `vpos_offset += extra_gap`(full 7mm) 프로토타입,
  전 미주 문서 측정. (조합 a=line_spacing 주입 유지 / b=주입 생략 둘 다 측정)

## 1. 측정 결과

| 항목 | 결과 |
|------|------|
| **페이지 수** (2022/2023/10월/미주사이20/구분선아래20/3-11) | **전부 불변** (typeset 페이지 분기 영향 없음) ✓ |
| #1256 케이스 문6→문7 | **287.0px 유지** (이중가산 없음) ✓ |
| 문5 (컬럼 하단) | **여전히 cram** — column-bottom cap 은 vpos 무관(y_offset 기반) ✗ |
| **3-11 render 오버플로우** | **+1752.9px** (p10 col1 pi=537 y=2845) — 치명적 desync ✗ |

조합 a/b 모두 3-11 오버플로우 동일 발생 → **이중가산이 원인 아님**.

## 2. 원인 (핵심 발견)

3-11 p10 단1 은 **POC 이전부터** `hwp_used≈1741.9px`(물리 단 ~1000px의 1.7배),
`diff=-806.1px` under-fill — **#1184 의 단일 미주 내부 비단조(2D) vpos** 가 만든 사전 결함.
POC 의 vpos 증가(+7mm×누적)가 여기에 **얹혀** render 절대-vpos→y 매핑을 off-page(y=2845)로
밀어냄(POC 시 hwp_used 1885px / diff -906px 로 악화).

→ **결론: typeset vpos 7mm 반영(A)은 현재 render 절대-vpos 모델 위에서 단독 적용 불가.**
render 가 컬럼/페이지 경계에서 vpos 를 일관 rebase 하지 못해(=#1184 미해결), vpos 증가가
누적 off-page 로 직결된다. 즉 A 의 전제(=render 가 vpos 를 정합 소비)가 현재 미충족.

추가로 문5 류 column-bottom cap 은 y_offset 기반이라 vpos 반영만으로는 미해결(별도 render 정리 필요).

## 3. 판단

POC 가 게이트 역할 수행: **A(typeset vpos 통일)의 선결 조건은 render 절대-vpos 컬럼 rebase/
밸런싱 재설계(#1184)** 이다. 이를 건너뛰고 vpos 만 올리면 광범위 render 오버플로우(특히 3-11
같은 조밀 문서). 페이지 수는 견디지만 **render 정합이 깨진다**.

## 4. 선택지 (작업지시자 결정 필요)

- **A′. #1184 render 재설계 우선** → 그 위에서 A(vpos 통일) 완성. 가장 정확하나 대형·다단계.
- **B. 점진 render 패치** (#1256 확장): 보고된 헤더(문5/26/29, 2023 문18/21, 10월 문7/8/11/28)를
  분기별로 정합 + 각 문서 오버플로우 회귀 검증. 부분적·취약(케이스마다 회귀 검증).
- **C. 보류**: #1256 부분 정합 유지, #1257 을 #1184 재설계 backlog 로 문서화.

## 5. 상태
- POC env-gate 코드는 **revert 완료**(working tree clean). 측정 데이터만 본 보고서에 보존.

---
승인/결정 요청: A′/B/C 중 어느 방향으로 진행할지 지시 바랍니다.
