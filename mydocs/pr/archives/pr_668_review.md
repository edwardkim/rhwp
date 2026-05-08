# PR #668 검토

## 메타

| 항목 | 값 |
|------|---|
| PR 번호 | [#668](https://github.com/edwardkim/rhwp/pull/668) |
| 제목 | M100 Task #660: Neumann ingest 파이프라인 v2 (스키마 + 빌더 + Skill + e2e + auto_number redesign) |
| 컨트리뷰터 | @metahan88-droid (GitHub 가입 2025-09-29, **rhwp 첫 PR**) |
| Fork | metahan88-droid/rhwp (`m100-task660-neumann-v2`) |
| Base | edwardkim/rhwp `devel` |
| 규모 | +1986 / -4, 24 files (대규모) |
| 커밋 수 | 6 (본 작업 4 + merge 2) |
| 커밋 author | `한 <han@han-ui-Macmini.local>` + Co-Authored-By Claude Opus 4.7 |
| mergeable | MERGEABLE이지만 mergeStateStatus=BEHIND |
| CI | no checks reported |
| 연결 이슈 | #660 (closes), #654 spike, #662 Skill, #665/#666/#667 후속 |
| 생성 | 2026-05-07 07:47 |

## 본 PR 평가의 본질적 시각 — 거버넌스 정책

본 프로젝트는 **하이퍼-워터폴 방법론 + AI 페어프로그래밍 거버넌스**가 본 영역의 운영 정책. **외부 컨트리뷰터도 본 방법론을 따르는 것이 정합 영역**.

본 영역의 권위 자료: **PR #629** (@planet6897 Jaeook Ryu, closed 2026-05-06, "본질 cherry-pick" 패턴).

## 권위 자료 영역 — PR #629 (본질적 비교 영역)

본 PR의 본질적 평가는 PR #629 패턴과 본질적으로 비교.

### PR #629 영역

| 항목 | 값 |
|------|---|
| 컨트리뷰터 | @planet6897 (Jaeook Ryu) |
| 인도물 | Task #628 — nested cell inline_shape_positions 키 충돌 정정 |
| 코드 영역 | src 7 파일 +62/-28 |
| 거버넌스 산출물 영역 | mydocs 6 파일 (orders + plans + plans/impl + report + working/stage1 + working/stage2) |
| AI 페어프로그래밍 | Co-Authored-By Claude Opus 4.7 (1M context) |
| 메인테이너 처리 | 본질 cherry-pick (src만, mydocs 영역 제외), author 보존, PR close |
| 시각 정답지 게이트웨이 | exam_science page 4 시각 통과 ★ |
| 회귀 sweep | 164 fixture / 1,684 페이지 / 차이 0 |

### PR #629 → PR #668 본질적 비교

| 영역 | PR #629 (권위 자료) | PR #668 (본 PR) | 본질 비교 |
|------|------|------|------|
| 거버넌스 산출물 영역 | mydocs 6 파일 (orders/plans/plans-impl/report/stage1/stage2) | mydocs 7 파일 (plans/stage1/stage2/stage3/e2e/v2/report) | **정합** (둘 다 거버넌스 산출물 영역 포함) |
| AI 페어프로그래밍 | Co-Authored-By Claude | Co-Authored-By Claude Opus 4.7 | **정합** |
| 단계별 보고서 | stage1, stage2 | stage1, stage2, stage3, e2e, v2 | **정합** (PR #668 더 상세) |
| 자가 검증 영역 | 회귀 sweep + cargo test 1134 | e2e + cargo test 14 | **정합** |
| 자가 수정 영역 | (PR 단일 인도물) | e2e hotfix + v2 redesign | **정합** (자가 수정 영역 우수) |
| 코드 영역 규모 | src +62/-28 | src/parser/ingest + src/document_core/builders + main.rs +900 | **PR #668이 본질적 신규 영역** |
| 인도물 본질 | 기존 영역 결함 정정 | 신규 영역 도입 | **본질적 차이** |
| 인도물 영역 위치 | 기존 영역 (renderer/layout) | 신규 영역 (parser/ingest, document_core/builders, .claude/skills) | **본질적 차이** |
| 시각 정답지 권위 | 기존 sample (exam_science) | 신규 sample (수능 PDF) | **본질적 차이** |

**본질적 결론**:
- PR #629의 거버넌스 정합 영역(mydocs 산출물 + AI 페어프로그래밍 + 단계별 보고서)은 본 PR이 **정합 본질로 준수**
- PR #629의 "본질 cherry-pick" 패턴(src만 cherry-pick, mydocs 제외)은 본 PR에 **정합 적용 가능**
- 단, PR #668은 **본질적 신규 영역 도입**(parser/ingest + document_core/builders + .claude/skills)이라 PR #629(기존 영역 결함 정정)와 본질적으로 다른 영역

본 영역은 PR #668의 본질적 검토 영역(아키텍처 신규 영역 + Skill 영역)이 **PR #629의 권위 자료 영역에 부재**한 영역이므로, 본 영역은 메인테이너 정책 결정 영역에 본질적으로 위치.

## rhwp 첫 PR 컨트리뷰터 — 환영 + 거버넌스 정합 영역 평가

`@metahan88-droid` 님은 본 PR이 **rhwp 레포 첫 PR**이다. 외부 입력(이미지/PDF/MD/DOCX) → HWPX 시험문제 변환 파이프라인(Neumann) 본 작업 1단계라는 야심찬 인도물 + JSON 스키마 + Rust serde 모델 + 빌더 + CLI + Claude Code Skill + 실제 수능 PDF 라운드트립 e2e + redesign v2까지 단일 PR에 묶어 제출.

본 인도물의 **거버넌스 정합 영역**은 매우 우수 (PR #629 권위 영역 정합):
- 하이퍼-워터폴 단계별 산출물 영역(수행계획서 + 단계별 보고서 + e2e 보고서 + v2 보고서 + 최종 보고서) **완비**
- AI 페어프로그래밍 영역(Claude Opus 4.7 Co-Authored-By) **명시**
- 자가 검증 + 자가 수정 영역(e2e hotfix 2건 + v2 redesign) **모범 사례** (PR #629 영역 본질적 진보)
- 회귀 테스트 영역(cargo test 14/14, 실제 수능 PDF 라운드트립 e2e) **고품질**

본 영역은 **거버넌스 정합 외부 컨트리뷰터의 모범 사례**로 평가.

## 인도물 영역

### 코드 영역 (cherry-pick 본질 영역)

| 영역 | 파일 | 규모 | 평가 |
|------|------|------|------|
| 스키마 정의 | `tools/rhwp-ingest/schema/ingest_schema_v1.json` | 118L | JSON Schema v7 준수, auto_number 명시 필드 |
| serde 모델 | `src/parser/ingest/{mod,schema}.rs` | 223L | IngestDocument/Question/Choice/Media/Placement |
| 빌더 | `src/document_core/builders/{mod,exam_paper}.rs` | 374L | IngestDocument → Document IR 변환 |
| CLI | `src/main.rs` build-from-ingest | +108L | media-dir + -o 인자 |
| 회귀 테스트 | parser::ingest 4 + builders 10 | 14/14 | cargo test --release --lib 통과 |
| Cargo.toml | serde_json = "1" 추가 | +1L | 신규 의존성 |
| 부수 변경 | search_query.rs Vec type 명시 | ±3L | serde_json Vec 타입 충돌 회피 |
| sample | sample_minimal.json (3 문제) | 53L | 회귀 sample |

### Claude Code Skill 영역 (별도 검토 영역)

| 영역 | 파일 | 규모 | 평가 |
|------|------|------|------|
| Skill 정의 | `.claude/skills/rhwp-exam-ingest/SKILL.md` | 234L | 5단계 워크플로우 |
| pdf_to_pngs.sh | helpers | 52L | PDF → PNG (e2e hotfix 정정) |
| crop_image.sh | helpers | 42L | bbox crop |
| extract_docx.py | helpers | 71L | DOCX → text + 이미지 |
| check_deps.sh | helpers | 68L | 의존성 점검 |
| `.gitignore` | `.claude/*` + `!.claude/skills/` | +3/-1 | 영역 트래킹 |

본 영역은 PR #629에 **부재**한 영역. AI 페어프로그래밍 거버넌스 영역의 **신규 인도물**이며, 본 영역의 본질적 정합은 메인테이너 정책 결정 영역.

### 거버넌스 산출물 영역 (cherry-pick 제외 본질 영역)

| 파일 | 거버넌스 영역 |
|------|----|
| `mydocs/plans/task_m100_660.md` | 수행 계획서 (하이퍼-워터폴 1단계) |
| `mydocs/working/task_m100_660_stage1.md` | 단계별 보고서 1 (하이퍼-워터폴 단계 보고) |
| `mydocs/working/task_m100_660_stage2.md` | 단계별 보고서 2 |
| `mydocs/working/task_m100_660_stage3.md` | 단계별 보고서 3 |
| `mydocs/working/task_m100_660_e2e.md` | e2e 검증 보고서 (자가 검증 영역) |
| `mydocs/working/task_m100_660_v2.md` | v2 redesign 보고서 (자가 수정 영역) |
| `mydocs/report/task_m100_660_report.md` | 최종 보고서 (하이퍼-워터폴 종결) |

본 영역은 **하이퍼-워터폴 거버넌스 정합 영역**이며, **PR #629 권위 영역에서 cherry-pick 제외 영역**. 본 PR의 본질적 cherry-pick 시 본 영역은 본질적으로 제외.

### v2 redesign 품질 (자가 수정 영역)

본 PR의 v2 redesign(`29ae304`)에서 Question에 `auto_number: bool` 명시 필드를 도입하고 빌더 휴리스틱을 완전 제거. 회귀 14/14 무에러, 실제 수능 PDF 라운드트립 v1과 동일 출력 입증. 본 redesign 자체는 **AI 페어프로그래밍 거버넌스 + 코드 리뷰 권고 반영의 모범 사례** + **`feedback_rule_not_heuristic` 정합 패턴**.

### 발견 결함 + 자가 수정 (자가 검증 영역)

`d7a8dc4` (e2e hotfix)에서 컨트리뷰터 본인이 e2e 검증 중 발견한 결함 2건을 즉시 정정:
- `pdf_to_pngs.sh`: pdftoppm leading-zero 출력(`page-08.png`)을 `printf "%03d"`가 8진수로 해석해 8페이지 이후 변환 실패 → 10진수 강제(`$((10#$n))`)로 정정
- `exam_paper.rs`: 첫 stem_block에 무조건 `{n}.` prefix 추가 로직이 사용자 명시 prefix 충돌 → `apply_number_prefix()` 헬퍼로 정정 + 회귀 테스트 2건 추가

본 자가 검증 + 자가 수정 능력은 **고품질 컨트리뷰터의 표지** + **하이퍼-워터폴 정합 영역**.

## 본질적 검토 영역 (PR #629 권위 영역 부재 영역)

PR #629 권위 영역 정합 영역(거버넌스 산출물, AI 페어프로그래밍, cherry-pick 패턴)은 본질적 정합. 그러나 PR #668의 본질적 신규 영역은 PR #629에 부재한 영역이므로 별도 메인테이너 검토 영역:

### 검토 영역 1: 외부 컨트리뷰터의 후속 이슈 등록 영역

컨트리뷰터가 PR 생성(07:47) **직전**에 메인테이너 레포에 후속 이슈 3건을 직접 등록:
- Issue #665 "Task #661: placement 4모드 IR 매핑..." (07:42)
- Issue #666 "Task #663: e2e 시험지 4종 라운드트립 검증..." (07:43)
- Issue #667 "Task #664: ingest 스키마 진화..." (07:43)
- Issue #654 "외부 입력 사전 검증 spike" (이전)
- Issue #660 "Neumann 본 작업 1: JSON 스키마 + Rust 빌더 골격" (이전)

GitHub Issues는 누구나 등록 가능 영역이지만, **rhwp 마일스톤(M100=v1.0.0) 분류는 메인테이너 인도물 정책 영역**. 본 컨트리뷰터의 등록 이슈는 모두 milestone 미지정 상태이므로, 메인테이너 검토 후 분류가 본 영역의 본질적 진행.

**검토 영역**: 본 후속 이슈 영역의 본질적 분류 + 권한 검토 영역 + 마일스톤 정합.

### 검토 영역 2: `.claude/` 워크플로우 인프라 영역

PR이 `.gitignore`를 수정해 `.claude/skills/`를 트래킹 대상으로 포함:
```diff
- .claude/
+ .claude/*
+ !.claude/skills/
```

`.claude/` 영역은 본 레포에서 본질적으로:
- `.claude/projects/.../memory/` — 메인테이너 + Claude 본인 영역 (auto-memory 영역, **로컬 영역**, git 트래킹 부재)
- `.claude/skills/` — 본 PR이 신규 도입하는 영역

**검토 영역**:
- `.claude/skills/`이 본 레포의 본질적 영역인지?
- 또는 본 영역을 `tools/rhwp-skills/` 또는 별도 영역으로 분리?
- `.gitignore` 정책의 본질적 영역 — `.claude/` 일괄 제외 영역을 negate하는 본질적 정합 영역?

본 영역은 메인테이너 정책 결정 영역. AI 페어프로그래밍 거버넌스 시각에서 Claude Code Skill 영역을 본 레포에 본질적으로 포함하는 것은 **거버넌스 인프라 영역의 본질적 인도물**로 평가 가능.

### 검토 영역 3: 본체 아키텍처 신규 영역의 본질적 위치

본 PR이 신규로 도입하는 인도물:
- `tools/rhwp-ingest/schema/ingest_schema_v1.json` — **외부 입력 → rhwp 본체 인터페이스 정의**
- `src/parser/ingest/` — 신규 파서 영역
- `src/document_core/builders/` — 신규 빌더 영역
- `rhwp build-from-ingest` — 신규 CLI 명령

`CLAUDE.md` 영역 — 파일 포맷별 파서 구조: HWPX/HWP5/HWP3는 `Document` IR로 변환. **`ingest`는 본 분류에 부재** (외부 입력은 HWP 포맷이 아닌 외부 영역).

**검토 영역**:
- 본 영역의 본질적 위치 — `parser/ingest/`가 정합 영역인지? (`parser/`는 HWP 포맷 파서 영역) 또는 `src/ingest/` 별도 최상위 영역?
- `document_core/builders/` 영역의 본질적 정합?
- CLI 명령 명명(`build-from-ingest`)의 본질적 정합?

본 영역은 메인테이너 아키텍처 정책 권한.

### 검토 영역 4: 시각 정답지 게이트웨이 영역 (본질)

PR #629 권위 영역 — 메인테이너 시각 판정 ★ 통과 (exam_science page 4). 본 PR도 본질적으로 시각 정답지 게이트웨이 본질.

본 PR의 인도물(시험지 변환 파이프라인)은 **한컴오피스 시험지 정합성**이 본질이지만, 게이트웨이 검증에 사용된 자료가:
- `samples/2010-exam_kor.pdf` — 본 레포에 포함된 권위 자료 PDF (컨트리뷰터가 추가)
- `tools/rhwp-ingest/schema/sample_minimal.json` — 컨트리뷰터 작성 3문제 샘플
- 한컴오피스 시각 정합성 검증 — **부재**

`feedback_visual_judgment_authority.md` 영역 — 한컴 2022 정답지, Claude는 정량 측정만 보조. 본 PR의 e2e 검증은 정량 측정만 수행, 시각 정답지 권위 영역 부재.

**검토 영역**:
- 본 PR이 생성하는 HWPX(`samples/2010-exam_kor.pdf` 페이지 1·2 변환 결과)를 한컴 2022로 열었을 때의 **시각 품질**이 시험지로서 정합 영역인지?
- 본 영역의 시각 검증은 **메인테이너 시각 권한 영역** (Claude는 정량 보조).

본 영역은 본 PR의 게이트웨이 검증 본질. 머지 가부의 본질적 영역.

### 검토 영역 5: 단일 PR 묶음의 본질적 검토 영역

본 PR은 6 커밋 = 4 본 작업 + 2 merge로:
1. Task #660 본 작업 (045ac58): JSON 스키마 + serde + 빌더 + CLI
2. Task #662 선행 (7112183): Claude Code Skill + 4 helpers
3. Task #660 e2e hotfix (d7a8dc4): pdf_to_pngs + 빌더 prefix 결함 정정
4. Task #660 v2 (29ae304): auto_number 명시 필드 도입

본 4건은 본질적으로:
- (1) Rust 본체 인도물 (Task #660)
- (2) Claude Code Skill 인프라 (Task #662, 본질적으로 다른 task)
- (3) (1)의 e2e 정정 (Task #660 후속)
- (4) (1)의 redesign (Task #660 v2)

**검토 영역**:
- 본 단일 PR 묶음이 본질적으로 정합 영역인지? cherry-pick 시 본질적으로 분리 진행 가능 (메인테이너 권한 영역)
- 거버넌스 시각: 단일 PR 묶음이 하이퍼-워터폴 단계 보고 영역과 본질적으로 정합한지?

PR #629 권위 영역 — 단일 task(#628), 단일 PR. 본 영역은 PR #668의 본질적 영역(Task #660 + Task #662 동시 포함)과 본질적으로 다른 영역. 본 PR의 cherry-pick 본질은 task 단위 분리 영역으로 진행 가능.

### 검토 영역 6: Claude Code Skill 의존성 영역

본 PR이 추가하는 `.claude/skills/rhwp-exam-ingest/SKILL.md`는:
- Claude Code Skill 영역에 의존 (Anthropic 영역)
- "Read tool로 시험지 PNG 직접 보고 자연어 분석" 영역 — Claude 본인 vision 영역
- "한국어 시험지 layout 의미적 이해" 영역 — Claude 본인 LLM 영역

본 Skill의 본질적 의존성:
- Anthropic Claude Code 환경
- ImageMagick(brew install) 의존
- python-docx 의존
- pdftoppm/pdftotext 의존

**검토 영역**:
- 본 Skill을 rhwp 레포에 본질적으로 포함하는 것이 정합 영역인지?
- 거버넌스 시각: AI 페어프로그래밍 거버넌스의 일환으로 본 Skill을 레포에 본질적으로 포함하는 것이 정합 영역?
- 본 Skill 영역의 의존성 정책(ImageMagick 등 외부 영역 의존)이 rhwp 의존성 정책과 정합?

본 영역은 메인테이너 정책 결정 영역. AI 페어프로그래밍 거버넌스 시각에서 본 Skill 영역은 **거버넌스 정합 인도물**로 본질적 평가 가능.

## 게이트웨이 검증 영역 (보류)

본 PR이 BEHIND 상태(devel 추격 필요). 시각 정답지 게이트웨이(검토 영역 4) 영역이 본질적으로 부재한 상태로 머지 시 영구 인도물로 보존되는 영역. **머지 검증 영역은 검토 영역 1~6 영역 중 본질적 게이트웨이 통과 후 진행**.

## 옵션 분류

### 옵션 A: PR #629 권위 영역 정합 — 본질 cherry-pick + 메인테이너 시각 정답지 게이트웨이 통과

본 옵션은 PR #629 패턴 정합. 컨트리뷰터의 거버넌스 정합 영역을 인정하면서, 메인테이너가 cherry-pick으로 src/.claude 영역만 머지(mydocs 제외).

**본 옵션 처리 영역**:
1. 메인테이너가 `local/devel`에서 cherry-pick (4 본 작업 커밋, author email `한 <han@han-ui-Macmini.local>` 보존):
   - 045ac58 (Task #660 본 작업): src/parser/ingest + src/document_core/builders + main.rs + tools/rhwp-ingest + Cargo.toml + search_query.rs
   - 7112183 (Task #662 Skill 선행): .claude/skills + .gitignore (검토 영역 2 본질 영역 — Skill 영역 본 레포 포함 여부 결정 영역)
   - d7a8dc4 (Task #660 e2e hotfix): pdf_to_pngs.sh 정정 + exam_paper.rs apply_number_prefix
   - 29ae304 (Task #660 v2 redesign): auto_number 명시 필드 도입
2. cherry-pick 시 mydocs 영역 본질적 제외 (PR #629 패턴):
   - `mydocs/plans/task_m100_660.md` 제외
   - `mydocs/working/task_m100_660_stage{1,2,3}.md` 제외
   - `mydocs/working/task_m100_660_e2e.md` 제외
   - `mydocs/working/task_m100_660_v2.md` 제외
   - `mydocs/report/task_m100_660_report.md` 제외
   - (만약 PR에 `mydocs/orders/` 변경 포함 시) 제외
3. cargo test --release --lib (parser::ingest 4 + builders 10 = 14/14)
4. cargo build --release 무에러
5. 광범위 회귀 sweep (PR #629 패턴 정합):
   - 기존 fixture 회귀 sweep — 회귀 0 영역 확인
6. e2e 라운드트립 영역:
   - `rhwp build-from-ingest tools/rhwp-ingest/schema/sample_minimal.json -o output/sample_minimal.hwpx`
   - 본 HWPX를 한컴 2022로 열어 **시각 정답지 게이트웨이** 통과 확인 (메인테이너 시각 권한 영역)
7. 메인테이너 시각 판정 통과 시 devel 머지 + push
8. PR #668 close + 처리 결과 + 정량 측정 + 시각 판정 + 거버넌스 정합 평가 코멘트 (PR #629 본질 패턴)
9. 후속 이슈 #665/#666/#667 영역 검토 (검토 영역 1) — 마일스톤 분류 또는 close 결정
10. Task #660 (closes #660 자동 처리)

**본 옵션의 본질**: PR #629 권위 영역 정합 + 시각 정답지 게이트웨이 통과 + 거버넌스 정합 영역 인정.

### 옵션 B: 옵션 A의 본질적 분리 — Task #660 본체만 cherry-pick + Task #662 Skill 별도 PR 권유

본 옵션은 검토 영역 5(단일 PR 묶음) 영역의 본질적 분리. Task #660 본체만 머지하고, Task #662 Skill 영역은 별도 PR로 분리.

**본 옵션 처리 영역**:
1. 메인테이너가 `local/devel`에서 cherry-pick (3 본 작업 커밋, Skill 제외):
   - 045ac58 (Task #660 본 작업): Skill 변경 부분(`.gitignore`)만 본질적 제외하고 Rust 본체 영역 cherry-pick
   - d7a8dc4 (Task #660 e2e hotfix): exam_paper.rs apply_number_prefix만 cherry-pick (pdf_to_pngs.sh는 Skill 영역 → 제외)
   - 29ae304 (Task #660 v2 redesign): auto_number 명시 필드 도입
2. cherry-pick 시 mydocs + .claude/skills 영역 본질적 제외
3. (옵션 A의 단계 3~7과 동일) 시각 정답지 게이트웨이 통과 후 머지
4. PR #668 close + Task #662 Skill 영역 별도 PR 재제출 권유 코멘트
5. 후속 이슈 #665/#666/#667 영역 검토

**본 옵션의 본질적 우려**: 컨트리뷰터의 본질적 인도물(거버넌스 정합 단일 PR)을 본질적으로 분리하는 영역. 본 분리가 본질적으로 필요한 영역인지 메인테이너 정책 결정 영역.

### 옵션 C: 본질적 협의 + 검토 영역 1~6 본질적 합의 후 진행

본 옵션은 본 PR이 본질적으로 다루는 영역(아키텍처 신규 영역 + Skill 영역)이 메인테이너 정책 결정 영역에 본질적으로 위치하므로, 컨트리뷰터와 본질적 협의 후 진행.

**본 옵션 처리 영역**:
1. PR #668 OPEN 유지 + 본질적 협의 코멘트
2. 검토 영역 1~6 영역의 본질적 합의:
   - 영역 1: 후속 이슈 분류 권한
   - 영역 2: `.claude/` 인프라 영역 + Skill 본 레포 포함 여부
   - 영역 3: 본체 아키텍처 신규 영역 위치 (`parser/ingest` vs `src/ingest`)
   - 영역 4: 시각 정답지 게이트웨이 본질
   - 영역 5: 단일 PR 묶음 분리
   - 영역 6: Skill 의존성 정책
3. 합의 후 PR 진행 또는 분리/재정의

**본 옵션의 본질적 우려**: 본 영역의 본질적 협의가 본 PR을 본질적으로 지연. 컨트리뷰터의 첫 PR 영역이라 협의 부담이 본질적 영역. PR #629 권위 영역에서는 본질적 협의 영역 부재(즉시 처리). 본 옵션은 본 PR의 본질적 신규 영역(아키텍처 + Skill)이 PR #629에 부재하므로 본질적으로 다른 영역.

## 메인테이너 게이트웨이 시각

본 PR의 본질적 영역:
- **코드 영역**: 고품질 (cargo test 14/14, 실제 PDF e2e, v2 redesign, 자가 검증)
- **거버넌스 정합 영역**: 매우 우수 (PR #629 권위 영역 정합 — 하이퍼-워터폴 + AI 페어프로그래밍 + 자가 수정)
- **PR #629 권위 영역 정합 영역**: 본질 cherry-pick 패턴 본질적 정합
- **본질적 신규 영역**: 검토 영역 1~6 (PR #629 권위 영역 부재)

거버넌스 시각에서 본 PR은 **외부 컨트리뷰터의 거버넌스 정합 모범 사례**. 본 영역의 본질적 정합 인정 + 시각 정답지 게이트웨이(검토 영역 4) 통과 후 머지가 본 영역의 본질적 진행.

**메인테이너 권고**: **옵션 A** (PR #629 권위 영역 정합 — 본질 cherry-pick + 시각 정답지 게이트웨이 통과) — 작업지시자 결정 영역.

본 옵션의 본질:
1. PR #629 권위 영역 정합으로 본질적 진행 (단일 패턴 영역)
2. 본질적 신규 영역(검토 영역 2, 3, 6)은 cherry-pick 시점에 메인테이너가 본질적 결정 (Skill 본 레포 포함 / 아키텍처 위치 / 의존성 정책)
3. 후속 이슈 영역(검토 영역 1)은 머지 후 영역 별도 정리
4. 시각 정답지 게이트웨이(검토 영역 4) 통과 본질이 머지 가부 본질
5. 단일 PR 묶음(검토 영역 5)은 cherry-pick 시 본질적 영역 분리 가능 (메인테이너 권한)

## 권위 자료 영역

본 PR이 본질적으로 다루는 인도물(시험지 변환 파이프라인)의 시각 정답지 영역:
- 권위 자료: `samples/2010-exam_kor.pdf` (수능 PDF, 본 PR이 컨트리뷰터가 추가)
- 본 PDF의 페이지 1·2 → ingest JSON → HWPX 변환 결과의 한컴 2022 시각 정답지 영역 — **메인테이너 시각 권한 영역**

본 영역의 본질적 정답지(한컴 2022로 본 시험지 HWPX를 열었을 때의 시각 품질)가 본 PR의 e2e 검증 영역에 본질적으로 포함되지 않음. 본 영역의 본질적 검증은 **메인테이너 시각 권한 영역**.

PR #629 권위 영역 — exam_science page 4 시각 통과 ★ 패턴. 본 PR도 본질적으로 시각 정답지 게이트웨이 통과 본질.

## 결론

본 PR은:
- **코드 영역**: 고품질, 인상적 (cargo test 14/14, 실제 PDF e2e, v2 redesign, 자가 검증)
- **거버넌스 정합 영역**: 우수 (PR #629 권위 영역 정합)
- **본질적 신규 영역**: 검토 영역 1~6 (메인테이너 권한 영역, PR #629 부재)

**처리 권고**: **옵션 A** (PR #629 패턴 정합 — 본질 cherry-pick + 시각 정답지 게이트웨이 통과) — 작업지시자 결정 영역.

## 작업지시자 결정 요청 영역

1. 옵션 A/B/C 중 어느 영역으로 진행? (PR #629 권위 영역 정합 시 옵션 A 본질)
2. 검토 영역 1 (후속 이슈 #665/#666/#667) — 메인테이너 권한 검토 + 마일스톤 분류 영역?
3. 검토 영역 2 (`.claude/skills/`) — 본 영역을 rhwp 레포에 본질적으로 포함하는 영역? `.gitignore` negate 패턴 본질적 정합?
4. 검토 영역 3 (본체 아키텍처) — `parser/ingest/` + `document_core/builders/` + `tools/rhwp-ingest/` 영역의 본질적 정합? cherry-pick 시 본질적 재배치 영역?
5. 검토 영역 4 (시각 정답지 게이트웨이) — 한컴 2022로 e2e HWPX 시각 검증 본질적 진행? 본 영역의 통과 본질이 머지 가부 본질?
6. 검토 영역 5 (단일 PR 묶음) — Task #660 본체 + Task #662 Skill 본질적 분리? 또는 단일 cherry-pick?
7. 검토 영역 6 (Skill 의존성) — 본 Skill 영역의 본질적 의존성(ImageMagick 등) 정책 영역?
