# 최종 결과보고서 — Task #1248: render/pagination trailing 모델 통일 (조사)

- **이슈**: edwardkim/rhwp#1248
- **브랜치**: `local/task1248` (base `stream/devel`)
- **성격**: 조사·설계 전용 — **코드 0줄 수정**
- **조사 문서**: [`mydocs/tech/investigations/issue-1248/trailing_model_render_vs_pagination_1248.md`](../../tech/investigations/issue-1248/trailing_model_render_vs_pagination_1248.md)

> **현행화 메모 (2026-06-03)**: 본 보고서는 #1247/#1259 반영 전 조사 스냅샷을
> 보존한다. 현재 `devel`에는 관련 PR이 이미 반영되어, 문서의 "PR #1247 그대로 머지"
> 같은 권장 행동은 당시 기준의 결정 기록으로 읽어야 한다. 현재 `devel` 위에 PR #1260
> 문서 커밋을 체리픽한 재검증에서는 `cargo test --lib height_cursor`가 31 passed,
> `cargo test --test issue_1082_endnote_multicolumn_drift`가 4 passed로 통과했다.

## 1. 배경

#1246(PR #1247)에서 미주 between-notes min-gap 을 `vpos_adjust` 의 8번째 특례로 추가하며,
trailing 처리가 render/pagination 에 분산돼 누적 복잡도가 한계라는 가설로 시작.

## 2. 결론 (한 줄)

> **전면 통일 권장 안 함. "typeset base-flow 1984HU 가정 ↔ render 재구성" 의 좁은 정규화(A)만
> 가능·가치 있음. 양방향 vpos-인코딩 게이트(D/E)는 입력 데이터 모호성에 기인한 비가역 영역.**

## 3. 핵심 발견

1. **trailing 은 SSOT 부재** — typeset(굽기)·pagination(7분기 제외)·render(7특례 재구성)가
   각자 다른 가정 (조사문서 §1).
2. **불일치는 typeset 가정 ↔ render 재구성 사이** — base-flow 1984HU 가 "단일줄 prev=일치,
   다줄 prev=어긋남". 이것이 #1246 gap≈0 의 정확한 원인 (§3.2).
3. **pagination 은 typeset 편에 정렬** — PR #1247 이 별도 overflow 수정 없이 pi=475 해소된 이유 (§3.3).
4. **특례 8종 중 6종이 "미주 제목 앞 gap" 한 질문에 의존**, forward↔backward 대칭 (§2.3).
5. **전면 통일 불가 근거** — 메모리 `tech_lazy_base_trailing_ls_gate` + 양방향 데이터 오차.
   핀 고정 테스트 30+ 건이 현재 동작 고정 → 단일 규칙화 시 회귀 (§4.3).

## 4. 판정 (영역 구분)

| 영역 | 분류 |
|------|------|
| A. typeset base-flow 가정 ↔ render | **통일 가능 (권장)** |
| B. min-gap S8(#1247) | A 에 흡수 가능 |
| C. forward 캡 S2/S3 | 부분 정규화 여지 |
| D. backward 게이트 S5/S6/S7 | **게이트 필수 (통일 불가)** |
| E. invalid-lazy-base/rewind S0/S1 | **게이트 필수** |

## 5. 권장 행동

1. **PR #1247 그대로 머지** — 본 조사는 #1247 을 막지 않음. 검증된 실용 패치.
2. **후속 이슈(A 정규화) 신설** — 조사문서 §4.5 골격. **S8 흡수가 성공 지표.** 긴급도 낮은 기술부채.
3. **D/E 영역 불가침** — 향후 미주 버그도 이 영역은 "새 게이트 추가" 로 대응.

## 6. 산출물

| 파일 | 내용 |
|------|------|
| `mydocs/tech/investigations/issue-1248/trailing_model_render_vs_pagination_1248.md` | 조사 본문 §1~§4 + 검증 로그 |
| `mydocs/plans/task_m100_1248.md` / `_impl.md` | 수행·구현 계획서 |
| `mydocs/working/task_m100_1248_stage{1,2,3}.md` | 단계별 보고서 |
| 본 문서 | 최종 결과보고서 |

## 7. 검증

| 명령 | 결과 |
|------|------|
| `cargo test --lib height_cursor` | 26 passed |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 4 passed |
| 코드 변경 | 없음 (조사 전용) |

현재 `devel` 기준 PR #1260 체리픽 재검증:

| 명령 | 결과 |
|------|------|
| `git diff --check devel..HEAD` | 통과 |
| `cargo test --lib height_cursor` | 31 passed |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 4 passed |

## 8. 후속

- A 정규화 이슈는 작업지시자 판단으로 신설 여부 결정 (본 조사는 골격만 제안).
- 조사 결론은 메모리에 기록하여 향후 미주 작업의 기준점으로 활용.
