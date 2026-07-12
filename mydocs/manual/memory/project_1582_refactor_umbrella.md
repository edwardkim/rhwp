---
name: project_1582_refactor_umbrella
description: "#1582 = 0.8/v1.0 아키텍처 리팩토링 umbrella 이슈 — 첫 단계 SourceProvenance+LayoutCompatibilityProfile, feature freeze 선행"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

**이슈 #1582 (OPEN, 닫지 않음)** 는 0.8/v1.0 구조 정리의 **기준(umbrella) 이슈**다
(2026-06-27 isty2e 감사 제안 → jangster77 방향 확정). 대규모 rewrite 아님.

**핵심 진단**: 9개 도메인 모델 축(canonical IR/package 보존/layout 호환/media/layout/visual/
session/WASM DTO/diagnostics)이 같은 struct·모듈에 혼재 → 국소 수정의 회귀 범위가 큼.
`Document` 의 `extra_streams`·`is_hwp3_variant` 혼재, `DocumentCore`=session aggregate,
`lib.rs` public surface 과대.

**확정된 진행 방식**:
- 착수 전 **feature freeze + baseline freeze** (public Rust API/WASM JSON/CLI output/대표
  샘플 behavior 고정).
- **1단계 = `SourceProvenance` + `LayoutCompatibilityProfile` 내부 policy 도입** — 기존
  `is_hwp3_variant`/`is_hwpx_source` 계열 shim 유지, observable behavior 불변으로 layout
  호환 판단 소유권만 분리 (저위험).
- `PackagePreservation`/`EmbeddedMediaProjection`/`VisualScene` 분리는 roundtrip 회귀 위험이
  커서 후속 단계.

**Why**: 0.8 릴리즈([[project_v080_hwpx_save_milestone]]) 시점과 결합된 리팩토링 계획.
이 축의 실사례가 #1608(is_hwp3_origin 오탐지)·#1230(emit/배치 두-경로 정정) —
[[feedback_fix_scope_check_two_paths]] 가 반복되는 구조적 이유.

**선행 리팩토링 3회 (mydocs/feedback/r-code-review-1st~5th.md, 리뷰 주도 사이클)**:
① Task 146 — layout.rs 거대 함수 해체(paginate_with_measured 1,456→120줄).
② Task 149 — wasm_api.rs 2.4만줄 God Object → `document_core/` 신설 + CQRS(commands/queries).
   리뷰 점수 5.4→9.0 도약. ③ 4차 리뷰(03-23) 후속 정리(Task 346/348/400 계열) — 200개 타스크
   후 대형 메서드 재발견(layout_column_item 827줄) = **부채 재축적 패턴 관측**. #1582 는 이
   사이클의 다음 도약(사실상 6차 리뷰 격 외부 감사). Task 149 가 behavior 불변으로 완수된
   전례가 #1582 1단계(shim+소유권 분리) 실행 가능성의 근거.

**How to apply**:
- 아키텍처 경계를 건드리는 이슈/PR 검토 시 #1582 방향(소유권 분리, shim 우선, behavior 불변)과
  정합 확인. 관련 작업은 #1582 를 참조 이슈로 연결.
- 감사 문서는 이슈 첨부만(레포 미반영). 제안자 isty2e 는 수행 여부 미정 — 착수 주체 미정 상태.
- freeze 타이밍은 현재 PR 유입 속도와 긴장 관계 — 작업지시자 결정 사항.
- **실행 관문 #1883 = CLOSED (v2 확정, 2026-07-05)**: 계획 게시 → 두 리뷰 취합 완료 — 재진단(`tech/task_m100_1883_diagnosis.md`: 영점 = 1,200줄 초과 70·
  CC>25 80·최대 288) + 마스터 플랜(`plans/refactoring_plan_2026.md` 초안 v1: Phase 0 freeze →
  1 Provenance/Profile → 2 복잡도 해체 → 3 Document 축 → 4 6차 리뷰) + 리뷰 안건 5개.
  거버넌스는 [[project_refactor_governance]](SOLID+복잡도), 복잡도 공식 측정 = 대시보드
  (`scripts/metrics.sh --snapshot` → `mydocs/metrics/{날짜}/` 커밋 보존). #1582 는 umbrella 존속.
