---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-04
pr: 6715
issue: 6711
author: edwardkim
---

# PR #6715 self-review — pr/report 월별 archive 정리

## 결론

**승인.** PR #6715는 #6711의 Stage 2-B로 `mydocs/pr` 119건과 `mydocs/report` 713건을
월별 archive 거버넌스에 따라 처리한다. code candidate
`15d4a8f25a8fb7b081af7179a863393cd6e2764b`를 독립 재검토한 결과 830건은 rename, 2건은
byte-identical archive가 이미 있는 중복 제거이며, 내용이 다른 동명 문서 2건은 suffix 경로로
모두 보존됐다. 예상하지 않은 삭제·경로 범위 누출·신규 링크 또는 canonical 오류는 발견하지 않았다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve
event를 만들지 않는다. 이 review와 오늘할일만 추가한 trailing head의 GitHub Actions,
`MERGEABLE`·`CLEAN`, 최신 `devel` 정합을 다시 확인하고 메인테이너의 merge 승인을 받아야 한다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- `review_impl`은 추가하지 않는다. 승인된 [수행계획](../../plans/task_m100_6711.md)과
  [Stage 2-B 보고서](../../working/task_m100_6711_stage2b.md)가 실행 순서·충돌 원장·검증 계보를
  충분히 고정한다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6715](https://github.com/edwardkim/rhwp/pull/6715) / @edwardkim |
| 관련 이슈 | [#6711](https://github.com/edwardkim/rhwp/issues/6711) (`Refs #6711`) |
| base | `devel@009e30fe1f6812b046862589783c68f890b4d363` |
| code candidate | `15d4a8f25a8fb7b081af7179a863393cd6e2764b` |
| 규모 | 905 files, `+523/-539`, 1 commit |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`·`CLEAN`, candidate checks 완료 |
| reviewer | self PR이므로 지정하지 않음 |

변경 줄 합계와 파일 수가 모두 1,000에 근접하거나 넘으므로 대형 PR 경로를 적용했다. GitHub가
rename을 전혀 인식하지 않는 보수적 경로 수는 1,735개로 PR files API 상한 3,000개보다 작다.
GitHub REST를 100개씩 10페이지로 전수 조회해 905개가 잘리지 않았고 로컬 상태 집계와 일치함을
확인했다.

## 이동·손실 방지 재검토

| 상태 | 수 | 판정 |
| --- | ---: | --- |
| rename | 830 | `pr/report`에서 같은 역할의 `archives/`로 이동 |
| 수정 | 72 | incoming Markdown link·canonical·단계 증적·오늘할일 갱신 |
| 삭제 | 2 | 기존 archive와 SHA-256이 같은 root 중복만 제거 |
| 추가 | 1 | Stage 2-B 결과 보고서 |
| 예상 밖 범위 | 0 | `mydocs` 밖 네 README도 이동 보고서의 incoming link만 변경 |

byte-identical root 두 건은 기존 archive와 SHA-256이 정확히 같다.

| 제거한 root | SHA-256 |
| --- | --- |
| `mydocs/pr/pr_2331_maintainer_review.md` | `085ea428cd97dc3646367e7df33e12ea3e5a21b3c7c6e673aab05337effcb8a2` |
| `mydocs/report/task_m100_1363_report.md` | `d2ed6bfa7e3ef29628ef0acb4599d352be578a3bf6870abfb40ea73a7d3fc2c9` |

내용이 다른 `pr_1844_review.md`, `pr_2370_review.md`는 기존 archive를 덮어쓰지 않고 각각
`pr_1844_review_archived_20260703_a72eb2d.md`,
`pr_2370_review_archived_20260725_100f449.md`로 별도 보존했다.

이동 뒤 root에는 `pr` 1개와 `report` 9개가 남았고 모두 9월 도입 문서다. cutoff 이전 direct
Markdown 잔존은 0개다.

## 링크·canonical·변경 범위 검증

| 검증 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `python3 scripts/check_markdown_links.py` | 609개 문서, 오류 0 |
| 이동 전후 유효 링크 | 6,623개 전부 보존, 손실 0 |
| 정규화된 historical broken link | 이동 전후 2,544건 동일, 신규·소실 0 |
| changed-file link gate | 기준선부터 존재한 5건만 재현, 신규 0 |
| canonical missing 집합 | 이동 전후 4건 동일, 신규·소실 0 |
| `python3 scripts/check_document_metadata.py` | 기존 4개 문서의 누락 16건 재현, 신규 0 |
| Rust·test·Cargo·WASM·workflow·비 Markdown 변경 | 0 |

이동 source 또는 이동 target에 해당하는 링크만 재계산했다. 초기 변환에서 관계없는 디렉터리 링크의
끝 `/`가 정규화되는 과잉 변경을 표본 검사로 발견해 제거했고, 기준선 원문에서 전체 변환을 다시
계산한 뒤 오류 집합을 대조했다.

`mydocs` 밖 변경은 다음 네 README가 이동된 #4100·#5447·#5652 보고서를 계속 가리키도록 한 href
갱신뿐이다.

- `pdf/issue5447/README.md`
- `pdf/issue5652/README.md`
- `samples/issue5447/README.md`
- `samples/issue5652/README.md`

기존 `samples`와 `pdf`의 README 수정은 review-only fast-pass 허용 목록 밖이므로 candidate가 Full
CI를 탄 것은 현행 정책과 일치한다. 런타임 코드·렌더 출력·sample·PDF bytes를 바꾸지 않아 로컬
Cargo/WASM/시각 sweep은 변경 범위 검증에서 생략했다.

## GitHub Actions와 최신 base

candidate SHA에 대해 다음 workflow가 성공했다.

- CI [run 33837538073](https://github.com/edwardkim/rhwp/actions/runs/33837538073)
- CodeQL [run 33837538103](https://github.com/edwardkim/rhwp/actions/runs/33837538103)
- Proptest [run 33837538031](https://github.com/edwardkim/rhwp/actions/runs/33837538031)
- Adapter inter-diff [run 33837538147](https://github.com/edwardkim/rhwp/actions/runs/33837538147)
- trusted Impact Policy [run 33837537878](https://github.com/edwardkim/rhwp/actions/runs/33837537878)과
  최종 집계 [run 33838489853](https://github.com/edwardkim/rhwp/actions/runs/33838489853)

최종 check 집계는 success 29, skipped 3, failure·pending 0이다. `upstream/devel`은 candidate 기준선
`009e30fe1f6812b046862589783c68f890b4d363`에서 전진하지 않았다. `git merge-tree --write-tree`의
결과 tree `43ba78494974d4f375a620fb0e78f2143b6bca0c`는 candidate의 tree와 같다.

## 잔여 위험과 후속 경계

- 저장소 밖 소비자가 옛 root 경로를 사용하면 GitHub history 외부의 link는 깨질 수 있다. 대량
  redirect stub은 만들지 않으며 실제 중요 소비자가 확인될 때 해당 링크나 canonical index를 고친다.
- historical broken link 2,544건, canonical missing 4건, metadata 누락 16건은 이번 이동에서 새로
  만든 결함이 아니다. 범위를 섞어 일괄 정정하지 않았다.
- 이 PR은 `pr/report`만 처리한다. #6711은 Stage 3 `working` batch와 Stage 4 전수 감사·최종 보고가
  끝날 때까지 close하지 않는다.

## Merge 후 contributor PR comment 계획

이 절이 `devel`에 반영된 뒤 [PR #6715](https://github.com/edwardkim/rhwp/pull/6715)와
[이슈 #6711](https://github.com/edwardkim/rhwp/issues/6711)에 다음 사실을 후속 comment로 남긴다.

- 정상 merge commit
  [`c9cc1f7fc77b43acc533b066897c01861b713059`](https://github.com/edwardkim/rhwp/commit/c9cc1f7fc77b43acc533b066897c01861b713059)으로
  병합됐고, 검증 head `e71758c7e19923b35f79881fd28d970332113304`가 `devel`에 포함됐다.
- exact-head [Full CI](https://github.com/edwardkim/rhwp/actions/runs/33860743628),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33860743626),
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/33860743581),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33860743585),
  [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33860743021)가 모두 성공했다.
- controller는 830건 rename을 `fail-closed:rename`으로 분류해 Full lane을 선택했고, Full CI의
  `Frontend package gates`도 6분 57초에 성공했다.
- 로컬 재검증은 canonical 링크 오류 0, cutoff 이전 `pr/report` root 문서 0, 동일본 삭제 2건의
  SHA-256 일치, source·workflow 범위 누출 0을 확인했다. changed-file 링크 5건과 metadata 16건은
  기존 보고서에 기록된 동일 기준선 오류다.
- 코드·renderer·UI·sample·PDF bytes 변경이 없어 시각 검증은 비대상이다.
- #6711은 Stage 3 `working` batch와 Stage 4 전수 감사·최종 보고가 남아 있으므로 OPEN으로 유지하고,
  다음 작업은 최신 `devel`에서 Stage 3 전용 브랜치를 만드는 것이다.

같은 merge commit과 증적을 이미 담은 maintainer comment가 있으면 중복 게시하지 않는다. 이 문서 보완
PR 자체에는 원 PR comment와 이슈 진행 comment를 반복하지 않는다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `15d4a8f25a8fb7b081af7179a863393cd6e2764b`
- trailing 조건: 이 review와 오늘할일만 추가한 최신 head의 GitHub Actions 성공,
  `MERGEABLE`·`CLEAN`, 최신 `upstream/devel` 정합 재확인
- merge 조건: 최신 head SHA 고정과 메인테이너의 별도 merge 승인
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 방식: branch protection을 우회하지 않는 정상 merge commit
- merge 뒤: 최신 `devel` 동기화 후 Stage 3 `working` batch를 시작하며 #6711은 OPEN으로 유지
