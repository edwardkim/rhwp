---
kind: snapshot
status: active
canonical: mydocs/working/task_m100_7095_6946_evidence_stage1.md
last_verified: 2026-09-16
---

# 구현 주장과 검증 증거 대조 지침 — 1회차

Issue: #7095, #6946 (보정 사례의 출처이며 이슈 종료 작업은 아님)

## 분석

- 시작 head: `444da0173`, branch: `codex/pr7141-7175-7178-20260916`. 시작 작업 트리는 깨끗하다.
- 사용자 요청은 반복되는 메인터너 보정의 원인인 실행·증거 심사 부족을 지침으로 보완하는 것이다.
  공통 helper를 공유해도 후속 분기에서 예산을 덮어쓸 수 있고, 내용 검사만으로 여백·테두리를
  검증할 수 없으며, 좌표 겹침으로 예약 소유를 판단할 수 없었던 사례를 근거로 한다.
- AGENTS.md에 구현 주장과 실제 검사 항목의 대조 기준을 두고 CLAUDE.md, CONTRIBUTING.md,
  PR 템플릿, 공통 reviewer 절차가 이를 참조하도록 연결한다. 기존 증적 재사용과 역할별 적용 범위를
  유지하고 테스트·CI·별도 보고서를 일괄 추가하지 않는다.
- CONTRIBUTING의 수정 전 실패 증명은 일반 변경에는 권장으로 유지하되 조판 결함 검출을
  주장하는 테스트에는 공통 기준이 적용됨을 명시한다. 기존 판정 용어를 유지한다.
- 문서 변경만 수행한다. 검증은 변경 diff의 정합성·공백, 변경 파일의 링크·메타데이터,
  새 내부 heading 연결을 확인한다. 결과보고 후 같은 회차를 커밋한다.

## 결과보고

- AGENTS.md에 실제 소비 경로, 핵심 가정을 깨는 반례, 수정 전 실패의 원인 확인,
  주장과 assertion/시각 관측의 대응, 증거 부족의 판정과 부분 해결의 종료 범위를 추가했다.
- CLAUDE.md와 CONTRIBUTING.md는 공통 기준을 연결한다. PR 템플릿은 주장·경로·기대값·검사·
  전후 결과·판정을 연결하며 기존 입력 출처·분할 계약·Visual Sweep·baseline 항목을 유지한다.
  공통 reviewer 절차도 이 기준을 사용한다. 기존 판정 용어와 충분한 증거의 재사용을 유지했다.
- 내용 대조: #7178의 x 겹침/예약 소유 추정은 소유 반례, #7141의 예산 덮어쓰기·여백 중복은
  후속 경로와 위치 assertion, 그림 projection의 테두리 과장은 생성 출처 반례와 테두리 관측으로
  연결된다. 이 확인은 지침의 사례 대응 검토이며 기존 결함의 테스트를 재실행했다는 뜻이 아니다.
- `git diff --check`: exit 0.
- `python3 scripts/check_markdown_links.py AGENTS.md CLAUDE.md CONTRIBUTING.md
  .github/pull_request_template.md mydocs/manual/pr_review/intake_and_review.md
  mydocs/working/task_m100_7095_6946_evidence_stage1.md`: 6개 문서, 내부 상대 링크 이상 없음, exit 0.
- `scripts/check_document_metadata.py`의 `validate_file`로 변경 manual·단계 문서 2개 확인:
  오류 0, exit 0. 공통 heading 1개와 CLAUDE/CONTRIBUTING/템플릿/reviewer의 참조 4개도 확인했다.
  링크 검사기가 anchor·외부 URL 응답을 검사하지 않으므로 새 heading과 참조 문자열은 별도로 대조했다.
- 문서 변경이므로 Rust·WASM·회귀 테스트·시각 캡처는 실행하지 않았다. CI 강제 게이트를 추가한
  변경이 아니며, 향후 작성자와 reviewer의 실제 준수까지 이번 문서 검사로 보증하지 않는다.
- 위 결과를 사용자에게 보고한 뒤 이 회차를 로컬 커밋한다. 원격 push·PR 생성은 수행하지 않는다.