- **리뷰 진행 상태 (2026-07-04)**: jangster77 회신 완료(방향 찬성 + 보완 4: ①freeze 진입 전
  렌더/layout PR inventory ②baseline manifest 고정 ③성공 기준 산식 고정 — 영점 80(대시보드
  전체) vs 70(.rs 한정) 모집단 정의 실증 ④public-api 는 advisory 시작·WASM JSON 별도 계약.
  Phase 2 PR 단위 = "함수 내 한 책임/추출 단위 = 1 PR"). **postmelee 회신 대기 — 도착 전
  v2 개정 착수 금지(작업지시자 확정)**. v2 반영 대기 목록: jangster77 4건 + PR 단위 +
  object_ops 도메인별 분할(2e 후보, 함수 175개/7개 도메인 응집·저위험) + studio TS 핫스팟
  (input-handler.ts 4,443줄) 언급 여부.
- **단계 재편 확정 (2026-07-04 작업지시자, #1883 공지됨)**: **1차 = 복잡도 해체**(복잡도
  높은 순 × 위험 낮은 순), Provenance/Profile 분리는 후순위. jangster77 의 "Provenance 선행"
  우려와의 조율안 = 1차에서 **소스-포맷 분기를 안 건드리는 추출 단위부터** 진행, 분기 밀집
  구간 추출은 Provenance/Profile 이후 배치 — v2 에 설계 반영.
- **라운드 1 (#1904) 완료** (2026-07-04, devel bf5228df): Phase 0 freeze(manifest+OVR baseline
  5샘플) + object_ops 8모듈 분할 + typeset_section_with_variant 7,059줄·CC 282→1,555줄·CC 104.
