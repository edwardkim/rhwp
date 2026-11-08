---
kind: review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-06
---

# PR #7609 리뷰 — 기여자 Visual Sweep 증적 첨부와 제출 기준

## 최종 판정

**승인** — 문서 변경 후보의 범위·내부 링크·공백·병합 가능 여부를 검증했고 사용자 지시에 맞는 제출 순서와 일반 예제를 확인했습니다.

최종 merge 조건은 기록을 포함한 최신 PR head의 required checks 통과, 충돌 부재와 사용자 병합 승인입니다. 작성 시점 CI는 대기 중이며 완료로 판정하지 않았습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7609](https://github.com/edwardkim/rhwp/pull/7609) / jangster77 / devel |
| 문서 후보 head | `7fbc35e932c5a5001f7e0fa9309a3ed676455228` |
| 검증 base | `d9749c5a15b95206455953494f6fc871c115cef1` |
| 역할 | collaborator 본인 PR의 self-review; reviewer 지정 없음 |
| 라우팅 | collaborator_self_merge + intake_and_review + local_validation + review_only_fast_pass |
| 관련 자료 | #7551의 첨부 형식 참고; 관련 이슈 종료 없음 |
| 규모·상태 참고값 | 문서 3개, 124줄 추가/22줄 삭제, Open, MERGEABLE, CI 대기 |

선택 문서와 부모 workflow·README·review_template을 읽고 적용했습니다. 상태값은 작성 시점 참고이며 최신 head에서 재확인합니다.

## 변경과 검토 범위

- 기여 지침·PR 템플릿·Visual Sweep 가이드에 생성 전 증적 준비 → stable 경로 커밋/push → 실제 head SHA의 이미지 URL → PR 본문 첨부 순서를 명시했습니다. PR 번호를 미리 알 필요가 없습니다.
- 특정 기여자의 개인 raw URL을 제거하고 자신의 저장소·SHA·입력·페이지로 바꾸는 일반 예제로 정리했습니다.
- #7551의 형식을 참고해 입력·페이지별 Native/fresh WASM review/standalone overlay 표를 구성하고, 전체 TSV 최저값과 대표 이미지·직접 판독 기록을 구분했습니다. 해당 PR 구현의 검토·승인 주장은 없습니다.
- 90% 미만·누락·측정 불가·필수 경로 미실행은 일반 사유로 제출 허용하지 않습니다. 정확히 90%는 통과하며 기존의 정식 해결 불가능한 글꼴 예외 계약은 유지합니다.
- renderer·편집 command·parser·model·serializer·workflow·sample·baseline을 수정하지 않았습니다. 구현 계획서가 필요한 코드 보정이나 다수 PR 통합은 없어 이 리뷰 안에서 절차를 기록했습니다.

## 검증 입력과 결과

문서 후보 SHA와 위 base에서 다음 검사를 완료했습니다.

| 명령·검사 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD` | PASS |
| `git diff --check` | PASS |
| `python3 scripts/check_markdown_links.py CONTRIBUTING.md .github/pull_request_template.md mydocs/manual/verification/visual_sweep_guide.md` | 3개 문서 내부 Markdown 상대 링크 이상 없음 |
| `git merge-tree --write-tree upstream/devel HEAD` | exit 0, tree `29426aefa7a6fd29ef8ca3fde49087116ecba4d8` |
| 제출 순서·90% 경계·예외·일반 URL 예제 내용 대조 | 세 문서 정합성 확인 |
| GitHub 게시 본문·head API 재조회 | 한글/LF 본문 정상, base devel, 후보 SHA 일치 |

조판 원칙 준수 검토: **비해당** — 실제 출력 소비 경로와 조판 규칙·기준값을 바꾸지 않는 기여자 안내 문서 변경입니다. 상수·정책·호출 경로의 제품 동작 변경도 없습니다.

검증 입력 커밋 확인: **비해당** — HWP/HWPX/PDF 입력을 사용하지 않았습니다. Rust/npm/WASM 빌드·회귀·Visual Sweep은 실행 코드와 실제 출력 변경이 없어 비해당입니다. 링크 도구는 외부 URL과 앵커를 검사하지 않으며, 변경한 예제 URL은 기여자가 실제 값으로 치환할 템플릿입니다.

## 기록 커밋과 남은 조건

같은 PR에 이 archive self-review와 2026-10-06 오늘할일의 이번 항목을 추가합니다. 기존 오늘할일을 보존하고 latest base/head의 merge-tree·공백·실제 merge tree 문서 링크·기록 보존을 push 전에 검사합니다. 로그·중간 산출물은 ignored `output/pr-review/visual-evidence-guidelines/`에만 둡니다.

최종 head의 GitHub mergeability와 required checks는 별도로 확인합니다. 시각 검증을 판정에 사용하지 않아 contributor 시각 증적 comment 계획은 비해당입니다. 병합과 후속 처리는 사용자 지시에 따라 별도 단계에서 진행합니다.

## 후속 보정 — 대표 페이지 이미지와 전체 TSV 첨부

사용자 지적에 따라 전체 페이지 PNG 생성·커밋·본문 첨부가 필요하지 않음을 세 기여자 문서에 명시했습니다. PR 본문에는 변경 효과·주요 경계를 보여주는 대표 페이지의 Native/fresh WASM review·standalone overlay와 검증 범위 전체 Native TSV 원본만 제출합니다. WASM TSV 첨부는 요구하지 않습니다. TSV는 ZIP 첨부 또는 다운로드 가능한 동일 파일 링크로 제공하며, 요약값·로컬 경로만으로 대신하지 않습니다. TSV는 ignored output에 보존하고 Git에 커밋하지 않습니다.

본문의 페이지별 이미지 표는 선택한 대표 페이지에만 반복합니다. 전체 검증 범위의 90% 게이트·누락/측정 불가 금지·정식 글꼴 예외 계약은 유지했습니다. 문서 후보 `a43edfcff0d21bb8185cf36591f2c99b937fc290` 이후의 기여 지침·템플릿·Visual Sweep 가이드 보정이며, 실제 제품 출력은 바꾸지 않았습니다. 로컬 공백·세 문서 내부 링크 검사를 다시 통과했고, 후속 push 전 정확한 base/head의 실제 merge tree 링크·오늘할일 기록 보존 검사와 push 뒤 PR 본문/head 재확인을 적용합니다.
