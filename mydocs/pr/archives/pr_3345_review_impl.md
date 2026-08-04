# kevin9327 PR #3345–#3394 통합 검토·구현 기록

## 라우팅과 범위

```text
base route: collaborator_external_pr
modifiers: intake_and_review, local_validation, visual_fixture_evidence,
  multi_pr_update_branch
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

최신 `upstream/devel`의 `19ed96d5ed79621cf81af109708423adc87130b5`에서 사용자에게 보이는
`review/kevin9327-20260726`을 만들고, 작성자 `kevin9327`의 아래 15개 open PR을 순서대로
cherry-pick했다. 원 PR은 모두 `devel` base이고, 이 문서 작성 시점에는 maintainer 보류
review/comment가 없었다. PR 상태·head SHA·CI는 변동 가능하므로 merge 직전에 다시 확인한다.

| PR | contributor 최신 head (작성 시점 참고값) | 통합 판단 |
| --- | --- | --- |
| #3345 | `3b6e39743c91` | 수용 |
| #3347 | `58b1792aa317` | 수용 |
| #3352 | `74a80bd9b07f` | 수용 |
| #3354 | `62e0ea217358` | 메인터너 계약 보정 후 수용 |
| #3356 | `5343eb2a6407` | 수용 |
| #3360 | `d8a5b08f6363` | 수용 |
| #3362 | `de6c85c65959` | 수용 |
| #3364 | `ad7d53889fc9` | 수용 |
| #3369 | `adaf8459666d` | 수용 |
| #3371 | `13b99acfae8b` | 수용 |
| #3374 | `2bc77a930bb7` | 수용 |
| #3376 | `8650f832df17` | 수용 |
| #3384 | `d571e8a7e996` | 메인터너 글자모양 보정 후 수용 |
| #3390 | `0cf47643e233` | 수용 |
| #3394 | `552d643d9074` | 메인터너 문서·증적 보정 후 수용 |

#3391은 별도 PR이 아니라 #3384에 반영된 이슈이므로 독립 cherry-pick 대상이 아니다.

주요 통합·보정 commit은 `7dae51ae8`(공유 CLI 계약), `7537eb772`(예시 JSON 줄끝),
`9cc95feb1`(#3384 최신 #3391 반영), `4ef8546c6`·`e1956b099`·`ac42912f9`(#3394 3개
gallery commit), `72fa44f60`(set-cell 글자모양 보정), `0ed57ab82`(gallery 증적 메타데이터
보정)이다. 원 contributor branch를 rebase·amend·force-push하지 않는다.

## 통합 시 확인·보정한 계약

- #3347 batch `search`와 #3354 단일 `search`는 같은 검색 helper를 거친다. 통합본은
  전체 일치 수를 먼저 계산하고 최대 1,000개만 반환하며, `totalMatchCount`와 `truncated`를
  두 경로 모두에 노출한다. 숨은 절단이 남지 않는다.
- #3374와 #3384가 함께 수정한 CLI/MCP 설명 충돌에서는 기존 MCP HWP 도구를 보존하고,
  `replace-text`와 `set-cell`의 도움말·capabilities·JSON 계약을 모두 유지했다.
- #3384의 최초 기본 검정 스타일 선택은 문서의 다른 셀에서 우연히 같은 "검정" 스타일을
  재사용할 수 있어 대상 셀의 글꼴·크기·자간을 바꿀 위험이 있었다. 메인터너 보정은 **대상
  셀 첫 글자모양을 복제**해 색상·기울임·굵기·밑줄·취소선만 바꾸고, 동등한 전체 스타일이 있을
  때만 재사용한다. 회귀 테스트는 산출 HWP를 다시 읽어 글꼴 ID·크기·장평·자간 보존을 확인한다.
- 표 좌표 `--row`/`--col`의 `u32 → u16` 묵시적 wrap을 입력 단계의 `u16` 상한 오류로
  바꿨다. 잘못된 큰 좌표가 다른 셀을 쓰지 않는다.
- #3376/#3390 텍스트·TSV의 CRLF 및 TSV 끝 공백 탭을 정규화했다. 랭킹의 빈 note 열은
  명시 `-`로 보존해 `git diff --check`가 실제 결함만 보고하게 했다.
- #3394 갤러리는 `kind: report`(manifest 허용 역할 밖)를 `reference`로 고쳤다. 12쪽
  몽타주의 상단에만 있던 한글 글꼴 누락 제목은 잘라냈고, 실제 12쪽·각 페이지 `p1`–`p12`
  증적은 바꾸지 않았다. 설명 제목은 Markdown 본문이 제공한다.

## 검증

- `CARGO_TARGET_DIR=target/review-kevin9327-20260726 CARGO_INCREMENTAL=0`
  `cargo test --profile release-test --tests`: **통과**. 스타일 보정 뒤 다시 실행했다.
- 같은 전용 target의 `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`,
  `git diff --check`: **통과**.
- 집중 회귀: `cargo test --profile release-test --test edit_set_cell_contract` — **5 passed**.
- macOS CLI 실증: `tools/forms/일반기안문_서식.hwpx`의 필드 23개를 예시 JSON으로 채우고
  산출 HWP를 `fields --json`으로 재독했다. `filledCount=23`, intentional empty 값 4개를
  제외한 non-empty 값 19개가 남았다.
- Windows 10 (`win10-ted`, **cmd.exe**)에서 독립 `target/review-kevin9327-20260726`로
  release-test `rhwp.exe` build 성공 후, #3390 `fidelity_compare.py plan 0 2`를 실행했다.
  한컴 공식 PDF 대비 p1 2.55%, p2 10.50%, p3 15.09%였으며 표·문단·머리말의 구조와 흐름을
  실제 PNG로 확인했다. 정확한 이미지는 [#3390 검토 기록](pr_3390_review.md)에 인라인으로 있다.
- #3376 비교 PNG, #3384 K-Startup 원본↔작성본, #3394 12쪽 갤러리를 실제로 열어 확인했다.
  fixture 추가가 아니라 기존 form/HWPX·문서 자산과 CLI 변경이므로 새 HWP/HWPX fixture의
  IR field sweep baseline 등록 대상은 아니다. renderer/layout 로직 변경도 아니므로 별도
  visual sweep·WASM build는 이번 통합 PR의 필수 게이트가 아니다.

## 최종 권고와 다음 단계

**15개 모두 메인터너 보정 포함 통합 PR로 수용 가능**하다. 아직 upstream에 push하거나 PR을
만들지 않았으므로, 이 결론은 로컬 통합 branch 기준이다. 작업지시자 승인 뒤에만 임시 upstream
head branch로 push하고 `devel` 대상 통합 PR을 만든다. 그 PR의 최신 head CI가 성공한 뒤 merge
승인을 다시 받고, merge 후에만 원 PR 15개를 close하며 각 `Closes #…` 이슈의 실제 close 상태와
감사 comment를 확인한다.
