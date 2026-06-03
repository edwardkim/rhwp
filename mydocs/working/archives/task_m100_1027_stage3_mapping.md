# Stage 3 정정 매핑 — #1027: 실제 페이지네이션은 typeset.rs (engine.rs 아님)

- 타스크: #1027 / 브랜치 `local/task1027`
- 작성일: 2026-05-20
- 단계: Stage 3 — 전용 리팩터 정적 매핑 (1차 deliverable)

## 1. 결정적 정정 (이전 Stage 3 시도 무효 사유)

페이지네이션 구현이 **두 개**이며, 기본 경로를 착각했다:

- **기본(active)**: `src/renderer/typeset.rs` `TypesetEngine::typeset_section` — export-svg / dump-pages / 렌더링이 사용.
- **fallback**: `src/renderer/pagination/engine.rs` `Paginator` — **`RHWP_USE_PAGINATOR=1` 일 때만** (`rendering.rs:1548~1552`: "TypesetEngine을 main pagination으로 사용. RHWP_USE_PAGINATOR=1 로 fallback").

이전 Stage 3 의 모든 편집·디버그는 **fallback engine.rs** 에 있었기에 무효였다(트레이스 미발화로 확정, TYPESET_DRIFT(typeset.rs)는 발화). → 메모리 `two-pagination-engines` 기록.

## 2. 실제 수정 surface (typeset.rs)

노트의 단락 fit/누적 결정:
- `typeset.rs:1596` `if st.current_height + fmt.height_for_fit <= available { place }`.
- `typeset.rs:1606~1610` 누적:
  ```rust
  st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
  ```
  - **단단(col==1)은 `fmt.total_height`** 사용 → sb + trailing_ls 과다 포함(#1027 Stage1 drift) = 노트 over-pack 직접 원인.
  - 주석(1601~): total_height 는 #359(k-water-rfp 311px drift)를 막으려 도입 — 즉 한 곳을 막으려다 다른 곳(노트)을 밀어내는 tension.
- TYPESET_DRIFT 진단도 이 함수(1467~1499).

## 3. 전면 vpos 앵커 구현 설계 (typeset.rs)

1. `TypesetState` 에 `page_vpos_base: Option<i32>` 필드 추가(현 상태엔 없음). 생성자 None, `reset_for_new_page`(321) 에서 None.
2. 단락 fit 함수에서 fit(1596) **이전**:
   - `page_vpos_base` None 이면 현재 단락 first_seg.vpos 로 설정(페이지/단 첫 항목).
   - 아니면 `current_height = (first_seg.vpos − base) px` 로 동기화(상/하향).
   - bypass: TAC 수식/그림/글앞뒤 Shape, vpos-reset(line>0 && vpos==0), multicolumn(col>1 은 별도 stacking).
3. 누적(1606): vpos 앵커 적용 시 total_height vs height_for_fit tension 재평가(#359 회귀 확인).

## 4. 리스크 / 검증 계획

- **기본 페이지네이션 변경 → 전 문서 영향**. 광범위 골든 재판정 필수(svg_snapshot 8 + 비공개 184p LAYOUT_OVERFLOW + 페이지 수 + 다른 샘플 sweep).
- #359(k-water-rfp), 다단(exam_eng), atomic TAC(1614~) 회귀 집중 확인.
- 단계적: (a) col==1 단단만 우선 적용 → 검증, (b) 다단 확장.

## 5. 다음 (Stage 3 구현)

위 설계대로 typeset.rs 에 `page_vpos_base` 추가 + fit 직전 앵커 구현 → 노트 8쪽 + 무회귀 검증. (engine.rs 는 RHWP_USE_PAGINATOR 회귀 테스트용으로만 별도 정합.)
