# task_m100_4530 stage1 — 살아있는 에이전트 대전(Codex)

- 이슈: #4530 / 브랜치: task_m100_4530 (base devel)
- 산출물 4종: tools/gen_agent_codex.py(생성 하네스, --check 멱등) ·
  mydocs/manual/agent_codex/ 13파일(생성 10장 + 손글 서문·판단트리·README,
  총 ~4.7천 줄) · tests/agent_codex_contract.rs(커버리지 가드 2) ·
  .claude/skills/rhwp-codex(진입 스킬)

## 구조 결정

- 문서의 단일 출처 = 바이너리(capabilities·--help·export-provenance-map) +
  실픽스처 실행 봉투. 표본 결정론: 고정 작업폴더(target/codex-tmp)·<repo>
  경로 정규화·배열 2원소/문자열 160자 절단·본문 불변 시 날짜 보존.
- 우산 명령(edit·inspect)은 capabilities 에 2단 이름이 없어(실측 0) 도움말
  줄에서 하위 장을 합성 — 편집 5·보안 3 실측 표본 확보.
- 실행 안 한 명령은 "계약만" 정직 라벨(batch=NDJSON 형식, mcp-serve=상주,
  진단면 30종은 85장으로 격리 + 비권장 명시).

## 함정 실록

- body_of 가 frontmatter 뒤 공백 줄을 본문에 남겨 --check 가 영구 붉는 버그를
  자가 실측으로 잡음(모든 장 '변경' 판정) — lstrip 수리 후 멱등 성립
  (재생성·검사 연속 실행 변경 0).
- 진단·프로브 30종을 미분류로 두면 교본이 오염 — 접두 규칙으로 85장 격리,
  진짜 미분류만 90장(현재 0건, 파일 자동 삭제).

## 검증

- gen --check 멱등 0변경 · agent_codex_contract 2/2(전 71명령 장 보유·생성
  표지) · check_markdown_links/metadata 통과 · 표본 실문서 렌더 증빙 커밋.
