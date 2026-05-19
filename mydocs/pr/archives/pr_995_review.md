# PR #995 검토 — HWP5 multi-TAC paragraph z-order 정합 (composer marker synthesis)

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #995 |
| 제목 | fix: HWP5 multi-TAC paragraph z-order 정합 — composer marker synthesis (closes #991) |
| 작성자 | jangster77 (Taesup Jang) — 기존 컨트리뷰터 (PR #989/#969 등 sample16 계열 연속) |
| base ← head | `devel` ← `jangster77:local/task991-fix` |
| 연결 이슈 | `closes #991` (⚠️ #991 이미 CLOSED — 2.1 참조) |
| mergeable | MERGEABLE |
| CI | Build & Test ✅ / Analyze rust·js·py ✅ / Canvas visual diff ✅ / CodeQL ✅ / WASM skip |
| 변경 | 8 파일 +717 / -0 — 소스 1 (`composer.rs` +130), 문서 7 |
| 생성 | 2026-05-18 10:52 |

## 2. 배경

### 2.1 이슈 #991 상태 — 검토 시 반드시 확인 필요 ⚠️

- 이슈 #991 은 **2026-05-18 09:00:42 에 jangster77 본인이 `COMPLETED` 로 close** (PR 생성 1h52m 전).
- close 시점 timeline 에 연결 commit 없음 (`commit_id: null`) — 코드 머지 없이 닫힌 상태.
- PR #995 의 Stage 1 보고서는 "이전 close + 작업지시자 추가 시각 검증으로 재오픈" 이라 기술하나, GitHub 상 #991 은 **현재 CLOSED 유지** (reopen 이벤트 timeline 미관찰).
- 메모리 룰 `feedback_close_issue_verify_merged` 정면 관련: 정정 commit 이 머지 안 된 채 이슈가 닫히면 동일 결함 재발 위험.
- **판단 보류 항목**: 작업지시자 확인 필요 — (a) #991 을 reopen 후 본 PR 로 close 할지, (b) 이미 close 된 이슈를 PR body 의 `closes #991` 로 둘지(merge 시 no-op), (c) #991 의 close 가 본 작업과 별개 부모(#942/#988) 정리였는지.

### 2.2 기술 배경 (이슈 #991 root cause)

`samples/hwp3-sample16-hwp5.hwp` (한컴 2022 가 HWP3→HWP5 변환) 페이지 18:
다이어그램이 페이지 우측 경계 overflow + "가."/"나." 라벨 z-order 어긋남.

PR 이 지목하는 root cause (이슈 #991 의 anchor 분석과는 다른 각도):

| | HWP3 parser | HWP5 parser |
|---|---|---|
| 확장 ctrl 의 `\u{FFFC}` 마커 | 컨트롤마다 1개 push | **미푸시** |
| char_offsets entry | 마커마다 1 | 컨트롤 skip → sparse |

→ HWP5 의 sparse text + char_offsets 로 composer 가 line 별 text 범위
계산 시 `utf16_range_to_text_range(0, 8)` 가 빈 range 반환 → ls[0]
(가. 라벨) 빈 라인 처리 → z-order 어긋남.

> 참고: 이슈 #991 본문은 root cause 를 "표/그림 anchor 가 PAPER→PARA 로 변경" + "문단 margin 2x" 로 분석. 본 PR 은 "HWP5 parser 의 inline marker 누락" 으로 분석. 두 분석이 같은 증상의 다른 층위인지, 본 fix 가 anchor/margin 문제는 미해결로 남기는지 검토 항목 (3.4).

## 3. 검토 항목

### 3.1 변경 내용

`src/renderer/composer.rs`:
- `synthesize_marker_paragraph(para) -> Option<Paragraph>` 신규 (+~120)
- `compose_paragraph` 진입부에서 호출 → synth 성공 시 shadow 후 기존 로직

핵심: HWP5 의 누락 마커를 composer **내부에서만** 합성. parser/IR
원본 미변경 → editor pipeline (insert_text/save/cursor) 영향 없음.

좁힘 3 조건 (모두 만족 시만 합성):
1. `inline_ctrl_count >= 3` (pi=394 = 3 TAC)
2. `n_leading = char_offsets[0] / 8 >= 2`
3. `existing_markers < inline_ctrl_count` (HWP3 자동 차단)

### 3.2 설계 적합성 — 메모리 룰 정합 ✅

- `feedback_hancom_compat_specific_over_general`: 일반화보다 케이스별
  구조 가드 우선. 본 PR 의 3중 좁힘 가드는 pi=394 패턴만 catch —
  룰 정합. F1(광범위)/F2-wide 가 cargo test 5~9 fail 인 반면
  F2-narrow 만 0 fail 인 점이 좁힘의 타당성 뒷받침.
- parser 미변경 → editor 무영향 설계는 회귀 표면 최소화. 합리적.
- composer-only 격리 → HWP3/HWPX path 미오염.

### 3.3 코드 품질 — 지적 사항 (수정 요청 후보)

**(a) `first_off`/`n_leading` 중복 계산.**
좁힘 가드에서 한 번 (`let first_off = para.char_offsets.first()...; let n_leading = first_off / 8;`) 계산 후, 합성 본문에서 `let first_off = offsets.first()...; let n_leading = first_off / 8;` **재계산** (shadowing). 동일 값. 죽은 코드는 아니나 불필요 재계산 + 가독성 저하. 가드 계산값 재사용 또는 본문 재계산만 남기고 가드를 helper 로 정리 권장.

**(b) 빈 paragraph 분기 — 도달 불가 죽은 코드 가능성.**
`if offsets.is_empty() && chars.is_empty()` 분기는 좁힘 가드
(`n_leading >= 2 && inline_ctrl_count >= 3`) 통과 후에만 도달.
그러나 `offsets` 가 비면 `first_off = 0`, `n_leading = 0` →
가드 `n_leading < 2` 에서 **항상 early-return**. 따라서 빈
paragraph 분기는 **현재 좁힘 조건 하에서 도달 불가능**. 의도된
미래 확장 여지면 주석 명시, 아니면 제거 권장. (기능 영향 없음 —
clean-up 성격)

**(c) trailing controls 추정 휴리스틱.**
마지막 char 후행 컨트롤을 `last_off + last_w` 부터 8 단위로 추정.
char_offsets gap 이 없는 trailing 은 정확 위치 추정 불가 (주석도
인정). pi=394 (3 TAC, 마지막이 나. 라벨 뒤) 에서 동작하나, 다른
trailing 분포에서 마커 위치 부정확 가능. 좁힘 조건이 이를 pi=394
유사 패턴으로 한정하므로 회귀 위험은 낮음 — 다만 한계 명시 적절.

### 3.4 검증 충실성 — 시각 판정 게이트 ⚠️

PR body / 보고서가 제시한 검증:
- cargo test --release --lib: 1297 passed, 0 failed
- cargo fmt --check 통과
- 240 sample 페이지 수 회귀 0
- **HWP5 sample16 p18 시각: "PDF 정합"** ← ⚠️

메모리 룰 `feedback_pdf_not_authoritative` / `feedback_v076_regression_origin`:
**한컴 PDF 는 정답지 아님.** v0.7.6 회귀의 origin 이 "컨트리뷰터가
자기 환경 PDF 를 정답지로 사용". 본 PR 의 "PDF 정합" 주장은
**작업지시자 직접 시각 판정으로 대체 검증 필수** — PR 검증 게이트
미통과 상태로 간주. (cargo test / 페이지 수 회귀 0 은 결정적 검증으로 수용 가능)

### 3.5 잔존 영향 (PR 자체 명시 — 검토 OK)

- HWPX 변종 (`hwp3-sample16-hwp5.hwpx`) 은 parser path 상이 → 본 fix
  미적용. #942/#988 close 영역으로 분리 — 적절.
- HWP5 sample16 p22 paragraph overlap 은 별도 root cause →
  #994 (OPEN) 분리 — 적절. 본 PR scope 정직하게 한정.

## 4. 처리 절차 (간소화 4단계)

1. ✅ PR 정보 확인 (본 문서 §1~2)
2. → 본 검토 문서 작성 + 승인 요청 (현 단계)
3. (필요 시) `pr_995_review_impl.md` — 코드 품질 (a)(b) 수정 요청 시
4. 검증 (로컬 빌드/테스트/clippy + 작업지시자 시각 판정) → `pr_995_report.md`

## 5. 1차 판단 (작업지시자 승인 전 잠정)

| 영역 | 평가 |
|------|------|
| 설계 방향 | 적합 — 좁힘 가드 + composer 격리, 메모리 룰 정합 |
| CI / 결정적 검증 | 통과 (cargo test 1297/0, 페이지 수 회귀 0) |
| 코드 품질 | 경미한 clean-up 여지 (3.3 a/b) — merge blocker 아님 |
| 시각 검증 | **미완** — PDF 자가검증, 작업지시자 직접 판정 필요 (3.4) |
| 이슈 연결 | **확인 필요** — #991 이미 close, reopen/no-op 여부 결정 (2.1) |

**잠정 결론**: 코드·설계·결정적 검증은 양호. **머지 전 2개 게이트**:
(1) 작업지시자의 HWP5 sample16 p18 직접 시각 판정,
(2) #991 이슈 연결 처리 방침 결정.
코드 품질 (3.3 a/b) 은 별도 clean-up 권고 또는 수정 요청 — 작업지시자 판단.

> 본 문서는 검토 계획 + 항목 통합. 작업지시자 승인/피드백 후
> 검증 단계 → `pr_995_report.md` 로 최종 판단 기록.
