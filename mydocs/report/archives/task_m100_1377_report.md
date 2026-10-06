# Task #1377 최종 결과보고서 — 미주 단 render↔typeset 발산 plumbing

- **이슈**: #1377 (M100) / 브랜치 `local/task1377` (base devel)
- **성격**: 미주 단 수직배치 render fidelity — 대형·고위험
- **결론**: 결정-전파 plumbing 채널 **구현 완료(green)**. universal min-clamp 는 미주 PDF-frame
  19건 회귀로 **불가 확정·보류**. 잔여 fidelity 는 별도 타스크 권고.
- **최종 테스트**: `cargo test` 전체 **123 그룹 / 회귀 0**.

## 1. 목표와 결과 요약

| 항목 | 목표 | 결과 |
|------|------|------|
| 발산 진단 | 근원 국소화 | ✅ pi=1128 빈 수식 spacer phantom line_spacing(54 vs 33.6px) |
| 전파 채널 구현 | typeset acc → ColumnContent | ✅ `endnote_para_advance` + typeset 기록, green |
| render 정합(clamp) | 발산 ~0 | ⚠️ 작동하나 19건 회귀로 **보류** |
| 전 exam·골든 회귀 | 무회귀 | ✅ clamp 보류 상태 123/123 green |

## 2. 단계별 경과

- **Stage 1** (`457eb174`): sep2020 p22 단0 발산 국소화 — pi=1129(+20px)·pi=1131(+26px), TAC 그림 뒤.
- **Stage 2~3** (`ba9f6923`·`63674597`·`2387b0c5`): 발산 근원 = pi=1128 빈 수식 spacer 의 phantom
  line_spacing(54 vs typeset 33.6). lever 3종(게이트·vpos·line_spacing cap) no-op/회귀. cap 이 p22
  실해소(1100.59→1087.72)하나 14건 between-notes gap 회귀 → 결정-전파 필요 도출.
- **Stage 4** (`78a5403d`): render-side proxy 2종 음성(line_spacing cap 14건·vpos-delta 31건) →
  typeset acc 전파만 정답이라 판단, 3-step 스펙 확정.
- **Stage 5** (`b9bf5ce6`): **본 세션** — 3-step 구현 + universal clamp 반증 + 보류 결정.

## 3. Stage 5 핵심 — 구현·검증·반증

### 구현 (커밋 `b9bf5ce6`, 순수 추가 54줄)
1. `ColumnContent.endnote_para_advance: HashMap<usize, f64>` (pagination.rs) + 13개 생성 지점.
2. typeset 기록: flush_column/always `mem::take`; split first_h/rest_h + 비-split else 의
   `current_height − en_acc_before` 채집(add/TACmax/A2 전 경로).
3. render min-clamp(build_single_column, `y_offset=new_y` 직후): 보류(주석 명기).

### clamp 검증 — 발산 해소 입증
sep2020 미주 단0 (EN_RENDER rel): pi=1129 시작 **72.1→51.7**(=typeset top), pi=1131 bottom
**445.3→418.9**(=Stage1 typeset). 발산 ~0. **clamp 로직 자체는 정확**.

### universal clamp 반증 — PDF-frame 19건 회귀
`cargo test` 전체에서 미주 PDF-frame 정합 가드 19건 FAILED
(#1189/#1209/#1256/#1261/#1274/#1284 — 2022 oct/nov·2023 sep·2024 between20). 질문 제목/꼬리가
PDF bbox 대비 최대 **~145px 위로 과압축**(예: 문28 856.9px → 711.4px).

**근본(구조적 분리 불가)**: clamp 대상은 전부 textless+control 미주 para. sep2020 pi=1128(nctrl=2)과
19건 spacer 가 동일형태 → phantom 선별 판별식 부재. render advance 가 **PDF 권위**인 미주가 다수라
typeset acc(과소추정) 일괄 clamp = 과압축. 메모리 `tech_trailing_model_no_ssot`("미주 전면 통일 금지")
충돌, Stage4 "typeset acc=보편권위" 가정 **반증**. 게다가 sep2020 발산은 clean 에서 이미 issue_1082
통과(≤5px)인 fidelity nicety → 19 회귀와 **net negative**.

## 4. 최종 산출물 (작업지시자 결정 = 기록 인프라 유지·clamp 보류)

- **유지**: `endnote_para_advance` 채널 + typeset 기록(향후 좁힌 clamp 토대).
- **보류**: render min-clamp — layout.rs 본 위치에 보류 사유 주석.
- **테스트**: 전체 **123/123 green**(회귀 0). HashMap 사용(설계 스펙·`wrap_anchors` 선례 일치).

## 5. 교훈 (재발 방지)

- **per-item 커서 루프 보정**: 조건부 보정은 반드시 **무조건 `y_offset = new_y` advance 다음에 add**.
  기존 advance 라인을 조건부 블록으로 **대체 금지**(비발동 항목 커서 전진 끊김 → 무관 좌표 테스트 회귀).
  본 세션 초안이 이 함정에 빠져 HashMap RandomState 비결정성으로 한동안 오진 → observe-only 통과로 정정.
- **전체 cargo test 필수**: --lib 만이면 19건 PDF-frame 회귀를 놓침([[feedback_full_cargo_test_before_pr]]).

## 6. 권고 (별도 타스크)

- universal clamp **불가 확정**. 잔여 미주 render-fidelity(sep2020 phantom)는:
  - (a) **phantom line_spacing 만 선별하는 좁힌 판별식** 연구(채널 재활용), 또는
  - (b) **phantom 발생지점**(렌더 빈 para line-layout, Stage1~3 국소화) **직접 교정** — 사후 clamp 보다 근본적.
