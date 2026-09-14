---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-15
---

# PR #7143 메인터너 보정 기록

[검토 결과](pr_7143_review.md)의 두 보류 사유를 메인터너 코드 `32aa62ed8`까지 보정했다. 새 코드의 최종 로컬 검증을 완료했으며 원격 source 수정·새 PR·merge는 하지 않았다.

1. 시작점은 `upstream/devel` `38af2aae3` + 원 PR `34e1186f4`를 체리픽한 `codex/pr7143-review-20260914` / `4a9546f8c`다. 원 contributor commit의 출처를 보존한다.
2. 보정 시 새 head/base를 먼저 확인한다. direct source push 경로를 선택하려면 `planet6897/rhwp`의 일반 push 권한과 이 PR의 maintainer 수정 권한, 정확한 branch의 push 가능성을 구분해 확인한다. 불가하면 현재 누적 branch에서 원본 저장소 대상 통합 PR 경로를 사용한다.
3. 분할 조각이 실제 소유하는 start/end unit과 높이·예산 결정을 공통 결과로 만든다. 일반 splittable 행도 renderer의 증가분을 예약하고, 증가분이 fit하지 않으면 실제 컷/이월을 생성한다. `start_cut` 누락과 기존 native two-row owner 경로의 차이도 확인한다. 문서별 예외나 임의 slack으로 덮지 않는다.
4. 별도 code/test commit으로 새 보정을 기록한다. 기존 4건, 메모리 축소 p3 body overflow, 원래 h만 fit하는 경계, 시작 컷 및 다수 rowspan 누적 경계를 검증한다. 합성 입력은 정상 한컴 PDF 사례와 분리한다.
5. 새 code head의 필수 lint·변경 범위 CI와 직접 visual sweep을 완료한다. target 문구뿐 아니라 본문/다음 행 위치, #6981의 p82·152 잔여 범위를 확인한다. 정상 기준으로 baseline을 완화하지 않는다.
6. CI가 녹색인 뒤 최종 검토·오늘할일·증적을 동일 처리 branch의 trailing commit으로 정렬한다. 기존 HWP/PDF는 같은 경로로 재사용하고 누락 파일만 보존한다. source head에 직접 넣는 경우 최신 base의 오늘할일을 통째 복사하지 않고 merge simulation·링크·기존 기록 보존을 확인한다.
7. 작업지시자의 push/PR/merge 승인 범위에 따라 진행한다. 최종 exact head CI와 MERGEABLE/CLEAN 확인 후 merge하고, post_merge 순서로 duration 결과·devel sync·관련 issue/PR comment·본 작업 소유 산출물 정리를 완료한다. contributor fork branch와 공유 target은 보존한다.

## 진행 결과

- 보정 commit: `9d1607bff` → `ecbe4ce82` → `32aa62ed8e7d8f9c52fe64f1e08f9af3710ef121`.
- `rowbreak_straddle_cut_units`를 높이 요구 계산과 실제 셀 배치가 공유한다. 시작 컷 소비와 native 2행 문단 owner를 동일하게 반영한다.
- 일반 splittable 행에도 잔여 높이를 예약한다. 누적 예약은 원래 `cut_row_h` 합 대신 실제 수용한 `consumed`를 사용하며, 예산 초과 시 작은 원래 높이로 되돌리지 않는다. 실제 끝 컷이 생기면 컷 높이로 측정하고 renderer도 같은 컷을 적용한다.
- 기존 4건과 신규 4건 focused 8/8 PASS. 신규 검사는 원본 표만 보존한 IR에서 본문 경계, 목표 문구 1회 출현 및 셀 경계, 41개 페이지 예산과 실제 페이지 전환을 검증한다. 원본 입력의 p3 4.213px 초과는 기존 0.5px 계약 안으로 복구됐다.
- 실제 PDF p83의 행 이월, 소비된 최종 컷 뒤의 빈 후속 페이지도 검사에 추가했다. 원 PR page-count baseline 변경은 되돌려 devel과 동일한 413쪽이며 한컴 415쪽 기준은 보존했다.
- 최종 head에서 focused 8건, 전체 nextest 9,884건(51 skip), 필수 3종 Clippy·workspace build·fmt·manifest, Native Skia lib·placeholder·direct PDF가 통과했다. fresh WASM을 실제 Chrome에서 실행한 Visual Sweep 16쪽도 완료했다. Native/WASM raster는 16쪽 중 10쪽이 동일하다. 나머지 6쪽은 각 9~64픽셀 차이이며 직접 비교에서 배치·본문 차이는 보이지 않았다. 16쪽 렌더 트리는 플랫폼별 문단 index sentinel을 제외하면 구조·텍스트·좌표가 동일하다. 실행 요약과 해시는 검토 문서 및 증적 JSON에 기록했다. 원 head의 CI 성공을 보정 head의 성공으로 재사용하지 않는다.
- Docker daemon 연결 불가를 확인했다. 저장소에 명시된 native `wasm-pack-locked.sh --no-opt` 진단 경로로 별도 검토 package를 만들었으며, 최적화 배포 빌드로 주장하지 않는다.

rollback은 메인터너 보정 commit에 한정하고 contributor 원 변경 또는 다른 작업의 파일을 되돌리지 않는다.

