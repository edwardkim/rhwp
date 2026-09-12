# Task #7056 Stage 1 — 조판 지침과 공통 PR review 판정 연결

- Issue: [#7056](https://github.com/edwardkim/rhwp/issues/7056)
- 계획: [수행](../plans/task_m100_7056.md), [구현](../plans/task_m100_7056_impl.md)
- 기준 SHA: `8a06c99b91916b6c01756ff1bfdebe4d40dca0b1`
- 계획 commit: `df53a1b7b`
- 작업 branch: `docs/7056-typesetting-review-guidance`

## 변경 결과

구현 원칙은 [AGENTS.md](../../AGENTS.md#조판-수정과-검토-원칙)에 두고 `CLAUDE.md`의 import를
유지했다. 원인 근거·반례, 저장 LineSeg와 재조판, 측정·배치의 공통 줄 구성·메트릭, 합성/한컴 증거,
baseline·golden·래칫 변경 근거를 명시했다. 기존 Rust lint·suite 준비·source SHA 기록·한컴 PDF 재사용과
기여자/MCP 역할 경계는 유지했다.

[PR 공통 계약](../manual/pr_review_workflow.md)과 [선택표](../manual/pr_review/README.md)가
[공통 준수 검토](../manual/pr_review/intake_and_review.md#27-조판-원칙-준수-검토)를 가리킨다.
maintainer 일반, collaborator self, collaborator 매개 외부 PR 모두 같은 항목을 기록한다.
비해당은 실제 diff·주장에 근거한 이유를 남기며, 적용 대상은 여섯 항목의 판정과 증거를 기록한다.

`충족/미충족/미검증/비해당`은 개별 항목의 상태이며 PR 최종 판정 세 종류를 대체하지 않는다.
원칙 위반이나 필수 증거 부족이 있으면 원본 개선·녹색 CI만으로 승인하지 않고 blocker와 해제 조건을
명시한다. 실행 검출 회귀와 코드 검토상 우려를 구분한다.

[시각 증적](../manual/pr_review/visual_fixture_evidence.md#조판-규칙과-기준값-변경-증거)과
[로컬 검증](../manual/pr_review/local_validation.md)의 연결도 추가했다. 소스·fixture·baseline 데이터와
CI workflow는 수정하지 않았다. `collaborator_self_merge.md`에 한정한 규칙은 추가하지 않았다.

## 검증 결과

| 검사 | 실제 결과 |
| --- | --- |
| 수정 전 대상 6개 문서의 링크 검사 | 종료 코드 0, 내부 링크 오류 없음 |
| `git diff --check` | 종료 코드 0 |
| `python3 scripts/check_markdown_links.py --changed-from 8a06c99b91916b6c01756ff1bfdebe4d40dca0b1 --forbid-redirect-references` | 종료 코드 0; 문서 616개·변경 9개·redirect 30개 검사, 링크 오류 없음 |
| `python3 scripts/check_document_metadata.py` | 전후 모두 종료 코드 1; 609개 중 기존 16건, 신규·해소 오류 모두 0건 |
| 동일 메타데이터 검사기의 `validate_file`로 변경 장기 문서만 검사 | 5개, 오류 0건 |
| 추가 조판 링크 anchor와 실제 제목 직접 대조 | 12개, 오류 없음 |
| import·기존 게이트 보존 | `@AGENTS.md` 유지, AGENTS의 기존 `문서와 검증` 이후 본문 동일 |
| 실제 diff 검토 | 공통 계약·접수·선택표를 통한 전 역할 적용, 기존 명령·권한·시각 증적 보존 확인 |

기존 메타데이터 누락 16건은 아래 네 문서의 `kind/status/canonical/last_verified` 각 4건이다.

- `mydocs/tech/benchmark_vs_alternatives.md`
- `mydocs/tech/investigations/issue-4964/README.md`
- `mydocs/tech/investigations/issue-5511/README.md`
- `mydocs/tech/investigations/issue-5511/task_m100_5511_cli_surface_inventory.md`

## 적용 여부와 한계

이번 변경은 지침·리뷰 절차 문서만 수정했다. 제품의 줄 구성·측정·배치·baseline 변경에 대한 실물
시각 검증은 비해당이며, 공통 규칙의 전달·판정 경로와 문서 정합성을 확인했다.
Cargo·WASM·한컴 변환·실물 회귀·GitHub CI는 실행하지 않았다. #7040의 구현을 재심사하거나
에이전트의 이후 실제 준수·제품 렌더 품질을 검증한 결과가 아니다.

이슈는 등록·담당자 지정 후 모든 PR 경로에 공통 적용한다는 사용자 보완 지시를 본문에 반영했다.
remote push·PR 생성·merge·issue close는 수행하지 않았다. 기존 checkout의 오늘할일 변경은 보존했다.
