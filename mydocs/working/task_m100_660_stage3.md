# Task #660 Stage 3 — 종합 + 본 작업 1단계 마무리

## 본 작업 1단계 완성 사항

| 영역 | 상태 |
|------|------|
| JSON 스키마 (`ingest_schema_v1.json`) | ✅ 확정 |
| serde 모델 (`src/parser/ingest/schema.rs`) | ✅ 통과 (4 tests) |
| 빌더 골격 (`exam_paper.rs`) | ✅ 통과 (4 tests) |
| CLI 명령 (`rhwp build-from-ingest`) | ✅ 동작 |
| e2e 변환 (sample_minimal.json → HWPX) | ✅ 5,356 bytes 21 문단 |
| WASM 빌드 무영향 | ✅ (parser/ingest는 std::fs 미사용) |
| clippy/build 무경고 | ✅ |

## 본 단계 의도된 한계 (후속 이슈에서 처리)

- **이미지 placeholder** (`[이미지: img/q1.png]`) — #661에서 Picture/BinData IR 빌드 + #182 후 직렬화 활성화
- **placement 4모드** Placement enum 만, 실제 IR 매핑은 #661
- **시험지 표준 ParaShape 풀** — 본 단계는 default(id=0). #661에서 `exam_styles.rs` 신규
- **Skill 측 (Vision 분석)** — #662에서 `.claude/skills/rhwp-exam-ingest/SKILL.md` 신규

## 다음 단계 (병렬 진행)

이슈 #660 완료 후 본 작업 2~4단계 진행:
- **#661 (layout 빌더 placement 4모드)** — 이슈 등록 차단 상태 (권한 시스템). 작업지시자 직접 등록 또는 후속 결정.
- **#662 (Claude Code Skill + helpers)** — 동일.
- **#663 (e2e 시험지 4종, depends on #182)** — 동일.

본 작업 코드 베이스는 이미 #661/#662가 즉시 진입할 수 있는 상태:
- ingest_schema의 `Placement` enum이 4모드 정의됨 → #661은 IR 매핑 함수만 추가
- `build-from-ingest` CLI가 동작 → #662 Skill은 이를 호출하면 끝

## 코드 변경 요약

신규 (10 파일):
- `src/parser/ingest/{mod,schema}.rs`
- `src/document_core/builders/{mod,exam_paper}.rs`
- `tools/rhwp-ingest/schema/{ingest_schema_v1,sample_minimal}.json`
- `mydocs/plans/task_m100_660.md`
- `mydocs/working/task_m100_660_stage{1,2,3}.md`

수정 (5 파일):
- `Cargo.toml` (+1 dep: serde_json)
- `src/parser/mod.rs` (+1 line: ingest 모듈 등록)
- `src/document_core/mod.rs` (+1 line: builders 모듈 등록)
- `src/main.rs` (+~110 lines: build-from-ingest 명령)
- `src/document_core/queries/search_query.rs` (3 lines: vec![] → Vec::<usize>::new() type inference 회피)

총 LOC: 약 +900 (코드 ~600 + 문서 ~300).

## 다음 행동

1. 본 단계 최종 보고서 (`task_m100_660_report.md`)
2. 오늘할일(`orders/20260507.md`) 행 추가
3. `local/task660` 브랜치 커밋
4. 작업지시자 승인 후 #660 close + #661/#662/#663 등록 진행
