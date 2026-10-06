---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-04
pr: 6730
issue: 6711
author: edwardkim
---

# PR #6730 self-review — working 월별 archive 2차 배치

## 결론

**승인.** PR #6730은 #6711 Stage 3-B로 최신 `devel`에 남아 있던 `mydocs/working`
cutoff 이전 Markdown 1,120건을 월별 archive로 이동한다. 최종 code candidate
`e44032cb9d16154c1c8b50aa1f1c184d9b720b86`을 재검토한 결과 목적지 충돌·동일본 중복
제거·divergent 충돌은 모두 0건이며, 이동 뒤 root에는 9월 생성 문서 51건만 남았다.

1,120건은 rename이고 107건은 이동 문서의 상대 링크, 후보 밖 incoming link, 활성 font evidence
소비자와 Stage 3 보고서의 정정이다. 이동 전후 내부 링크 9,224건, 유효 링크 8,671건, historical
broken link 553건이 그대로 유지됐으며 신규 metadata 오류는 없다. 예상하지 않은 삭제·범위 누출,
Rust·Cargo·WASM·workflow 변경도 발견하지 않았다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve
event를 만들지 않는다. 이 review와 오늘할일만 추가한 trailing head의 GitHub Actions,
`MERGEABLE`·`CLEAN`, 최신 `devel` 정합을 다시 확인하고 메인테이너의 별도 merge 승인을 받아야 한다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- `review_impl`은 추가하지 않는다. 승인된 [수행계획](../../plans/task_m100_6711.md)과
  [Stage 3 보고서](../../working/task_m100_6711_stage3.md)가 실행·검증 계보를 고정한다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6730](https://github.com/edwardkim/rhwp/pull/6730) / @edwardkim |
| 관련 이슈 | [#6711](https://github.com/edwardkim/rhwp/issues/6711) (`Refs #6711`) |
| base | `devel@9e8e8bc567cb27b406a945a39637869c3b7fd3b7` |
| code candidate | `e44032cb9d16154c1c8b50aa1f1c184d9b720b86` |
| 규모 | 1,227 files, `+1,296/-1,221`, 1 commit |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`·`CLEAN`, candidate checks 완료 |
| reviewer | self PR이므로 지정하지 않음 |

변경 파일 수와 줄 합계가 1,000을 넘으므로 대형 PR 경로를 적용했다. rename을 전혀 인정하지 않는
보수적 경로 수는 2,347개로 PR files API 상한 3,000개보다 작다. GitHub REST를 100개씩 전수
조회한 1,227개 파일이 로컬 rename-aware diff와 일치했다.

## 이동·손실 방지 재검토

| 상태 | 수 | 판정 |
| --- | ---: | --- |
| rename | 1,120 | `working`에서 같은 역할의 `archives/`로 이동 |
| 수정 | 107 | 상대 링크·incoming link·활성 소비자·Stage 3 증적 갱신 |
| 삭제 | 0 | 중복 제거 대상 없음 |
| 추가 | 0 | 기존 Stage 3 보고서를 현행화 |
| 예상 밖 최상위 범위 | 0 | `mydocs`, `pdf`, `samples`, `scripts`만 변경 |

Git 최초 도입 시각을 최신 base의 root Markdown 1,171개에 다시 적용한 결과 cutoff 이전 후보는
1,120개, 9월 유지 문서는 51개, 판정 불가 문서는 0개였다. 기존 archive 동명 충돌도 없다. 후보
inventory SHA-256은
`a060090a259dd53c89f94a9b6cf6a2159e626c018e0b7d823f03eb7e07e4746d`로 Stage 3 보고서에 고정했다.

## 링크와 활성 경로 소비자

이동 문서 내부 상대 링크와 후보 밖 incoming source 103개의 논리 target을 새 위치에 맞게 원자적으로
갱신했다. 활성 실행 경로는 다음 세 곳만 함께 정정했다.

- `scripts/font_rule_ledger.mjs`: 이후 생성하는 gate evidence 경로 5개
- `scripts/font_typesetting_risk_evidence.mjs`: evidence anchor 경로 2개
- `mydocs/tech/investigations/issue-4962/font_typesetting_risk_contract.json`: evidence 입력 3개와
  이동으로 내용이 바뀐 artifact SHA-256 1개

과거 실행 결과 JSON, CI classifier fixture, source 주석, historical `canonical` 문자열은 현재 파일을
여는 활성 소비자가 아니므로 원문을 보존했다. 완성 문자열과 분할 조립 경로를 별도로 재검색했으며,
cutoff 이전 direct Markdown 잔여는 0건이다.

## CodeQL 경고 #186 판정

candidate의 CI·언어별 CodeQL 분석은 성공했지만, GitHub Advanced Security 종합 check가 기존
`js/insufficient-password-hash` alert #186을 신규 변경 경고로 귀속해 처음에는 실패했다. 지적 위치
`scripts/font_rule_ledger.mjs:104`는 base와 byte-identical하고, 이번 PR이 바꾼 같은 파일의 내용은
834행 이후 evidence 문서 경로 5개뿐이다.

SARIF data-flow를 확인한 결과 CodeQL은 `loadDocumentWithPassword()`의 반환 객체 전체를 암호 유래
데이터로 표시하고, `DocumentInfo.fontsUsed`에서 공유 폰트 대체 cache와 Node runtime snapshot rows를
거쳐 `sha256Text()`까지 전파했다. 실제 `DocumentInfo`에는 암호 필드가 없고 SHA-256 입력은 폰트 규칙
snapshot JSON이다. 브라우저 문서 로딩과 Node 계측도 서로 다른 실행 경계이므로 실제 password hashing
취약점이 아니다.

근거와 구조적 재발 방지는 별도 [#6731](https://github.com/edwardkim/rhwp/issues/6731)에 등록했다.
alert #186에는 같은 근거와 #6731을 남기고 `false positive`로 분류했으며, GHAS 종합 CodeQL check는
`No new alerts`·`success`로 갱신됐다. 실제 암호 보안 검사를 약화시키는 query 제외나 SHA-256 구현 변경은
이 PR에 섞지 않았다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `python3 scripts/check_markdown_links.py` | 609개 canonical 문서, 오류 0 |
| 이동 전후 전체 링크 그래프 | 전체 9,224건·유효 8,671건·historical broken 553건 보존 |
| `python3 scripts/check_document_metadata.py` | 변경하지 않은 기존 4개 문서의 16건만 재현, 신규 0 |
| font ledger·risk Node 계약 | 24 passed |
| risk contract evidence input | 6개 path·SHA-256 일치 |
| `node --check`·JSON parse | 변경한 두 MJS와 contract JSON 통과 |
| 범위 감사 | Rust·Cargo·WASM·workflow 변경 0건 |
| current-base merge tree | `17344276a27826edfa118347b5bce064a9c0c54e`, candidate tree와 동일 |

렌더링·레이아웃, HWP/HWPX sample bytes와 PDF bytes를 바꾸지 않아 시각 검증은 비대상이다. exact
candidate의 Full CI가 성공했으므로 같은 광범위 회귀를 self-review에서 중복 실행하지 않고 링크·관련
Node 계약·merge tree를 재검증했다.

## GitHub Actions와 최신 base

candidate SHA에 대해 다음 workflow와 trusted status가 성공했다.

- CI [run 33877016234](https://github.com/edwardkim/rhwp/actions/runs/33877016234)
- CodeQL [run 33877016244](https://github.com/edwardkim/rhwp/actions/runs/33877016244)
- Proptest [run 33877016034](https://github.com/edwardkim/rhwp/actions/runs/33877016034)
- Adapter inter-diff [run 33877016321](https://github.com/edwardkim/rhwp/actions/runs/33877016321)
- CI Impact Policy Controller
  [run 33877015307](https://github.com/edwardkim/rhwp/actions/runs/33877015307)
- exact-head `CI Impact Policy` status
  [run 33878473628](https://github.com/edwardkim/rhwp/actions/runs/33878473628)

alert #186 분류 뒤 exact-head check runs는 success 28, policy skip 3, failure·pending 0이며 commit
status도 success다. `upstream/devel`은 candidate의 parent
`9e8e8bc567cb27b406a945a39637869c3b7fd3b7`에서 전진하지 않았다.

## 잔여 위험과 후속 경계

- 저장소 밖 소비자가 옛 root 경로를 사용하면 GitHub history 외부 링크는 깨질 수 있다. 대량 redirect
  stub은 만들지 않으며 실제 중요 소비자가 확인될 때 canonical index 또는 해당 링크를 정정한다.
- alert #186은 false positive로 분류했지만 재발 방지 구현은 아직 완료되지 않았다. #6731에서 실제 암호
  흐름의 보호 불변식을 유지하면서 CodeQL 모델·sanitizer·모듈 경계를 별도로 검토한다.
- historical Markdown·metadata 오류는 이번 이동에서 새로 만든 결함이 아니다. 범위를 섞어 일괄
  정정하지 않았다.
- 이 PR은 `working` 2차 배치까지만 처리한다. #6711은 Stage 4 전수 감사와 최종 보고가 끝날 때까지
  close하지 않는다.

## Merge 후 comment 계획

정상 merge commit이 `devel`에 반영되고 merge SHA의 필수 Actions가 성공한 뒤 PR #6730과 이슈
#6711에 다음 사실을 남긴다.

- 정상 merge commit SHA와 검증한 최종 PR head SHA
- 1,120 rename·수정 107건·충돌 및 중복 제거 0건·1,227개 API 파일
- 내부 링크 9,224건과 유효·historical broken 집합 보존, 신규 metadata 오류 0건
- alert #186의 false-positive 근거, #6731 후속 분리와 GHAS check 정상화
- 렌더링·sample·PDF bytes 변화가 없어 시각 검증은 비대상이라는 판정
- #6711은 OPEN으로 유지하고 최신 `devel`에서 Stage 4 전수 감사를 시작한다는 후속 경계

게시 뒤 API로 한글·선두 BOM·`??` 치환과 merge SHA·run URL을 검증한다. 같은 사실을 이미 담은
maintainer comment가 있으면 중복 게시하지 않는다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `e44032cb9d16154c1c8b50aa1f1c184d9b720b86`
- trailing 조건: 이 review와 오늘할일만 추가한 최신 head의 GitHub Actions 성공,
  `MERGEABLE`·`CLEAN`, 최신 `upstream/devel` 정합 재확인
- merge 조건: 최신 head SHA 고정과 메인테이너의 별도 merge 승인
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 방식: branch protection을 우회하지 않는 정상 merge commit
- merge 뒤: 최신 `devel` 동기화 후 Stage 4 전수 감사를 시작하며 #6711은 OPEN으로 유지
