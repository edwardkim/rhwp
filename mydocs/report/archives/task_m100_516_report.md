# Task #516 최종 보고서 — Web Canvas 그림 워터마크 효과 + AI 메타정보 + 다층 레이어 도입

## 결함 요약

| 항목 | 내용 |
|------|------|
| **이슈** | [#516](https://github.com/edwardkim/rhwp/issues/516) |
| **마일스톤** | v1.0.0 (M100) |
| **assignee** | edwardkim |
| **대표 fixture** | `samples/복학원서.hwp` (가운데 고려대학교 엠블럼) |

본 task 의 본질은 **rhwp-studio (web Canvas) 의 그림 효과 적용 + AI 활용 메타정보 + 다층 레이어 인프라 도입** 의 3 영역 통합. Discussion [#529](https://github.com/edwardkim/rhwp/discussions/529) 의 후보 결정 (옵션 C HTML Hybrid) 에 따라 다층 레이어 인프라 도입 첫 사이클로 확장.

## 작업지시자 정합 사항 (시간순)

1. **#516 본질 확장**: 단순 web 그림 효과 누락 정정 → 한컴 워터마크 효과 통합 처리
2. **AI 메타정보 추가**: PageLayerTree JSON 의 `watermark.preset` 필드, dump 의 `[image_attr]` 줄
3. **편집자 의도 보존**: 한컴 자동 프리셋 (`b=70, c=-50`) vs 복학원서 편집자 정의 (`b=-50, c=70`) 정합
4. **다층 레이어 도입**: Discussion #529 후보 3 안 비교 후 옵션 C 확정. M200 후보 B (WebGPU) 단계적 경로 보존
5. **DTP 정체성 통찰**: 아래아한글 = 쿽 익스프레스 대체 의도 → DTP 인프라로서의 다층 레이어
6. **워터마크 시각 별도 task**: 회색조/투명도 정합 시각은 분리 (이슈 #535)

## 본 task 의 최종 범위

### 포함 (완료)

| 영역 | 내용 |
|------|------|
| 1. ImageAttr 헬퍼 | `is_watermark()` / `is_hancom_watermark_preset()` / `watermark_preset()` |
| 2. dump 메타정보 | 4 dump 사이트에 `[image_attr] effect=... b=... c=... watermark=...` 추가 |
| 3. PageLayerTree JSON | `watermark.preset` + `wrap` + `mime` 필드 (PR #510 정합 + 다층 레이어 분리용) |
| 4. PageControlLayout JSON | image control 의 `wrap` 필드 (옵션 3-C hit-test 분기용) |
| 5. Web Canvas effect/brightness/contrast | `compose_image_filter` 헬퍼 + CSS filter 적용 |
| 6. 다층 레이어 wrap 필터 | `LayerFilter` enum + `renderPageToCanvasFiltered(layer_kind)` API |
| 7. PCX → PNG 변환 (PageLayerTree mime) | overlay `<img>` data URL 호환을 위한 변환 적용 |
| 8. TS overlay 인프라 | page-renderer 의 `applyOverlays` + `getOverlayImages` + `createOverlayLayer` |
| 9. hit-test 옵션 3-C | `findPictureAtClick` 두 단계 (텍스트 우선 / BehindText 후순위) |
| 10. 회귀 테스트 | `tests/issue_516.rs` (8 tests) |

### 분리 (별도 task)

| 영역 | 분리 task |
|------|----------|
| 워터마크 시각 (회색조/투명도 정합) | [이슈 #535](https://github.com/edwardkim/rhwp/issues/535) |

## 변경 영역 정리

| 파일 | 영역 | 변경 |
|------|------|------|
| `src/model/image.rs` | 헬퍼 | `ImageAttr` 에 워터마크 식별 메서드 3 |
| `src/model/shape.rs` | TextWrap 직렬화 | `serde::Serialize` derive |
| `src/renderer/render_tree.rs` | ImageNode 확장 | `text_wrap: Option<TextWrap>` 필드 |
| `src/renderer/web_canvas.rs` | Web Canvas | `LayerFilter` enum + `compose_image_filter` + `should_render_image` + image dispatcher 분기 |
| `src/wasm_api.rs` | wasm API | `renderPageToCanvasFiltered` 신규 |
| `src/paint/json.rs` | PageLayerTree JSON | `watermark.preset` / `wrap` / `mime` 필드, PCX/BMP→PNG 변환 적용 |
| `src/document_core/queries/rendering.rs` | PageControlLayout JSON | image 의 `wrap` 필드 |
| `src/main.rs` | dump (4 사이트) | `[image_attr]` 메타 출력 |
| `src/renderer/layout/paragraph_layout.rs` 외 5 파일 | ImageNode 생성 site | `text_wrap: Some(pic.common.text_wrap)` 전파 (8 site) |
| `rhwp-studio/src/core/wasm-bridge.ts` | bridge | `renderPageToCanvasFiltered` + `getPageLayerTree` |
| `rhwp-studio/src/view/page-renderer.ts` | overlay 인프라 | `applyOverlays` + `getOverlayImages` + `createOverlayLayer` + `OverlayImageInfo` + `collectOverlayImages` |
| `rhwp-studio/src/engine/input-handler-picture.ts` | hit-test 옵션 3-C | `findPictureAtClick` 두 단계 패스 |
| `tests/issue_516.rs` (신규) | 회귀 | 8 tests |

소스 변경 규모: ~+400 / -10 (Rust + TS 통합)

## 검증 결과

### 결정적 검증

| 게이트 | 결과 |
|--------|------|
| `cargo build --lib` | ✅ Finished |
| `cargo build --release` | ✅ Finished |
| `cargo test --lib` | ✅ **1110 passed** (회귀 0) |
| `cargo test --test issue_516` (신규) | ✅ **8 passed** |
| `cargo test --test issue_418/501/514` | ✅ 1 + 1 + 3 |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** |
| `paint::json::tests` (PR #510 호환) | ✅ 4/4 passed |
| `cargo clippy --lib + --test issue_516` | ✅ 0 건 |
| `npx tsc --noEmit` (rhwp-studio) | ✅ 0 errors |
| WASM 빌드 (Docker) | ✅ exit 0, ~4.45 MB |
| `rhwp-studio/public/` 동기화 | ✅ |

### 시각 판정 (작업지시자, 본 사이클 기준)

| 검증 항목 | 결과 |
|----------|------|
| 1. 학교 로고 출력 (PCX → PNG, BehindText overlay) | ✅ 정상 |
| 2. 워터마크 시각 (회색조/투명도) | → 이슈 #535 분리 |
| 3. hit-test 옵션 3-C (텍스트 우선) | ✅ 정상 |
| 부수 — 그림 객체 선택 + 드래그 | ✅ 정상 (보존, 작업지시자 "이건 좋은 듯") |

## 핵심 발견 (본 task 의 가르침)

### 1. 다층 레이어 = HWP/DTP 정체성 인프라

작업지시자 통찰 (Discussion #529 Appendix):
> 아래아한글은 개발 목적이 쿽 익스프레스를 대체하는 것이었으니, 사실 엄밀히 따지면 '워드프로세서' 만을 충족하기 위한 프로젝트가 아니었다.

→ rhwp 의 정체성 = **한국형 DTP 엔진의 오픈소스 재현 + 워드프로세서**. 다층 레이어는 단순 정정이 아니라 **DTP 정체성 인프라**. M200 (v2.0.0) 의 WebGPU 도입 (후보 B) 의 디딤돌.

### 2. 한컴 출력은 정답지가 아니다

메모리 `feedback_pdf_not_authoritative` 의 정확한 사례. 복학원서 엠블럼의 시각:
- IR `b=-50, c=70` 그대로 적용 → 진한 회색 (rhwp 출력)
- 한컴 출력 → 연한 회색 + 흐릿함

같은 IR 값에 대한 다른 시각 결과 → 한컴 시각은 권위 미입증. 작업지시자 정합 ("한컴을 믿으면 안 됨, 자체 해석"). rhwp 자체 시각 해석 권위 정의.

### 3. 편집자 의도 vs IR 데이터 충실 vs 시각 본질

세 정합 사이에서 균형 필요:
- 편집자 의도: 한컴 GUI 에서 본 시각 (한컴 출력)
- IR 데이터 충실: rhwp 가 IR 값을 그대로 적용
- 시각 본질: 워터마크 = 흐릿함 (텍스트 가독성 보장)

본 task 는 IR 데이터 충실 우선. 시각 본질 보장은 분리 task #535 에서 처리.

### 4. renderer 마다 별도 함수 (Stage 5.2 의 가르침)

Task #514 의 발견 ("rhwp 의 image MIME 감지 / 변환 코드는 renderer 마다 별도 사본") 이 본 task 의 다층 레이어 도입에서도 재확인:
- CLI SVG (`svg.rs`) — 단일 평면
- Web Canvas (`web_canvas.rs`) — 단일 평면 → 다층 layer 로 진화
- PageLayerTree JSON (`paint/json.rs`) — Layer 트리 (이미 다층)

각 경로가 별개로 정의되므로 시각 결함 정정 시 모든 경로 점검 필요. 본 task 가 다층 레이어 도입의 명확한 사례.

## 산출물

### 본 task

| 영역 | 산출물 |
|------|--------|
| 수행 계획서 | `mydocs/plans/task_m100_516.md` |
| 구현 계획서 | `mydocs/plans/task_m100_516_impl.md` |
| 단계별 보고서 | `mydocs/working/task_m100_516_stage{1,3,4,5,5_1,5_2}.md` |
| 최종 보고서 | 본 보고서 (`mydocs/report/task_m100_516_report.md`) |
| 기술 조사 (다층 레이어 후보 3 안) | `mydocs/tech/multi_layer_rendering_strategy.md` |

### 외부 공개

| 채널 | 산출물 |
|------|--------|
| GitHub Discussion | [#529](https://github.com/edwardkim/rhwp/discussions/529) — Ideas 카테고리 (다층 레이어 후보 3 안 + DTP 정체성 Appendix) |

### 후속 task

| 이슈 | 분리 영역 |
|------|----------|
| [#535](https://github.com/edwardkim/rhwp/issues/535) | 워터마크 효과 — 회색조/투명도 시각 정합 |

## 메모리 정합

| 메모리 | 본 task 적용 |
|--------|--------------|
| `feedback_process_must_follow` | 이슈 → 브랜치 → 할일 → 계획서 → 단계별 → 보고 절차 준수 |
| `feedback_assign_issue_before_work` | 이슈 #516 + #535 메인테이너 assign |
| `feedback_check_open_prs_first` | 외부 PR 충돌 점검 |
| `feedback_search_troubleshootings_first` | Stage 1 사전 검색 |
| `feedback_hancom_compat_specific_over_general` | preset 분류 (`hancom-watermark` 정확 정합 + `custom` 폴백) |
| `feedback_pdf_not_authoritative` | **본 task 의 핵심 정합** — 한컴 출력 권위 미입증, rhwp 자체 시각 해석 |
| `reference_authoritative_hancom` | 한컴 2010 + 2022 직접 시각 판정 (참고만) |
| `feedback_visual_regression_grows` | Stage 5 시각 판정 필수 게이트 (반복 적용) |
| `feedback_commit_reports_in_branch` | task 브랜치 commit |
| `feedback_close_issue_verify_merged` | close 전 commit devel 머지 검증 |
| `feedback_self_verification_not_hancom` | IR 변경 없음 (라운드트립 보존) |
| **(메모리 후보)** `feedback_image_renderer_paths_separate` | renderer 별 별도 함수 (Task #514 의 발견 + 본 task 재확인) |
| **(메모리 후보)** `project_dtp_identity` | 아래아한글 = QuarkXPress 대체. rhwp = 한국형 DTP 엔진 + 워드프로세서 |
| **(메모리 후보)** `feedback_rhwp_visual_authority` | rhwp 자체 시각 해석 권위 — IR 충실보다 시각 본질 우선 가능 |

## Stage 진행 요약

| Stage | 내용 | 상태 |
|-------|------|------|
| 1 | 본질 진단 (편집자 정의 워터마크 식별) | ✅ |
| 2 | 구현 계획서 (6 영역) | ✅ |
| 3 | 모델 헬퍼 + dump + JSON 구현 | ✅ |
| 4 | Web Canvas CSS filter + 회귀 테스트 + WASM | ✅ |
| 5 | 시각 판정 → 다층 레이어 인프라 도입 결정 (Discussion #529) | ✅ |
| 5.1 | PageLayerTree wrap 분류 (Rust) | ✅ |
| 5.2 | 다층 레이어 (HTML Hybrid) 도입 + hit-test 옵션 3-C | ✅ |
| 5.3+ | 워터마크 시각 정합 → 별도 task #535 | (분리) |
| 6 (현재) | 최종 보고 + merge + push + 이슈 close | ✅ 본 보고서 |

## Stage 진행 결정 사항 (작업지시자)

| 결정 | 영향 |
|------|------|
| 옵션 2 — 본 task Stage 5+ 로 후보 C 직접 도입 | 다층 레이어 인프라 도입 첫 사이클이 됨 |
| 후보 C (HTML Hybrid) → M200 단계적 후보 B (WebGPU) | DTP 정체성 본격화 경로 |
| 결함 2 분리 (이슈 #535) | 본 task 범위 명확화 |
| hit-test 옵션 3-C | 텍스트 우선 + BehindText 객체 선택 보존 |

## 다음 단계

본 task 마무리:

1. WASM 빌드 + studio 동기화 (D-1 제거 후 재빌드)
2. local/task516 → local/devel merge
3. local/devel → devel push
4. 이슈 #516 close (devel 머지 검증 후)
5. 이슈 #535 (분리) 후속 task 시작 (별도 사이클)
