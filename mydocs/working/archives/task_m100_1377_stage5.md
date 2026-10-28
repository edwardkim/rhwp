# Task #1377 Stage 5 — 결정-전파 plumbing 구현 + universal clamp 반증 (기록 인프라 유지·clamp 보류)

- **이슈**: #1377 (M100) / 브랜치 `local/task1377`
- **단계**: Stage 5 — Stage4 3-step 설계 구현 → universal min-clamp 가 PDF-frame 19건 회귀 → clamp 보류
- **결과**: typeset 기록 채널(plumbing)은 구현·유지. render min-clamp 는 **전면 적용 불가**로 실증·보류.
  전체 테스트 **123/123 green**. 작업지시자 결정: 기록 인프라 유지·clamp 보류.

## 1. 구현한 것 (Stage4 3-step 설계 그대로)

1. **`ColumnContent.endnote_para_advance: HashMap<usize, f64>`** 필드 추가
   (pagination.rs 구조체 + 13개 생성 지점: layout.rs scratch·pagination/state.rs flush×2·
   page_number.rs·layout/tests.rs×5·integration_tests.rs·copy_converged_pages offset 적용).
2. **typeset 기록** — `TypesetState.current_column_endnote_advance` 에 미주 para 별 누적 advance
   기록 후 flush_column/flush_column_always 에서 `mem::take` 로 ColumnContent 전달.
   - 비-split else 경로: `en_acc_before` 스냅 → 블록 종료 시 `current_height - en_acc_before` 채집
     (add/TACmax/A2 어느 경로든 실제 단 advance 채집).
   - split 경로: 단 A 의 first_h + 단 B 의 rest_h 를 각 단 맵에 기록.
3. **render min-clamp** (build_single_column): `endnote_flow` 단의 문단 항목에서 render advance
   (`new_y − _y_in`)가 기록된 typeset advance 보다 크면 `y_offset = _y_in + ts_adv` → **현재 보류**.

## 2. clamp 검증 — 의도대로 작동(발산 해소) 입증

sep2020(`samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`) 미주 단0 (EN_RENDER, rel):

| pi | clamp 전(=clean) | clamp 후 | typeset top(Stage1) |
|----|------|------|------|
| 1128(빈 수식 spacer, vis=false nctrl=2) | dy=54.1 | dy clamp→33.6 | 33.6 |
| 1129 (TAC 그림) 시작 | 72.1 (+20px 발산) | **51.7** | **51.7** ✅ |
| 1131 (TAC 그래프) bottom | 445.3 (+26px) | **418.9** | **418.9** ✅ |

→ phantom line_spacing 발산을 typeset 정합으로 교정. 발산 ~0. **clamp 로직 자체는 정확**.

## 3. 그러나 universal clamp = PDF-frame 19건 회귀 (전면 적용 불가)

`cargo test` 전체 → **issue_1139 파일 미주 PDF-frame 정합 가드 19건 FAILED**:
- #1189/#1209/#1256/#1261/#1274/#1284 계열 (2022 oct/nov·2023 sep·2024 between20).
- 증상: 미주 질문 제목/꼬리가 PDF bbox 대비 최대 **~145px 위로 과압축**
  (예: `issue_1284_2024_between20_page22_23` 문28 제목 PDF 856.9px → clamp 후 711.4px).

**근본 원인 (구조적 분리 불가):**
- clamp 발동 대상은 전부 `vis=false`(textless) + `nctrl≥1`(TAC 그림/수식 컨트롤) 미주 para.
  sep2020 pi=1128 (`nctrl=2`)과 19건 회귀 유발 spacer 가 **동일 형태** → phantom 만 선별할
  안전 판별식 부재 (`nctrl==0` 으로 좁히면 pi=1128 자체가 제외돼 무의미).
- render advance 가 **PDF-정합 권위**인 미주가 다수 → typeset acc(과소추정)로 일괄 clamp = 과압축.
  메모리 룰 **`tech_trailing_model_no_ssot`**("미주 trailing 전면 통일 금지 — render 다줄↔typeset
  1984HU 가정 불일치")와 정확히 충돌. Stage4 의 "typeset acc=보편 권위(14테스트 보증)" 가정 **반증**.
- 게다가 sep2020 발산은 **clean 에서 이미 테스트 통과**(issue_1082 sep2020 ≤5px). 즉 clamp 의
  유일 이득은 테스트 미발현 fidelity nicety. 19건 PDF-frame 회귀와 맞바꾸는 건 **명백한 net negative**.

## 4. 구현 중 자기-회고 (디버깅 기록)

- clamp 초안이 필수 `y_offset = new_y;` 를 **대체**(삭제)해, clamp 비발동 항목(전 본문 포함)의
  커서 전진이 끊겨 무관한 좌표 테스트가 깨짐. 원인을 한동안 "HashMap RandomState 비결정성"으로
  **오진**(BTreeMap 전환·probe 실험)했으나, observe-only(=`y_offset=new_y` 유지) 버전이 통과함을
  근거로 자가 버그로 규명·정정. 최종본은 `y_offset = new_y` **유지 후** clamp 추가 형태(현재는 보류).
- 교훈: per-item 커서 루프에서 조건부 보정은 **반드시 무조건 advance 다음에 add**. 기존 라인 대체 금지.

## 5. 최종 상태 (작업지시자 결정 = 기록 인프라 유지·clamp 보류)

- **유지**: `endnote_para_advance` 채널 + typeset 기록 (향후 좁힌 clamp 의 토대). diff 순수 추가 54줄.
- **보류**: render min-clamp — 보류 사유를 layout.rs 본 위치 주석에 명기.
- **전체 테스트 123/123 green** (회귀 0). HashMap 사용(설계 스펙·`wrap_anchors` 선례 일치).

## 6. 권고

- universal clamp 는 **불가** 확정. 잔여 미주 render-fidelity(sep2020 phantom)는 **phantom line_spacing
  만 선별하는 좁힌 판별식** 연구가 선행돼야 함(별도 타스크 권장). 채널은 그때 재활용 가능.
- 또는 phantom line_spacing 의 **발생 지점(렌더 빈 para line-layout)을 직접 교정**하는 접근이
  clamp(사후 보정)보다 근본적일 수 있음 — Stage1~3 의 "빈 para blank-line line_spacing 전진" 국소화 활용.
