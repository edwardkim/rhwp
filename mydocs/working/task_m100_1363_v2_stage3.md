# Stage 3 (v2) — fit/split SSOT 정합 시도 (음성 결과: 점진 불가, 홀리스틱 재작성 필요)

Stage 2 가 누적을 sim 정합화했으나 fit/split 결정은 옛 모델 사용 → 불일치(p17 단1 overflow,
pi=1127 미이동). Stage 3 은 fit/split 도 시뮬 기반으로 전환을 시도했다. **결론: 점진적
sim-통합 불가 — break 루프 전면 재작성 필요.**

## 1. 시도
A2 경로 내에서:
1. `simulate_endnote_column_bottom_y` 에 `extra_para_full` 인자 추가 — 현재 단 + 새 para(full)
   시뮬 bottom 으로 fit 판정.
2. `split_endnote_to_fit` 게이트의 `current_height + en_fit > available` 를 **시뮬 overflow**
   (`a2_overflow_with_para`)로 대체.
3. split 불가(단일줄) para 가 단 overflow 시 **fit-or-advance**(시뮬 overflow → advance_column).

## 2. 측정 — 악화
| 대상 | Stage 2(A2) | Stage 3 시도(A2) |
|------|------------|------------------|
| p17 pi=894 (C×C 우단) | 단1 이동 ✓(단 overflow) | 변화 없음(단1 1087.9 유지) |
| p21 pi=1127 (p21 유지) | p22 미이동 | p22 미이동 |
| **전 페이지 overflow** | p17 단1만 | **p17 43.6 + p22 121.2 (신규 악화)** |

## 3. 근본 원인 — break 루프는 점진 통합 불가
- 미주 단-break 은 **단일 "fit-or-advance" 루프가 아니라 ~10개 상호작용 휴리스틱 게이트**
  (`current_height > available*0.85/0.88/0.90`, compact-profile 특수분기, local/internal
  rewind 처리, `advance_large_between_single_line_rewind` 등)로 구성.
- 일부 결정만 sim 권위로 바꾸면 **나머지 휴리스틱과 충돌** → 이중 advance/신규 overflow.
- p17/p21 overflow 는 **단일줄 para 의 누적 초과**(각자는 fit, 합쳐서 over)인데 현 모델엔
  이 케이스의 깨끗한 break point 가 없음(split 은 다줄 전용, 휴리스틱 임계는 sim 과 불일치).

## 4. 결론
fit/split SSOT 는 **break 루프 전체를 시뮬 구동 단일 fit-or-advance 로 홀리스틱 재작성**해야
가능. ~10개 휴리스틱 게이트를 **동시에** 시뮬 권위로 치환 + 전 exam 재튜닝. 점진 게이트
교체로는 불일치만 누적(본 Stage 실증). → 별도 대형 작업(Stage 3 재설계).

코드: Stage 3 시도 전량 revert(typeset.rs Stage 2 상태 복귀). A2 는 누적 정합까지만 유지.

## 5. Task #1363 v2 종합 현황
| 단계 | 성과 | 상태 |
|------|------|------|
| Stage 1 | vpos_adjust 입력 분해, 렌더-시뮬 경로 확정 | ✅ |
| Stage 2 | 누적 SSOT(A2 시뮬 스냅) — C×C 우단 이동 입증, 회귀 3건 격리 | ✅ |
| Stage 3 | fit/split 점진 통합 — **불가(악화), 홀리스틱 재작성 필요** | ❌ 음성 |

**판단**: 후보 A 의 누적 절반(Stage 2)은 성립·안전(A2 opt-in, 기본 B 무회귀). break 절반은
홀리스틱 재작성이 필요한 대형 잔여. 기본 B(PR #1368)는 변함없이 #1357 부분 개선 유지.

## 6. 권고
- A2 누적 인프라(Stage 2)는 향후 break 재작성의 토대로 보존(커밋됨).
- break 루프 홀리스틱 재작성은 별도 스코핑·대형 착수 — 본 세션 범위 초과. 작업지시자 판단 요청.