## 지침이 보정을 예방하지 못한 원인과 개선 — 2026-09-15

사용자 요청에 따라 `CLAUDE.md`에서 공통 지침·기여 제출·review로 이어지는 경로를 감사했다.
원 PR `34e1186f4`와 보정 기록 `f5f8c586d` 사이에서 `CLAUDE.md`, `AGENTS.md`, `CONTRIBUTING.md`의
diff는 없었다. 원 PR에 최신 공통 원칙이 없었다고 설명할 수 없다. 작성자의 에이전트 실행 로그는
확보하지 않았으므로 Claude 사용 여부, import 성공 여부나 지침을 읽고 무시했는지는 단정하지 않는다.
확인한 원인은 코드·테스트·문서에 드러난 다음 차이다.

| 발견된 사실 | 기존 지침·검사의 한계 | 이번 개선 |
| --- | --- | --- |
| renderer는 모든 per-row 조각을 늘렸지만 pagination은 특수 분기만 예약 | 공통 결과 원칙은 있었으나 제출 항목이 helper 이름·focused PASS만으로 채워질 수 있었음 | 적용 분기별 컷·소유 유닛→요구→예약→실패 컷/이월→배치 경로를 실제 코드와 연결 |
| 원래 높이만 fit하면 수용 후 renderer가 다시 늘림 | 목표 문구의 셀 내부 표시 검사는 조각의 본문 점유를 검증하지 않음 | 원래 높이만 fit하는 경계와 본문 외곽·다음 내용까지 완료 조건에 포함 |
| 물리 빈 밴드와 패딩 중복 계상, 소진된 끝 컷 뒤 빈 페이지 | 내용 컷·물리 소비량·종료 상태를 같은 것으로 간주한 가정이 반례에 드러나지 않음 | 빈 밴드/패딩 계상 위치와 시작·끝 컷·누적 rowspan·소비 완료 경계를 명시 |
| `CONTRIBUTING.md`가 시각 검증을 “참고”, 자가 검증을 “회귀 없음의 증명”이라고 설명 | 필수 범위 표와 직접 시각 확인 원칙에 상충하며 녹색 검사에 과도한 의미를 부여 | 해당 렌더링 주장에는 직접 시각 검증이 필수임을 통일하고 검사 범위 밖의 안전을 주장하지 않도록 수정 |
| 메인터너 후보 `9d1607bff` 9,882 PASS 후 p83 이월, `ecbe4ce82` 9,883 PASS 후 빈 p378 발견 | 이 작업도 전체 검사를 먼저 돌리고 직접 경계 비교를 늦춰 보정 왕복을 늘렸음 | focused 경계·영향 페이지 Visual Sweep 선행, 이후 최종 head 전체 게이트·fresh WASM 확인으로 순서 보완 |

원칙 위반은 기존 지침을 충족한 구현이 아니었다. 동시에 선언형 원칙만 강화하고 호출 경로와 독립적인
반례를 확인하지 않으면 같은 문제가 재발할 수 있다. `CLAUDE.md`는 동작을 검사하는 실행 프로그램이
아니므로 문구 추가만으로 준수를 보장한다고 주장하지 않는다. 이번에는 다음 세 지점에 같은 계약을
연결했다.

- 구현: [AGENTS 공통 입증](../../../AGENTS.md#분할이어받기-변경의-입증)과
  [CLAUDE 완료 기준](../../../CLAUDE.md#구현과-검증의-완료-기준).
- 제출: [CONTRIBUTING](../../../CONTRIBUTING.md#페이지-분할--페이지네이션-영향-pr-의-경우)의
  상충 문구·실행 순서와 [PR 템플릿](../../../.github/pull_request_template.md)의 구체적인 근거 항목.
- 검토: 모든 역할의 [공통 review 표](../../manual/pr_review/intake_and_review.md#27-조판-원칙-준수-검토),
  [시각 증적](../../manual/pr_review/visual_fixture_evidence.md#조판-규칙과-기준값-변경-증거),
  [로컬 검증 순서](../../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증).

문구 존재만 검사하는 CI를 의미 검증으로 추가하지 않았다. 실행 가능한 보호는 앞서 추가한 네 경계
테스트에 남아 있고, 새 지침은 이 사례에서 각각 무엇을 발견해야 하는지 위 표로 대조한다. 새 PR의
자동 검사와 직접 증거를 대신하지 않으며, 개선 이후 실제 메인터너 보정 빈도가 감소했는지는 아직
측정하지 않았다. 기여 스킬의 활성 시각 가이드는 이미 CONTRIBUTING을 정본으로 참조하고 있어
같은 규칙을 복제하지 않았다.

이번 지침 변경은 Rust·테스트·기준 입력·CI workflow를 바꾸지 않는다. 수정 문서의 메타데이터와
상대 링크·anchor, 상충 문구 제거, 제출→검토 경로 연결 및 `git diff --check`를 확인했다.
`check_markdown_links.py`의 수정 문서 9개 검사와 추가 내부 anchor 12개 대조를 통과했다.
새 Rust 검증을 실행한 것으로 기록하지 않으며, 앞 절의 실행 결과는 코드 `32aa62ed8`에 대한 것이다.
