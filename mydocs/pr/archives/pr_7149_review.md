# PR #7149 검토 — 감사 대상에서 사라진 제외 항목 검출

## 접수와 범위

| 항목 | 확인 내용 |
|---|---|
| 원 PR | [#7149](https://github.com/edwardkim/rhwp/pull/7149), `lpaiu-cs`, devel 대상 |
| 원 head | `42d1493ff2f90642f40484fb3657f55bcb88ee39` |
| 현재 base | `4fddb1bb7244fb478ea18f927aca273f93ac7322` |
| 누적 검토 branch/head | `codex/pr7145-pr7149-review-20260915` / `65df76e6195e0b87471d82359eb11c437d0bf1a0` |
| 규모 | 1 commit, 2 files, +27 −4 |
| 접수 시점 참고값 | OPEN, non-draft, MERGEABLE/CLEAN, maintainerCanModify=true; reviewer `jangster77` 지정 |
| 관련 이슈 | [#7148](https://github.com/edwardkim/rhwp/issues/7148) |

기본 경로 `collaborator_external_pr`, 보조 경로 `intake_and_review`, `local_validation`,
`multi_pr_update_branch`를 적용했다. 최신 upstream/devel 위에서 #7145 다음으로 `cherry-pick -x`했다.
두 원 PR 모두 current-base merge simulation이 충돌 없이 통과했다. 기존 merged PR #3672 등 기여
이력이 있어 첫 기여자 경로는 적용하지 않았다. 원 PR 종료·원격 코드 push·통합 PR 생성은 수행하지 않았다.

## 코드와 동작 검토

`mutation-method-registry.ts`의 `exportHwp`, `exportHwpVerify`, `exportHwpWithPassword`
제외 항목을 삭제했다. 세 이름은 MUTATING_VERB에도, 현재 Rust `&mut self` inventory에도
없으므로 제외 처리가 필요하지 않다. 저장 API 자체의 호출·바이트·undo 동작을 변경하지 않는다.

새 테스트는 실제 registry, bridge, Rust source에서 얻은 이름을 교차해 두 감사 경로에 모두
해당하지 않는 제외 항목을 검출한다. 기존 드리프트·분류 검사는 유지되므로 향후 해당 API가
`&mut self`로 바뀌면 다시 분류를 요구한다. registry를 복사한 별도 구현을 정상 검증으로 사용하지 않았다.

브리지에 존재하지 않는 제외 이름은 `.filter(bridge.has)`에서 빠지는 제한이 있다. 이 PR의
검증 주장은 **브리지에 실제 존재하나 감사가 더 이상 묻지 않는 항목**이며, 존재하지 않는 이름까지
검출한다고 해석하지 않는다. 현재 이슈가 지목한 세 이름은 이 범위에 포함된다.

## 완료한 검증

| 명령·대조 | 결과 |
|---|---|
| `node --test tests/mutation-routing-guard.test.ts` (Studio) | 10/10 통과 |
| 격리 사본에서 새 테스트 + 삭제 전 registry | 9 pass / 1 fail; 위 `exportHwp*` 세 이름을 정확히 지목 |
| `npx tsc --noEmit` (Studio) | exit 0 |
| `npm test` (Studio) | 1,735 pass / 2 skip / 0 fail, 총 1,737 |
| `git diff --check upstream/devel..HEAD` | 통과 |
| [원 head CI](https://github.com/edwardkim/rhwp/actions/runs/34895915442) | API head SHA가 원 head와 일치, success. Frontend package gates success; Rust 관련 job은 scope에 따라 skipped |

음성 대조는 저장소 파일을 되돌리지 않고 Git 외부 테스트 사본의 registry 입력만 교체했다.
조판 규칙·줄 구성·분할·paint·renderer 상수·baseline 완화는 비해당이다. HWP/HWPX/PDF 입력과
렌더 출력 변경이 없는 검사·registry 정리이므로 이 PR 자체의 Visual Sweep은 생략했다.
같은 누적 branch의 renderer 검증은 [#7145 검토](pr_7145_review.md)에 별도로 기록한다.

## 메인터너 표기 보정

사용자가 연결한 [Copilot 코멘트](https://github.com/edwardkim/rhwp/pull/7149#discussion_r4014011807)는
테스트 이름과 주석 두 곳의 `래칥 → 래칫` 오타다. `fec435fa9`에서 수정하고 동일 테스트를
다시 실행해 10/10 통과했다. 테스트 조건과 제품 코드는 바꾸지 않았으므로 앞서 통과한
프런트엔드 전체 테스트·TypeScript 검사를 반복하지 않았다. 원 PR에는 push하지 않았다.

## 최종 판정

**승인**. #7148이 제시한 기존 세 항목 제거와 같은 실패 계급의 재발 검출을 실제 실행으로 확인했다.
동작상 신규 코드 검토 지적은 없다. 표기 지적 두 곳은 위 보정으로 해소했다. 원 PR CI는 원 head의 증거이며 통합 head CI 통과로 대신 기록하지 않는다.
통합 PR을 제출·merge할 때는 최종 head의 변경 범위별 사전 게이트 및 GitHub CI를 확인한다.
이번 검토만으로 원 PR이나 이슈를 close하지 않는다.

통합 순서·증적·후속 처리는 [처리 계획](pr_7149_review_impl.md)을 따른다.