- **라운드 2 (#1925) 완료** (2026-07-05, devel 9f4ea6bb): layout_composed_paragraph CC 288→226·
  3,771→3,071줄, 추출 3건(마커/est/빈줄, 전건 소스분기 0), 행동 회귀 0. B run 방출 루프
  (의존 45/mut 11)는 RunEmitState struct 설계 선행으로 이연 — EndnoteFlowState(라운드 1 이연)와
  같은 패턴. 분할 과도기 CC>25 +1~2/라운드는 실증된 계획 효과.
- **라운드 3 (#2001) 완료·CLOSED** (2026-07-06, devel ddadb114): parse_paragraph_list(HWP3)
  CC 234→76·2,270→896줄, Hwp3CharScan(5필드)+추출 3함수, break 28곳 전수 분류 후 반환값
  치환. metrics.sh 소품 3건(요약 줄 버그·--snapshot 라벨·--no-coverage) 정비·실전 검증.
  1위 함수 3라운드 연속 해소(282→104, 288→226, 234→76), 전체 최대 CC 288→226.
- **typeset.rs 재성장 공식 관측** (r3 스냅샷): typeset_section_with_variant 104→117,
  typeset_block_table 112→127 — 07-05 기능 PR 유입분. 재축적 추세 감시.
- **라운드 4 (#2003) 완료·CLOSED** (2026-07-06, devel b2fa0f5d): layout_composed_paragraph
  CC 226→146(RunEmitState 8필드 값왕복+emit_line_runs) + parse_object_control_char 104→37
  (Hwp3DrawingCarry+개체 디스패치 체인 통이동). **최대 CC 179 — 첫 200 미만** (영점 288
  대비 −38%), 1위 함수 4라운드 연속 해소, 행동 회귀 통산 0. dependabot 4건도 같은 날 처리
  (quick-xml 0.41 보안 릴리스 로컬 게이트 포함).
- **의존 스캔 사각지대 2종 기록**: eprintln 포맷 문자열의 가짜 대입 매칭 / 인덱스 대입
  `v[i] =` 미검출 — 차기 라운드 분석 절차에서 보정 필수 (r4 보고서 §3).
- **라운드 5 (#2026) 완료·CLOSED** (2026-07-07, devel 33bf1ad6): typeset_section_endnotes
  CC 179→6·5,539→141줄 — #1904 이연 EndnoteFlowState(7필드) 완결, prepare_endnote_emit
  CC 22(경계 미만), 본체 typeset_endnote_paragraphs 153(다음 분해 대상). **1위 함수
  5라운드 연속 해소**(282→104, 288→226, 234→76, 226→146, 179→6), 행동 회귀 통산 0.
- **라운드 6 (#2029) 완료·CLOSED** (2026-07-08, devel 641a685f): layout_partial_table
  CC 169→55·1,773→549줄 (+layout_partial_table_cells 115 분리 — muts 0/단일 sink,
  6라운드 중 최적 추출 조건). 축소 판단 1건(준비부/마무리 = 표 프레임 기하 생산-소비 쌍
  유지, PartialTableFrame 은 표 계열 2차로). **최대 CC 288→153 (−47%)**, 1위 함수
  6라운드 연속 해소, 행동 회귀 통산 0.
- **의존 스캐너 사각지대 체크리스트 6종 확정** (r4~r7): ①포맷 문자열 가짜 대입 ②인덱스
  대입 ③클로저 파라미터/필드 리터럴 오탐 ④중첩 필드 push ⑤&mut 인자 전달형 변이
  ⑥클로저 캡처 변이(FnMut — fn 승격 시 컴파일러 E0384 검증이 안전망) —
  추출 전 분석의 표준 점검 항목.
- **라운드 7 (#2064) 완료·CLOSED** (2026-07-08, devel 33959dd1): typeset_endnote_paragraphs
  **1차 회전 (보수 목표)** CC 153→138·4,397→4,227줄. 국면 지도 8구획(P1~P8) 작성 후 저위험
  2건만 추출 — compute_endnote_metrics(P3, EnMetricsVars Copy 26필드+&mut bool)/
  emit_endnote_split(P8, 직접 파라미터 21개·mut→반환 bool), 모두 CC<25·예외 목록 무증가(87).
  평탄 파이프라인(지배 블록 2개뿐, let ~230)이라 "국면 지도 없이는 추출하지 않는다" 원칙 수립.
  **P6 fit 판정 연쇄(~1,600줄, CC 밀집처)·P7 배치(~970줄)는 2차 회전 인계**
  (report/task_m100_2064_report.md §5). 1위 함수 7라운드 연속 해소, 행동 회귀 통산 0.
- **라운드 8 (#2067) 완료·CLOSED** (2026-07-09, devel 3728abfb): layout_composed_paragraph
  **3차 회전** CC 146→**98**·2,093→1,589줄 (라운드 2·4·8 누적 288→98). 추출 5건 — 순수 2
  (compute_line_extra_spacing/collect_shape_marker_labels) + TAC 3
  (place_unmatched_line_tac_pictures/place_empty_line_tac_forms/place_empty_line_inline_equations,
  TacPictureLineVars 7필드·EquationTacLineVars 20필드). **전부 CC≤25** — 241줄 수식 블록도
  CC 18 (깊은 중첩 가중이 추출로 해소되는 전형). 소스분기(is_hwp3_variant)는 §1 대로
  caller 유지(hwp3_indent_scale 값 전달). 1위 함수 8라운드 연속 해소, 행동 회귀 통산 0.
- **라운드 9 (#2079) 완료·CLOSED** (2026-07-09, devel 47fbe1e4, **goal 방식 — 단계별 승인
  생략·게이트 자체 검증·최종 일괄 보고, 작업지시자 지시로 첫 적용**): typeset_endnote_paragraphs
  **2차 회전** CC 138→**118**·4,227→3,461줄(−766). 핵심 = **104줄 렌더-시뮬 예측부 4곳 중복
  발견·소거**(predict_endnote_render_y, −272줄·CC −9) + P6 판정 함수군 judge_* 17건(읽기 전용
  &self, 직접 파라미터, 전부 CC≤25). 집계자(advance_for_fit 논리 76)는 지역 불리언 집계라
  추출 부적합 — 잔류가 옳다. 배치당 소득 체감(−7→−4)으로 §0 축소 종료, P7(~970줄) 3차 회전
  인계. 불확실 파라미터는 컴파일러 E0425/E0308 로 수렴시키는 방식 정착. 1위 함수 9라운드
  연속 해소, 행동 회귀 통산 0.
- **goal 루프 R10~R13 완료** (2026-07-09, devel c1ed5407→27b3461f→61766c3b→0e12ba3b —
  **연속 자율 4라운드 첫 실행**, 작업지시자 승인 "강한 회귀 검증 하네스 구축 판단"):
  R10 #2085 typeset_block_table 129→**37**(+scan_block_table_split_rows 93, §5 심사) /
  R11 #2089 layout_table_cells 124→**42**(+layout_horizontal_cell_paragraphs 83, §5 심사) /
  R12 #2091 layout_table_item 121→**48**(+layout_table_control_block 75, §5 심사 —
  **early_return 프로토콜** 신규: 함수 조기 return 을 Option 신호로 반환) /
  R13 #2094 typeset_section_with_variant 120→**104**(라운드 1 수준 복원, 신규 3함수 전부
  CC<25 — **continue 신호 프로토콜**: 외부 루프 continue 를 bool 반환으로).
  전체 최대 CC 129→**118**, CC>25 87→90(+3, §5 과도기 — 신규 대형 3건은 후속 분해 후보 등재).
  행동 회귀 통산 0 유지 (라운드당 게이트 전수). 루프 운영 교훈: ①이슈 채번 후 브랜치명
  즉시 검증(#2089 채번 점프 사고) ②추출 작업 전 태스크 브랜치 확인(devel 워킹트리 사고
  2회 — stash 이동으로 복구) ③게이트는 트리 안정화(커밋) 후 착수(R12 오염 중단·재실행).
- #2085·#2089·#2091·#2094 CLOSED (2026-07-09 일괄 승인, devel 머지 검증 완료).
- **goal 루프 2차 R14~R17 완료** (2026-07-09, devel 0f093be8): R14 #2106
  typeset_endnote_paragraphs 3차 118→115 + judge_* 12건 — **분해 포화 선언** (판정 체인의
  공식 CC 기여 미미 실증, 잔여는 aggregator/shadow 파이프라인/소스분기 인접 = Provenance
  이후 재설계 영역, 회전 후보 제외). R15 #2120 build_single_column 116→**90**
  (render_para_border_groups 285줄 통이동). R16 #2122 ir_diff 116→**37** — **핵심 발견:
  fn-지역 emit macro 가 호출부 ~30곳 인라인 전개로 CC 증폭**, IrDiffEmitter struct 전환이
  CLI 진단 계열 표준 처방 (출력 3모드 바이트 동일 검증). R17 #2126 §5 상환 시도 →
  **추출 기법 소진 실증** (소형 추출 −1, 대형 통이동은 개수 순증 모순) — §5 등재 3건
  (scan 93/horizontal_cell 82/table_control 75)의 상환 = 유형별 핸들러 재설계 영역 인계.
- **Phase 2 국면 판정 (r17)**: 최대 CC 288→**117**(layout_partial_table_cells — 외부 PR
  유입 재성장), "1위 함수 해소" 축 종료 국면. CC>25 는 80→93 (기능 유입+통이동 산물) —
  **"개수" 축은 마스터 플랜 Phase 재평가 안건** (핸들러 재설계 기획 필요, 작업지시자 결정).
  행동 회귀 17라운드 통산 **0건**. #2106/#2120/#2122/#2126 CLOSED.
- **#1904 CLOSED — 1차 리팩토링 종결·총평 게시** (2026-07-09, 작업지시자 지시): 서브 이슈
  16건(R2~R17) 전원 CLOSED. 결산 — 최대 288→117(−59%, 기준 <100 근접 미달)·상위20합 −26%·
  **CC 총합 +2.3% (미달, 사후 계산으로 드러난 지표 설계 허점)**·회귀 0(달성). 교훈: 통이동은
  이동이지 감소가 아님, 총량 지표 부재가 이동을 성과로 오독시킴, 유입 활발 국면에선 재성장이
  상환 잠식. **후속 = #2130 산식 개정 (§5 v2.1, #1582 서브)**: 총량 지표(총합·상위20합)
  공식화 + "감소 잠재량 순" 선정(실증 유형: ①중복 블록 R9 ②macro 확장 R16 ③공통 인수
  판정 체인). CI 유입 가드·freeze 는 작업지시자가 채택하지 않음 (유입 활발 국면). 재설계
  (Provenance/핸들러 통합)는 유입 소강 시 별도 기획.
- **【현행 방침】리팩토링 유보 — 기능 구현 우선** (작업지시자 확정 2026-07-11, #1582
  결론 코멘트): 2차 사이클 R1(−2)·R2(−13) vs 유입(+25/일) 실측으로 비효율 결론.
  근거 = 기초 공사 완비(회귀 하네스 19라운드+주간 PR 30건 회귀 0 / 최대 CC 288→117 /
  이음새 실익 #2156 안착). **재개 트리거 3종**: ①최대 CC 200 재돌파 ②소스분기발 버그
  반복 유입(Provenance 가 실제 장애물화) ③유입 소강기. v2.1 산식·스캐너·시계열
  대시보드는 관측 인프라로 존속 — 주기 체크포인트(스냅샷 --diff)로 드리프트 감시.
  #2131·#2132·#2173 CLOSED. **리팩토링 라운드 제안은 트리거 충족 전에는 하지 않는다.**
- **#2130 CLOSED — 산식 v2.1 확정** (2026-07-09, devel 20bac9c7): §5.1 지표 이원화
  (분포=위험 집중도 / **총량=CC 총합·상위20합·>25합 = 라운드 성공 기준**), 통이동은 준비
  단계로만 계상. 선정 = "감소 잠재량 순"(`tools/reduction_potential.py`, 실증 4유형:
  ①중복 ②지역macro ③guard체인 ④소스분기). metrics.sh 에 총량 3종+`--diff` 함수별 변화 표.
  **차기 후보(잠재량 순)**: compute_char_positions(중복 243) / build_char_properties_json_by_id
  (134) / measure_table_impl(106) / apply·format_shape_props_inner 쌍(87+75, 함수간 동형) /
  paginate_pass(소스분기 9곳+CC 105 — **잠재량·CC 동시 상위, 재개 시 1순위**).
  감사 문서(isty2e)는 "유입 소강 시 재설계" 기준 텍스트로 승격 — #1582 재검토 코멘트
  (2026-07-09): Stage 0 사실상 완료, Stage 1(Profile)은 shim-점진 방식이면 freeze 불요,
  Stage 착수 전 verdict 재실측 필수 (06-27 좌표 어긋남).
- **#1883 CLOSED — 마스터 플랜 v2 확정** (2026-07-05, devel 5d5a635e): 두 리뷰 취합 완료.
  v2 핵심 = §1 금지 목록(분기 이산/밀집 구간/혼합 커밋 금지 — PR 체크 기준), §3 gate 이원화
  (렌더=1차 gate, API·WASM JSON·CLI output=advisory→2차 착수 전 스냅샷 관문), §5 산식 고정
  (모집단=runtime 로직 제외 4군, CC 예외 심사제 부록 A, 과도기 +1~2 허용), §6 PR 설명 3요소
  (responsibility/무변동 gate/다음 앵커) + 의존 임계(읽기 12/mut 10 → struct 선행).
  이후 리팩토링 PR 검토·계획은 refactoring_plan_2026.md v2 를 기준으로 판단한다.

관련: [[project_v080_hwpx_save_milestone]] [[feedback_fix_scope_check_two_paths]] [[feedback_hancom_compat_specific_over_general]]
