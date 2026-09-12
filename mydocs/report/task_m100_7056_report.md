# Task #7056 결과 — 조판 지침과 모든 PR 경로의 공통 준수 검토

- Issue: [#7056](https://github.com/edwardkim/rhwp/issues/7056)
- 근거: [PR #7040 검토 댓글](https://github.com/edwardkim/rhwp/pull/7040#issuecomment-5637631535)
- 계획: [수행 계획](../plans/task_m100_7056.md), [구현 계획](../plans/task_m100_7056_impl.md)
- 상세 증적: [Stage 1](../working/task_m100_7056_stage1.md)
- 기준 SHA: `8a06c99b91916b6c01756ff1bfdebe4d40dca0b1`
- 계획 commit: `df53a1b7b`; 지침·공통 review 구현 commit: `b7ef86a38`

## 개선 결과

| 요구 사항 | 반영 위치와 결과 |
| --- | --- |
| Claude·Codex 공통 구현 원칙 | [AGENTS.md](../../AGENTS.md#조판-수정과-검토-원칙)에 독립 근거·반례·공통 메트릭·시각 증거·기준값 변경 근거를 추가 |
| 중복 지침 정리 | [CLAUDE.md](../../CLAUDE.md)의 `@AGENTS.md`와 파서·저장·PDF 재사용·검증 게이트를 보존하고 중복 렌더링 원칙을 참조로 전환 |
| 모든 PR 경로의 공통 적용 | [PR 공통 계약](../manual/pr_review_workflow.md)·[선택표](../manual/pr_review/README.md)에서 역할과 무관한 적용 여부 기록을 요구 |
| reviewer의 직접 준수 판정 | [접수·리뷰 기록 2.7](../manual/pr_review/intake_and_review.md#27-조판-원칙-준수-검토)에 여섯 항목의 코드·증거 대조, 상태와 보류 해제 조건을 추가 |
| 합성/한컴·실행/우려 구분 | [시각 증적](../manual/pr_review/visual_fixture_evidence.md#조판-규칙과-기준값-변경-증거)에 사례별 증거 유형과 미검증 기록을 추가 |
| 기준값 완화의 독립 심사 | 시각 증적과 [로컬 검증](../manual/pr_review/local_validation.md)을 연결하고 실제 회귀를 baseline 갱신으로 숨기지 않는 계약 유지 |

원칙은 maintainer 일반·collaborator self·collaborator 매개 외부 PR 모두에 공통 적용된다.
별도 역할 전용 문서에 규칙을 가두지 않았다. 관련 없는 PR도 실제 변경에 근거한 비해당 사유를 적으며,
중첩 표 네 사례의 의무 검증은 해당 줄 구성 수정에만 적용한다. 원칙별 상태는 PR 최종 판정 세 종류와
구별하며, 원칙 위반과 필수 증거 누락은 녹색 CI나 원본 샘플 개선으로 승인하지 않는다.

## 검증과 한계

Stage 1의 공백·내부 링크·추가 anchor 검사는 통과했다. 변경 장기 문서 5개의 메타데이터 오류는 0건이다.
전역 메타데이터 검사는 전후 모두 기존 네 문서의 16건으로 종료 코드 1이었으며, 신규 오류는 0건이다.
기존 AGENTS의 Rust lint·suite 준비·증빙 절과 Claude import가 유지되는 것도 확인했다.

최종 정리에서 중첩 표 네 사례의 적용 범위를 명확히 했고, 정당한 조건 보완까지 금지하는 해석을 피하도록
샘플 속성 추가로 잘못된 가정을 덮지 말라는 review 문구를 구체화했다. 결과보고서를 포함한 최종 문서
링크·공백 검사와 기준 branch 병합 시뮬레이션은 최종 commit을 기준으로 실행 결과를 별도 PR 초안에 기록한다.

문서 지침·절차 개선이므로 제품 렌더링·Rust·WASM·fixture·baseline·CI workflow는 변경하지 않았다.
Cargo·한컴 변환·실물 시각 검증·GitHub CI는 미실행이다. #7040을 재심사하거나 향후 에이전트의 실제 준수를
검증한 결과로 보고하지 않는다. 로컬 개선 완료 후 사용자의 PR 생성 지시에 따라
[PR #7057](https://github.com/edwardkim/rhwp/pull/7057)을 `upstream` 작업 branch에서 `devel` 대상으로
생성했다. [공통 준수 항목을 포함한 self-review](../pr/archives/pr_7057_review.md)와 오늘할일을 같은
PR에 기록하며 최신 head CI·병합 검증 결과는 PR 본문을 따른다. merge·issue close는 수행하지 않았다.
기존 checkout의 미커밋 오늘할일 변경과 branch는 보존했다.
