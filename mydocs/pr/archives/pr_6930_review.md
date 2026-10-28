# PR #6930 작성자 self-review

- 검토 시각: 2026-09-09 12:33 KST. 아래 GitHub 상태는 작성 시점 참고값이다.
- base route: 내부 task PR / `collaborator_self_merge.md`의 작성자 self-review 경로
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서,
  `github_operations.md`, `codex/docs_and_git_workflow.md`
- reviewer는 별도 지정하지 않았다. 이 문서는 GitHub approve event가 아니다.

## 1. 대상과 범위

| 항목 | 확인값 |
| --- | --- |
| PR / Issue | [#6930](https://github.com/edwardkim/rhwp/pull/6930) / [#6916](https://github.com/edwardkim/rhwp/issues/6916) |
| 작성자·담당 | edwardkim / edwardkim, milestone v1.0.0, enhancement·mcp·packaging |
| base / head branch | devel / task_m100_6916 |
| 검토한 PR head | `2577e3328951b5a856a9c88b5e87ee5d6c5b0ec3` |
| 최신 base | fetch한 `c72ad805cc60e4a5cf5689c18b44e7214cec68fe` |
| 규모 | 검토 전 16 files, +866/−8, 7 commits. 대부분 계획·검증 문서 |
| 상태 | OPEN, non-Draft, MERGEABLE, CLEAN. merge 전 다시 확인 |

제품 CLI의 Gym README compile-time 의존을 제품 소유의 짧은 안내로 바꿨다.
URI/name/title/MIME, list/read·오류 계약은 보존하고 description·본문·size만 승인된 범위로 변경했다.
Gym 자산과 일반 API를 제거하지 않고 의존 방향을 Gym → rhwp로 확정했다.

## 2. 코드·운영 검토

1. `src/mcp_serve.rs` 변경은 정적 리소스 선언으로 한정된다. handler·CLI dispatch·parser·renderer는 동일하다.
   Gym 경로 fallback이나 네트워크 다운로드가 없어 설치본이 Gym을 다시 요구하는 경로를 추가하지 않았다.
2. 새 안내 파일이 실제 존재하고 Oracle sparse 입력과 mirror test가 함께 바뀌었다.
   trigger·권한·timeout·continue-on-error·verdict 정책은 바뀌지 않았다.
3. 새 integration test는 임시 디렉터리를 원자적으로 생성한 뒤 자신이 만든 경로만 정리한다.
   파일 기반 표준 스트림, 유한 대기, child 종료 guard 및 런타임 바이너리 경로 우선 계약을 확인했다.
4. 테스트는 URI 유일성·의도한 본문·MIME·size·미지 URI 오류를 검사한다.
   원본만 `tests/cases/`에 추가했고 generated suite·manifest·Cargo marker는 PR에 없다.
5. Gym 평가 의미·일반 CI 정책·package manifest는 그대로다. 조직·프레임 감사의 Gym 검사는
   제품 독립 빌드와 다른 조건으로 문서에 명시되어 있다. 렌더 영향·신규 fixture가 없어 visual sweep은 해당 없음이다.

추가 코드 보정이 필요한 문제는 발견하지 못했다. 수행·구현 계획에 실행 순서와 복구 단위가 있으므로
별도 review_impl은 추가하지 않았다.

## 3. 검증 근거

로컬 검증은 [최종 보고서](../../report/task_m100_6916_report.md)와
[Stage 3](../../working/task_m100_6916_stage3.md)의 완료 결과를 확인했다.
native/WASM/workspace lint 전체, integration 9,302 passed(ignored 46), Gym 없는 Linux/Docker WASM,
설치본 MCP 15개 리소스, npm 포함물, 외부 바이너리 Gym core-cli 54/54가 통과했다.
검증 코드 `369e789ee` 이후 제출 head까지는 기록 문서만 바뀌었고 빌드 입력의 동일성을 재확인했다.
현재 base merge simulation tree는 `2273a2c8b4964a86c52310d449794446409f1f72`로 head tree와 같았다.
동일 코드의 녹색 Full CI가 있으므로 로컬 전체 회귀를 다시 실행하지 않았다.

| GitHub 실행 | 확인 결과 |
| --- | --- |
| [CI 34305410646](https://github.com/edwardkim/rhwp/actions/runs/34305410646) | Full 성공. lint·Native Skia·frontend package·A/B/C/D tests·Build & Test 성공 |
| [CodeQL 34305410613](https://github.com/edwardkim/rhwp/actions/runs/34305410613) | Rust/Python/JS-TS Analyze 및 CodeQL check 성공 |
| [Adapter 34305410604](https://github.com/edwardkim/rhwp/actions/runs/34305410604) / [Proptest 34305410626](https://github.com/edwardkim/rhwp/actions/runs/34305410626) | 실제 worker 성공 |
| [CI Impact Controller 34305410444](https://github.com/edwardkim/rhwp/actions/runs/34305410444) | 성공, exact head의 CI Impact Policy status도 SUCCESS |
| [Gym 34305410409](https://github.com/edwardkim/rhwp/actions/runs/34305410409) | contracts 성공, Full Gym benchmark validation SKIPPED |
| [Oracle 34305404821](https://github.com/edwardkim/rhwp/actions/runs/34305404821) | sparse 제품 빌드 실제 성공, compareExit=0, verdict=completed |

전체 check rollup은 31 success + 5 skipped = 36개이며 pending/failure는 없었다.
Oracle에서는 실제 PDF 1,212개, LFS pointer 0개를 확인하고 제품 release를 8m 06s에 빌드했다.
판정 artifact는 promotionEligible=true이나 이는 운영 정책의 자격 값이지 제품 정확성 보증이 아니다.
쪽수 비교 1,030쌍은 **999 match, 29 mismatch, 2 error**, unpaired 414였다.
따라서 Oracle 전건 일치라고 주장하지 않는다. 이번 검토는 sparse 입력 분리의 정상 실행을 확인한 것이며,
각 불일치·오류의 원인과 이번 PR 이전 존재 여부는 별도 분류하지 않았다.

## 4. 최종 판정

**승인** — 제품/Gym 의존 분리라는 범위에서 코드·독립 빌드·공개 계약·Full CI 증적이 충족되었다.
Windows/macOS 실행, Gym 전수 평가, Oracle 잔여 불일치 해결을 완료 범위에 포함하지 않는다.

다음은 기록 commit의 승인된 push → 최신 trailing head CI 확인 → 메인테이너 병합 승인 순서다.
이번 PR은 workflow 파일을 포함하므로 review-only 재사용은 trusted controller의 실제 결과로 확인하며
fast-pass를 미리 보장하지 않는다. 같은 코드의 Full run을 근거로 하되 최신 head 검사를 생략하지 않는다.
일반 merge commit 방식의 병합·이슈 close·원격 comment는 별도 승인 전 실행하지 않는다.
이 검토 기록 작성 단계에서는 원격 push·GitHub review/comment·merge·close를 수행하지 않았다.
