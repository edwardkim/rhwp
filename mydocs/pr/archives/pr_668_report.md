# PR #668 처리 보고서

## 메타

| 항목 | 값 |
|------|---|
| PR | [#668](https://github.com/edwardkim/rhwp/pull/668) M100 Task #660: Neumann ingest 파이프라인 v2 |
| 컨트리뷰터 | @metahan88-droid (rhwp 첫 PR) |
| 처리 옵션 | A (PR #629 패턴 정합 — 본질 cherry-pick + 시각 게이트웨이) |
| 처리 결과 | 시각 한계 인정 후 머지 + PR close + Issue #660 close |
| local/devel commit | `80535a3` (author 보존: `한 <han@han-ui-Macmini.local>`) |
| devel merge | `9fc8f32` (--no-ff, push 완료) |
| 처리 일자 | 2026-05-07 |

## 처리 영역 결정 — 옵션 A

검토 보고서 [`pr_668_review.md`](pr_668_review.md)에서 옵션 A/B/C 영역을 분류. 작업지시자가 옵션 A를 승인:
- PR #629 권위 자료 패턴 정합 (본질 cherry-pick + author 보존 + 시각 게이트웨이 + close)
- 거버넌스 산출물 영역(`mydocs/`)은 cherry-pick에서 제외
- 메인테이너 시각 게이트웨이는 WASM 빌드 후 rhwp-studio 웹 에디터에서 sample_minimal.hwpx 시각 확인 영역으로 결정

## cherry-pick 영역 분류

### 포함 영역 (16 파일 +1,352 / -4)

| 영역 | 파일 | 규모 |
|------|------|------|
| 신규 파서 | `src/parser/ingest/{mod,schema}.rs` | +223 |
| 신규 빌더 | `src/document_core/builders/{mod,exam_paper}.rs` | +374 |
| 신규 CLI | `src/main.rs` build-from-ingest | +108 |
| 신규 스키마 | `tools/rhwp-ingest/schema/{ingest_schema_v1,sample_minimal}.json` | +171 |
| Skill 인프라 | `.claude/skills/rhwp-exam-ingest/SKILL.md` + helpers 4종 | +467 |
| Cargo.toml | serde_json = "1" 추가 | +1 |
| 부수 변경 | `src/document_core/mod.rs`, `src/parser/mod.rs` (모듈 등록) | +2 |
| 부수 변경 | `src/document_core/queries/search_query.rs` Vec 타입 명시 | ±3 |
| 부수 변경 | `.gitignore` `.claude/skills/` negate | ±3 |

### 제외 영역 (7 파일, PR #629 패턴 정합)

| 파일 | 분류 |
|------|------|
| `mydocs/plans/task_m100_660.md` | 컨트리뷰터 수행계획서 |
| `mydocs/working/task_m100_660_stage1.md` | 컨트리뷰터 단계별 보고서 1 |
| `mydocs/working/task_m100_660_stage2.md` | 단계별 보고서 2 |
| `mydocs/working/task_m100_660_stage3.md` | 단계별 보고서 3 |
| `mydocs/working/task_m100_660_e2e.md` | e2e 검증 보고서 |
| `mydocs/working/task_m100_660_v2.md` | v2 redesign 보고서 |
| `mydocs/report/task_m100_660_report.md` | 컨트리뷰터 최종 보고서 |

본 영역은 PR #629에서도 동일하게 cherry-pick에서 제외했던 패턴(메인테이너 산출물 영역과 외부 컨트리뷰터 산출물 영역의 분리). 컨트리뷰터의 거버넌스 정합 작업 자체는 인정하되, 메인테이너 산출물 영역에 영구 보존하지 않음.

## 결정적 재검증 (본 환경)

| 검증 | 결과 |
|---|---|
| `cargo test --release --lib` | **1155 passed**, 0 failed (신규 parser::ingest 4 + builders 10 포함) |
| `cargo test --release --lib parser::ingest` | 4 passed |
| `cargo test --release --lib document_core::builders` | 10 passed |
| `cargo test --release` (전체 workspace) | 모두 GREEN |
| `cargo clippy --release --lib` | 신규 경고 0 |
| `cargo build --release` | 무에러 |

PR #602 처리 시점 베이스라인(1140 passed) 대비 **+15** (parser::ingest 4 + builders 10 + α). 회귀 0.

## 회귀 sweep (대표 fixture 5종, 195 페이지)

| fixture | 페이지 | 결과 |
|---|---|---|
| samples/aift.hwp | 77 | 정상 |
| samples/exam_kor.hwp | 20 | 정상 |
| samples/exam_math.hwp | 20 | 정상 |
| samples/exam_science.hwp | 4 | 정상 |
| samples/hwpx/aift.hwpx | 74 | 정상 |

본 PR은 신규 모듈만 추가하고 기존 경로 변경은 search_query.rs 3 라인(테스트 타입 명시) + 모듈 등록 2 라인이라 회귀 위험은 구조적으로 낮음.

## e2e 라운드트립

```
$ rhwp build-from-ingest tools/rhwp-ingest/schema/sample_minimal.json -o output/pr668/sample_minimal.hwpx
저장 완료: output/pr668/sample_minimal.hwpx (5356바이트, 문제 3개, 문단 21개)

$ rhwp export-text output/pr668/sample_minimal.hwpx
1. 다음 글의 주제로 가장 적절한 것은?
환경 오염은 현대 사회의 중요한 문제 중 하나이다. ...
① 환경 보호의 중요성을 강조하는 글
② 도시 생활의 편리함을 설명하는 글
... ⑤
2. 다음 그래프에서 알 수 있는 사실로 적절한 것은?
... ⑤ 2022년에는 전년 대비 감소했다
3. 다음 글의 흐름으로 가장 적절한 것은?
...
```

PR 본문 명시 영역과 일치 — HWPX 5,356 bytes, 문제 3개 / 문단 21개, ① ~ ⑤ 평문 보존.

## Docker WASM 빌드

- `pkg/rhwp_bg.wasm`: **4,598,886 bytes** (4.6M, v0.7.10 + serde_json + parser/ingest + builders + main.rs CLI)
- PR #629 시점 4,590,307 bytes 대비 **+8,579 bytes** (parser/ingest + builders + CLI 추가 정합)
- `rhwp-vscode/media/` 동기화 완료

## 메인테이너 시각 게이트웨이 (rhwp-studio 웹 에디터)

http://localhost:7700에서 `output/pr668/sample_minimal.hwpx` 시각 확인.

### 발견 사항: SVG/WASM 렌더 불일치

**현상**: 1번 문제 두 번째 줄("환경 오염은 ... 영향을 미친다.")이 SVG 렌더에서는 1줄로 출력되지만 WASM 렌더에서는 본문 폭(~150mm)에 맞춰 2줄로 줄바꿈.

**원인** (`src/document_core/builders/exam_paper.rs:97-106` `make_text_para()` 헬퍼):
- 모든 문단에 단일 LineSeg(`segment_width=50000`, `text_start=0`, `tag=0x00060000`)만 부여
- 실제 줄나눔(line breaking)을 빌더가 사전 계산하지 않음
- SVG 렌더(`export-svg`): 단일 LineSeg를 신뢰하여 한 줄로 그림
- WASM 렌더(`rhwp-studio`): 자체 layout으로 본문 폭에 맞춰 다시 계산 → 두 줄

본 한계는 PR 본문에서 컨트리뷰터가 명시한 영역(이미지 placeholder, ParaShape default(id=0))과 동일 범주의 골격 단계 한계. PR 본문에 명시되지 않은 추가 한계로, 후속 이슈에 보강 안내.

### 작업지시자 시각 판정 결과

★ **머지 통과** — 본 PR은 Neumann 본 작업 1단계 골격 영역이고, 시각 정합성은 후속 이슈 #665 (placement 4모드 + ParaShape 시험지 표준 + Picture/BinData IR 빌드)에서 본격 해결.

### 머지 결과

- `git merge local/devel --no-ff` → devel `9fc8f32`
- `git push origin devel` 정합
- PR #668 close + 컨트리뷰터 안내 코멘트 추가 (PR #629 패턴 정합)
- Issue #660 close + 머지 안내 코멘트

## 후속 영역

### 컨트리뷰터 등록 후속 이슈 #665/#666/#667 영역 (처리 완료)

컨트리뷰터가 PR 생성 직전(07:42-07:43)에 메인테이너 레포에 등록한 후속 이슈 3건:
- #665 "Task #661: placement 4모드 IR 매핑 + ParaShape 시험지 표준 + Picture/BinData IR 빌드"
- #666 "Task #663: e2e 시험지 4종 라운드트립 검증"
- #667 "Task #664: ingest 스키마 진화 — passage_groups, boxed, page footer"

**작업지시자 결정**:
- 마일스톤: **v1.0.0** (#1) 일괄 지정
- assignee: **@metahan88-droid** 일괄 지정
- 안내 코멘트: 각 이슈에 PR #668 처리 결과 + LineSeg 영역 보강 권유 등록

본 영역은 컨트리뷰터가 이어서 진행하는 후속 PR 영역으로 통합. 컨트리뷰터의 거버넌스 정합 영역(PR #668에서 입증)을 신뢰하여 메인테이너가 직접 작업하지 않고 후속 PR을 기다리는 영역.

### 컨트리뷰터에게 안내할 영역

- PR 머지 + cherry-pick 사실 + author 보존 사실 안내
- "rhwp 첫 PR" 환영 + 거버넌스 정합 영역(하이퍼-워터폴 단계별 산출물 + AI 페어프로그래밍 + 자가 검증/수정) 인정
- mydocs 영역 cherry-pick 제외 사유(메인테이너 산출물 영역 분리 정책, PR #629 패턴 정합)
- 후속 이슈 #665/#666/#667 영역의 메인테이너 권한 검토 진행 안내

## 본 PR의 본질

Neumann ingest 파이프라인 — 외부 입력(이미지/PDF/MD/DOCX) → JSON → HWPX 변환 — 의 본 작업 1단계 인도물:
- JSON Schema v7 정의(auto_number 명시 필드 — v2 redesign에서 휴리스틱 제거)
- Rust serde 모델 (IngestDocument/Question/StemBlock/Choice/Media/Placement)
- Document IR 변환 빌더
- `rhwp build-from-ingest` 신규 CLI 명령
- Claude Code Skill (rhwp-exam-ingest, helpers 4종 — pdf_to_pngs/crop_image/extract_docx/check_deps)

본 PR은 v0.7.10 베이스라인에서 신규 영역만 추가하고 기존 경로 변경은 최소(부수 5 라인). 회귀 위험은 구조적으로 낮음.

## 거버넌스 정합 영역 평가

PR #629 권위 자료 패턴과 비교한 거버넌스 정합도:

| 영역 | PR #629 (@planet6897) | PR #668 (@metahan88-droid) |
|------|---|---|
| 하이퍼-워터폴 단계별 산출물 | stage1, stage2 | stage1, stage2, stage3, e2e, v2 (더 상세) |
| AI 페어프로그래밍 명시 | Co-Authored-By Claude | Co-Authored-By Claude Opus 4.7 |
| 자가 검증 | 회귀 sweep + cargo test 1134 | e2e + cargo test 14 |
| 자가 수정 | (단일 인도물) | e2e hotfix + v2 redesign (자가 수정 영역 우수) |
| 코드 품질 | feedback_rule_not_heuristic 정합 | v2 redesign에서 휴리스틱 제거(rule_not_heuristic 정합) |
| 회귀 위험 | 낮음 | 낮음 (신규 영역만 추가) |

본 PR은 PR #629 권위 자료 영역과 동등 또는 더 우수한 거버넌스 정합. **rhwp 첫 PR로서 매우 인상적인 사례.**
