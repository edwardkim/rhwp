# PR #8120 self-review — PDF 의미 표·문단과 편집 왕복

## 최종 판정

**승인 — 작업지시자가 확정한 #7998 해결 범위의 구현·로컬 검증과 code head CI를 확인했습니다.**
작업지시자는 2026-10-21 “self-review 로 머지 진행합니다”라고 지시했습니다.
미완료 범위와 시각 예외는 아래대로 유지합니다. 이 판정은 모든 PDF의 원래 작성 구조 복원이나
완전한 한컴 호환·parser 할당 안전성 증명이 아닙니다.

검토 code head는 `7ee8bfb6323a27b7d865e8ae287064278879032d`, 최신 기준은
`2e3b4f880bd11d35398e6ea0100846611f13a72d`입니다. 작성 시점 OPEN·Ready·MERGEABLE/CLEAN이며
review-only trailing head의 CI·병합 tree·원격 SHA는 push/merge 직전에 별도로 확인합니다.

## 접수와 역할

| 항목 | 확인 결과 |
|---|---|
| PR / 작성자 / base | [#8120](https://github.com/edwardkim/rhwp/pull/8120) / edwardkim / devel |
| 규모 | 문서·입력·시각 증거 포함 1,547파일, +64,036/-916; 대형 PR 경로 적용 |
| 기본 경로 | collaborator_self_merge; 작성자 self-review이므로 reviewer를 지정하지 않음 |
| 보조 경로 | intake_and_review, local_validation, visual_fixture_evidence, rework_and_exceptions, review_only_fast_pass, post_merge |
| 관련 이슈 | [#7998](https://github.com/edwardkim/rhwp/issues/7998) 참조. 후속 #8106–#8110을 이 PR에서 닫지 않음 |
| 현재 승인 | self-review·병합 진행. 새 source/test 보정이나 전체 검증 재실행 없이 기록만 추가 |

라우터와 선택표 및 위 자식 가이드, 프로젝트 부트스트랩·문서/Git 절차를 읽었습니다.
구현 단계의 계획·독립 근거·사용자 판정은 기존 [결과보고](../../report/task_m100_7998_report.md),
[stage37](../../working/task_m100_7998_stage37.md)·[stage38](../../working/task_m100_7998_stage38.md)를
재사용합니다. 코드 변경 없이 같은 검증을 다시 실행해 새 증거로 표시하지 않습니다.

## 코드와 증거의 대조

아래는 실제 호출 경로와 기존 실행 항목을 연결한 검토입니다. 각 절편의 합성 계약·독립 Print·
사용자 수용 범위를 구분하며, 그림의 유사성만으로 의미 셀 구조를 확정하지 않습니다.

| 변경 주장 | 확인한 생산·소비 경로와 반례 |
|---|---|
| 실제 Table/Cell과 문단 복원 | `table_inference::Ledger`의 source/clip/지역 격자 → `recognition::PageRecognition`의 typed proposal → `structured_lowering::lower`와 `composition::discard_roots`. 모호한 가족은 원본을 소비하기 전에 보류하며 소유가 확인된 제어만 제거. `issue_7998_pdf_table_inference`, `cell_regions`, `local_regions`, `table_composition`의 실제 셀·문자·중첩/교차 반례와 양 저장 검증 대조 |
| 줄·공백·기준선과 편집 | `text_analysis`의 공통 관측 줄 → `structured_lowering::paragraph`/body → `Paragraph/LineSeg`, `GlyphWidths`의 동일 measurer와 CHARSHAPE. 원본 literal 공백과 위치 탭을 구분. `body_soft_wrap`, `body_indentation`, `shared_tab_stops`, `source_space_tabs`, `ideographic_space` 및 실제 Worker의 수정 전 197% FAIL/수정 후 PASS 대조 |
| 셀 테두리·채우기·행열 변경 | `native_borders`의 표준 BorderFill과 `border_paint`의 원본 소유 표식 → `table_ops`의 사전 plan → `border_paint_edit/structure`의 journal·live Table 변경. 넓은 면 채움은 실제 셀 속성으로 편집. 미지원 병합·실제 성장 경계는 변경 전 거절. retained/compound/fill/interval/merged-neighbor 회귀와 독립 한컴 편집 Print 대조 |
| HWPX 병합 행 보존 | `table_row_grid::projection`이 실제 셀의 완전·비중복 coverage와 zone 경계를 증명 → HWPX writer의 투영 → bounded sidecar/fingerprint 검증 후 rhwp logical grid 복원. 무효·충돌 metadata 무시, 투영 실패는 성공 패키지로 숨기지 않음. stage27의 실패 파일·정상 한컴 출력과 topology 회귀 대조 |
| 비사각 텍스트 가림 | 원본 clip chain → `model::text_clip` → TextBox/paint layer의 clipPath → Skia/WebCanvas/SVG/PDF/CanvasKit. 편집 원문을 raster로 바꾸지 않음. HWP/HWPX의 rhwp 재열기와 한컴 native 비호환을 분리. `text_clip`, `text_clip_placement`, `text_clip_owned_lists` 및 최신 executable renderer contract 대조 |
| 일반·회전 글상자 더블클릭 | `onDblClick`의 실제 hit/path/offset/page → `getCursorRectByPathNear`; `transformed_text`와 paint가 같은 `ShapeTransform::matrix_about`을 소비. paragraph 0만으로 편집을 차단하지 않으며 반복 바탕쪽의 다른 페이지 진입은 거절. boolean15쪽·2010 회전 레이블의 실제 입력/삭제·양 저장과 사용자 편집 동작 통과 대조 |
| tac 원본 쪽번호·표 간격 | `PageControlEvents::active_position`을 Typesetter/Paginator가 같은 물리 쪽에서 소비. `complete_stored_prefix_advance`는 저장 줄·formatter·실제 표·다음 문단의 닫힌 점유를 확인하며 편집/분할/무효 저장에는 비적용. `issue_7998_tac_prefix_numbering`과 #6708, 원본66쪽 및 표지 감추기·61쪽 번호 재시작 대조 |
| 기존 그림 원점 | `CellParagraphBefore::stored_anchor_top`을 HeightMeasurer와 셀 실제 배치가 소비. 저장 lead 중복 예약을 제거하고 source-aware 문단 offset 보존. #7902/#6655/#4068 원 해결과 독립 Print 대조 후 기존 ±2px 기대를 유지; 양 형식의 대상5쪽/양 backend raw100% |
| 유한 입력 자원 | operator/reference/XObject/collection/font counter를 각 실제 범위에서 계상하며 명시 host cap 유지. 선택 페이지/명시8참조 거절과 누적 한도 반례를 구분. parser 내부 전체 할당 감사는 `parser_allocation_bounds_proven=false`로 유지 |

## 조판 원칙 준수 판정

| 검토 항목 | 판정과 근거 |
|---|---|
| 제출 전 로컬 검증·Open PR | 충족 — 아래 실제 실행 head별 게이트와 code head의 GitHub Full CI |
| 기존 이슈·해결 PR 선행 분석 | 충족 — stage38의 기존17실패 표와 #6708·그림 원점·선택 페이지/font pool의 원 계약·수정 전후 로그 연결 |
| 구현 근거·일반성 | 충족 — 원본 glyph·clip/paint·문단 단일 소유에 근거; 문서 ID 조건·출력 숨김 없이 불확정 원본 보존 |
| 측정·배치 일관성 | 충족 — 공통 measurer, occupied lead/inline frame, source paint frame의 실제 소비 경로 대조 |
| 반영 PR 기능 보존 | 충족 — #7930의 Qt geometry·빈 clipboard/HTML 공백·emoji 계약과 실제 Qt smoke, 유입7파일의 렌더 경로 비변경 확인 |
| 최종 출력 소비 | 충족 — Native/fresh Docker WASM 및 실제 CanvasKit 강제3쪽·편집/양 저장 결과를 별도로 확인 |
| 분할·이어받기 | 충족(변경 범위 한정) — 저장 TAC 선행 줄의 요구/예약/최종 advance와 다음 표를 대조; 일반 pagination 알고리즘 교체 아님. 표준·보존 경계의 행열/병합 변경은 실제 편집 Print와 원자 거절로 구분 |
| 줄 소속·점유 높이 | 충족 — 저장/재조판 구분, inline 그림 여백 및 multi-line 문단의 순서·후속 문단 검사 |
| 독립 근거·반례 | 충족 — 원본 PDF/독립 한컴 Print와 합성 구조 계약을 분리; 잘못된 진단 파일은 성공으로 세지 않음 |
| 기준값 변경 | 충족 — baseline/90% 허용치 완화 없음; 기존17실패의 기대 변경은 원본 출력 보정·자격23입력37쪽 이후 관계 검사로 정리 |
| 회귀 추가·변경 선행 조건 | 충족 — 자격37쪽과 tac66쪽 양 backend 모두90% 이상. px는 양쪽 raw98% 이상 완료 쪽만 ±2px, 나머지 소유·순서·포함·비겹침 |
| 확대 판독 | 충족 — 기존 메인테이너 시각·구조 통과 유지. 이번 review에서 tac45쪽 Native와 실제 편집 복원43쪽 WASM 비교 패널을 다시 열어 간격·글줄·뒤 행 및 도구 라벨 판독 |
| 주장·검증 범위 | 충족 — 아래 미달 예외·한컴 호환·비신뢰 입력 한계를 해결로 확대하지 않음 |

## 검증 입력과 시각 결과

검토 head의 공개 입력/기준 PDF/TSV·대표 증거 **370개**를 manifest의 SHA-256·바이트 수 및
commit blob과 대조했고 차이는0개였습니다. 개인 견적서의 `local-only-private`2행은 공개
입력으로 세지 않습니다. 별도 비공개 이력서/견적서 원문·식별 이미지와 상용 글꼴은 로컬에
남기라는 지시를 유지하며, 공개 검증을 했다고 바꾸지 않습니다.50MiB 이상 PDF 신규 포함은 없습니다.

- [P5 원문/입력/producer/글꼴 예외](../assets/issue_7998/final-p5/README.md):158입력169쪽/backend.
  재독한 Native 최저89.54211%, 미달6쪽은 해시 고정 정식 `font_mismatch_exception`;
  나머지163쪽 최저92.17391%. WASM169쪽 최저90.70673%, 미달0쪽.
- [devel 비교44행](../assets/issue_7998/final-p5/devel-comparison.tsv):각 backend11입력22쪽.
  Native 견적서1쪽99.79020→95.16159%(-4.62861%p)는 승인된 #8110 후속.
  이를 제외한0.1%p 이상 하락은 없으며 무회귀0건으로 요약하지 않습니다.
- [추가 원본/실제 저장/독립 Print](../assets/issue_7998/submission-20261021/README.md):실제 편집 복원
  HWP/HWPX324쪽/backend 최저97.08677/97.12104%, 누락·쪽수 차이0.
  추가138쪽의 XObject/collection/회전 원본 미달은 승인된 열기·회전 한정 예외입니다.
- [기존 실패 자격23입력37쪽](../assets/issue_7998/submission-20261021/qualification/README.md):
  Native97.90355/WASM97.75192% 이상, 정상 입력·정확한 기대를 제거하지 않았습니다.
- [tac 원본66쪽](../assets/issue_7998/submission-20261021/tac-baseline/README.md):
  Native96.39090/WASM95.51053% 이상,26·45·53쪽 및 사용자 직접 판정 통과.

같은 입력·독립 출력·96dpi·compat2022·인쇄 profile을 사용했습니다. source `d1d3091c5`의
fresh Native/Docker WASM과 후속 동작 불변 lint·문서/검사 변경을 구분합니다. 비교 기준5e2에서
현재 devel2e3까지의 실제 변경은 Qt/Native clipboard7파일이라 해당 렌더 증거를 재사용했습니다.
전체 재출력은 이번 self-review에서 반복하지 않았습니다. 명령·빌드·해시와 원래 실행 경로는
위 공개 manifest 및 stage38에 있고, 이후 청소에서 필요한 원본·producer·실패 로그는 저장소 밖에 보존합니다.

## 로컬 및 정확한 code head CI

로컬 실제 명령·실행 head·소요 시간은 [stage38 최종 표](../../working/task_m100_7998_stage38.md#필수-로컬-게이트-최종-결과)를
따릅니다. release lib3,984 PASS, integration11,209 PASS/0FAIL/기존45skip, Native lib4,041 PASS,
placeholder2/PDF14, fmt·Native/WASM/workspace/root Clippy·workspace build·doc·FFI/Qt가 성공했습니다.
제품을 바꾸지 않은 이번 self-review에서 이 전체 묶음을 반복하지 않았습니다.

code head `7ee8bfb6323a27b7d865e8ae287064278879032d`의 실제 결과:

| Workflow | 확인 결과 |
|---|---|
| [CI](https://github.com/edwardkim/rhwp/actions/runs/39548626497) | success; A/B/C/D 실제 test worker·Native Skia·lint·package·Build & Test 성공 |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39548626330) | success; 계약 검사·실제 Canvas/PDF diff·선택 CanvasKit readiness 모두 실제 실행 성공 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39548626381) | success; 같은 SHA GHAS check의 실패·대기 없음 |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39548626329) | success |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39548626403) | success |

최초81de의 Render Diff는 새 clipPath 종류를 누락한 별도 계약 검사에서 실패했습니다.
7ee8은 E2E/기록만 보완했고, 수정 전 동일 FAIL·수정 후 선행12명령 PASS와 실제 원격 성공을
구분했습니다. 임계값·제품·시각 증거를 바꾸지 않았습니다.

## 남은 범위와 처리 순서

[백로그](../../plans/task_m100_7998_backlog.md)의 #8106–#8110, B21/B22/B23/B25/B26 및
추가 원본 출력 결함을 유지합니다.5pt 셀 전체 채움 확장 수용, 비사각 clip의 rhwp 왕복만의
수용과 한컴 미지원도 그대로입니다. 별도 후속 이슈를 이 PR의 해결로 종료하지 않습니다.
원 이슈 #7998은 PR의 `Refs` 범위를 유지하며 별도 종료 승인 없이 close하지 않습니다.

문서-only trailing commit과 오늘할일을 같은 PR에 포함하고, 정확한 최신 base/head의
merge-tree·공백·링크·기존 기록 보존 검사를 통과한 뒤 push합니다. 최신 trailing CI의
실제 fast-pass와 aggregate, PR SHA/mergeability를 확인하고 이미 승인된 병합을 실행합니다.
일반 squash merge를 사용하며 branch protection을 우회하는 admin 옵션은 사용하지 않습니다.

## Merge 후 contributor PR comment 계획

작성자 self PR이므로 외부 기여자 credit이나 GitHub self-approve를 만들지 않습니다.
PR의 한국어 후속 comment에는 실제 merge SHA, 위 code CI와 trailing 결과, 승인 범위·남은
이슈를 기록합니다. 같은 SHA·증거의 comment가 있으면 중복 게시하지 않습니다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을
직접 연결하고 아래 파일을 `<merge-commit-sha>`로 고정한 Markdown 이미지로 표시합니다.

- `mydocs/pr/assets/issue_7998/submission-20261021/resource-restored-native-review-043.png`
- `mydocs/pr/assets/issue_7998/submission-20261021/resource-restored-wasm-overlay-043.png`
- `mydocs/pr/assets/issue_7998/submission-20261021/tac-baseline/native-review_045.png`

실제 확인 범위는 위324쪽/backend와66쪽/backend 및 그 대표43/45쪽이며, 미달0·쪽수 차이0은
이 범위만의 결과입니다. 모든 승인 예외를 합산해 전체 fidelity 통과로 쓰지 않습니다.
본문은 UTF-8 파일을 `--body-file`로 보내고 API 재조회로 실제 LF·BOM·한글·이미지 SHA를 확인합니다.
병합 후 devel fast-forward와 duration 갱신 결과를 확인하고, 사용자 WIP/공유 target 및
활성 작업을 보존하는 조건으로 작업 전용 branch/worktree/output을 정리합니다.
