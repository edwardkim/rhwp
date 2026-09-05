# #5319 실 에이전트 시험지 ingest(PDF/이미지→HWPX) — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5319
브랜치: `feat/agent-exam-ingest` (`upstream/devel` 기준 격리 worktree)
범위: `.agents/skills/rhwp-exam-ingest/` (SKILL.md · references/ ·
examples/ · fixtures/ · helpers/) ·
`scripts/tests/test_agent_exam_ingest.py` · 본 문서
비범위: `gym/` · 다른 스킬 · 열린 PR 파일 ·
`src/document_core/builders/exam_paper.rs` · 새 rhwp CLI

## 무엇을

에이전트가 PDF/이미지/MD/DOCX 시험지를 HWPX 로 바꿀 때 **입력 정규화 →
Vision 구조 인식 → ingest.json → crop → `build-from-ingest --media-dir -o`**
순서를 빠뜨리거나, `auto_number` 를 중복하거나, poppler 없음을 전체 실패로
읽거나, 수식/표를 스키마에 없는 필드로 욱여넣는 실수를 줄인다.

기존 `rhwp-exam-ingest` 는 SKILL.md 한 장과 helper 네 개만 있었다.
이 작업은 본문을 30초 판단 내비게이터로 재작성하고 `references/` 21장,
예제 24개, 스키마·봉투·트랜스크립트 픽스처, helper `--json`/`--dry-run`
계약을 닫는다.

## 왜

이슈 본문: 에이전트가 시험지 원본을 HWPX 로 바꾸는 **실사용 경로**.
gym 금지. 새 CLI / DocumentCore exam_paper 발명 금지.

코어는 이미 있다.

- JSON 스키마: `tools/rhwp-ingest/schema/ingest_schema_v1.json` (#660, #667)
- Rust 모델: `src/parser/ingest/schema.rs` (`deny_unknown_fields`, #3358)
- 조립: `rhwp build-from-ingest --media-dir -o` (기존 CLI)
- helper: `pdf_to_pngs.sh` · `extract_docx.py` · `crop_image.sh` · `check_deps.sh`

에이전트가 필요한 것은 새 빌더가 아니라 **언제 어느 helper 를 치고,
어느 스키마 필드를 채우고, 어느 실패 봉투에서 멈추는가** 이다.
`auto_number` 기본을 무시한 채 stem 에 `"1. "` 를 넣으면
`1. 1. 다음 글의 주제는?` 이 인쇄된다.

DoD: additions 5000–10000 (최소 5000). PR 전 `cargo fmt --all -- --check`.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-exam-ingest` 에
   `feat/agent-exam-ingest` 를 `upstream/devel` 에서 분기.
   `rhwp`, `rhwp-desk*`, `rhwp-handoff`, `rhwp-scaffold-final`,
   `rhwp-doc-repro` 는 쓰지 않음. 이름 있는 worktree 를 훔치지 않음.
2. SKILL.md 를 사다리·정지 규칙·인계 인덱스로 재작성.
3. `references/` 21장: 입력 정규화, PDF/DOCX/이미지/MD, 스키마,
   passages, boxed, placement, auto_number, bbox, build-from-ingest,
   check_deps 봉투, 실패 봉투, 한계, 함정, 트랜스크립트, 게이트,
   발화 행렬, 종료 코드.
4. helper 에 `--json` / `--dry-run` 을 추가. 새 rhwp 서브커맨드가 아니다.
   - `check_deps.sh --json` → `DEP_MISS_POPPLER` /
     `DEP_MISS_IMAGEMAGICK` / `DEP_MISS_PYTHON_DOCX` 봉투
   - `crop_image.sh` bbox 10진 정수·w/h≥1 계약, exit 4
   - `pdf_to_pngs.sh` DPI 72–600, `page_001.png`
   - `extract_docx.py` fallback 은 실패가 아님
5. `_gen_pack.py` 가 `fixtures/` · `examples/` · `19_intent_matrix.md` 를 방출.
   유효/무효 스키마, 의존성 봉투, 발화 80+, 모의 30문항, 트랜스크립트.
6. `scripts/tests/test_agent_exam_ingest.py` 가 발명 명령·gym·이웃 스킬
   재작성·스키마 모양·헬퍼 dry 경로(파이썬 extract_docx 실호출,
   셸 스크립트는 파일 계약)를 바이너리·poppler 없이 검사.

## 하지 않은 것

- `src/document_core/builders/exam_paper.rs` 수정
- `src/parser/ingest/schema.rs` 필드 추가
- 새 `rhwp` 서브커맨드 / `build-from-ingest` 플래그
- Picture writer (#182) 구현
- Equation IR · Table IR
- gym pack / 과제 / 채점기
- form-fill · table-exchange · onboarding · safe-edit · doc-triage 본문
- capability 등록부 (다른 열린 에이전트 PR 과 충돌 회피)

## 검증

```bash
python -m unittest scripts.tests.test_agent_exam_ingest
cargo fmt --all -- --check
```

`cargo test` 는 이 PR 에서 해당 없음 (Rust 변경 없음).
clippy / WASM / studio / 작업 증빙 캡슐도 해당 없음.

라이브 `build-from-ingest` 는 기존
`tools/rhwp-ingest/schema/sample_minimal.json` 경로가 정본이다.
스킬 픽스처 `valid_*.json` 은 같은 필드만 쓴다.

## 권위

- `tools/rhwp-ingest/schema/ingest_schema_v1.json`
- `src/parser/ingest/schema.rs`
- `mydocs/manual/cli_commands.md` §`build-from-ingest`
- 관련 이슈: #5319 · #660 · #667 · #3358 · #182
