# Task M100 #6584 Stage R1 — v0.8.6 기여자 계보 원장

- **범위**: `v0.8.4` `496333b27d21ddb9114ba9ae340bcb895870c9a7` .. release base
  `063041a2ced54085b5cf94c2e646ac7aa0e1960d`
- **계측일**: 2026-09-02 KST
- **결과**: 사람 20명, bot 1개, 미해결 번호 0건, 미해결 Git 정체성 0건
- **기계 정본**:
  `mydocs/tech/investigations/issue-6584/release_contributor_ledger.json`
- **ledger SHA-256**:
  `934a96927831ec87d2b19db296ea1976111b1f1e815c66291369e2a0c1929c28`

## 1. 결론

v0.8.6 공개 사람 기여자 집합은 아래 20개 credit key로 고정한다.

`chrisryugj`, `coolwithyou`, `davindev`, `dkh0324`, `edwardkim`,
`humdrum00001010`, `JamesPsh`, `jangster77`, `jeong-sik`, `johndoekim`,
`keepYaoung`, `kevin9327`, `kjh0523`, `lpaiu-cs`, `planet6897`, `postmelee`,
`RaghavShubham`, `Shadungi`, `t2c-lab`, `thhan74`

대소문자를 보존한 위 목록을 case-insensitive 정렬하고 줄바꿈으로 이은 집합의 SHA-256은
`169db39bb034abca43b16bae1e6a9d65f127af0e29785e4dcee855e1bed3a2bf`다. Stage R3에서
`CHANGELOG.md`, `CHANGELOG_EN.md`, GitHub 릴리스 노트의 집합을 이 값과 대사한다.

`dkh0324`는 범위 안 직접 Git author로 확인되지만 2026-09-02 현재 같은 이름의 공개 GitHub 계정은
확인되지 않았다. 다른 사람에게 합치면 계보를 훼손하므로 Git author credit key를 그대로 보존하며 공개 문서에서는
GitHub mention인 `@dkh0324`로 만들지 않는다.

## 2. 계측·필터 결과

| 항목 | 결과 | 판정 |
|---|---:|---|
| 범위 커밋 | 2,214 | release base와 일치 |
| mailmap 적용 뒤 Git 정체성 | 33 | author와 `Co-authored-by` 합집합 |
| `#번호` + 신규 PR archive 후보 | 1,906 | 이슈·과거 PR 언급을 포함한 원시 후보 |
| GitHub Issue | 847 | 기여 PR 수에서 제외 |
| GitHub PullRequest 후보 | 1,059 | 언급만 된 PR을 포함 |
| 번호 후보 중 merge commit이 범위 안인 PR | 253 | commit message·archive 출발점 |
| `devel` merged PR 독립 조회 | 261 | merge commit으로 release range 재검증 |
| 독립 조회가 추가로 복원한 PR | 9 | 번호 없는 merge/squash 계보 누락 방지 |
| 최종 고유 PR provenance | 262 | `devel` 261개 + 범위 안 task-branch PR 1개 |
| 범위 merge가 아닌 PR 언급 | 806 | 사람 credit 근거로 사용하지 않음 |
| cherry-pick 원본 SHA | 630 | commit trailer에서 결정론적으로 추출 |
| GitHub에서 조회 가능한 원본 SHA | 629 | 1개는 원본 객체가 조회되지 않음 |
| 원본 SHA의 `associatedPullRequests` | 0 | 닫힌·fork 원본 연결 복원에는 사용할 수 없음 |
| 사람 / bot / AI disposition | 20 / 1 / 1 | 서로 분리 |
| 미해결 PR 번호 / Git 정체성 | 0 / 0 | 종료 게이트 통과 |

PR 번호가 commit message나 review archive에 나타났다는 이유만으로 기여자를 올리면 806개 과거·종속 PR이
과대계상된다. 반대로 그 후보만 사용하면 번호가 본문에 남지 않은 실제 `devel` merge PR 9개가 빠진다. 따라서
`devel` merged PR을 독립 조회하고 merge commit이 정확한 릴리스 커밋 집합에 있는지 다시 판정했다. 후보에서
확인된 범위 안 task-branch PR 1개를 더해 최종 고유 PR provenance는 262개다. maintainer 통합 PR로 흡수된
외부 기여는 원 PR이 GitHub에서 `merged`가 아니더라도 보존된 Git author와 co-author로 복원했다.

GitHub의 `associatedPullRequests`는 629개 조회 가능 cherry-pick 원본에서도 결과를 주지 않았다. 이를
필수 근거로 삼으면 실제 기여자가 누락되므로 원본 SHA 연결은 보조 진단으로만 남기고, release commit에 보존된
author·co-author를 정본 증거로 삼는다.

## 3. 사람별 근거 요약

`PR`은 범위 안 merge commit으로 확인된 PR 수이며, `commit evidence`는 해당 credit key에 귀속된 author 또는
co-author 커밋 증거 수다. PR이 0이어도 commit evidence가 있으면 직접 provenance가 존재한다.

| credit key | PR | commit evidence | identity hash | 비고 |
|---|---:|---:|---:|---|
| `chrisryugj` | 0 | 2 | 1 | 직접 Git author |
| `coolwithyou` | 1 | 2 | 1 | PR + 직접 Git author |
| `davindev` | 0 | 1 | 1 | 직접 Git author |
| `dkh0324` | 0 | 1 | 1 | 직접 Git author, 공개 GitHub 계정 미확인 |
| `edwardkim` | 24 | 527 | 1 | maintainer |
| `humdrum00001010` | 1 | 58 | 3 | 동일인 정체성 통합 |
| `JamesPsh` | 1 | 6 | 2 | 동일인 정체성 통합 |
| `jangster77` | 186 | 656 | 3 | `Taesup Jang`·`TaesupJang` 동일인 통합 |
| `jeong-sik` | 5 | 46 | 1 | PR + 직접 Git author |
| `johndoekim` | 5 | 60 | 2 | author·co-author 정체성 통합 |
| `keepYaoung` | 3 | 12 | 2 | 동일인 정체성 통합 |
| `kevin9327` | 3 | 381 | 3 | author·co-author 정체성 통합 |
| `kjh0523` | 1 | 5 | 1 | PR + 직접 Git author |
| `lpaiu-cs` | 0 | 47 | 1 | 직접 Git author |
| `planet6897` | 16 | 327 | 2 | `Jaeook Ryu`·`jaeook.ryu` 동일인 통합 |
| `postmelee` | 14 | 193 | 2 | author·co-author 정체성 통합 |
| `RaghavShubham` | 0 | 1 | 1 | 직접 Git author |
| `Shadungi` | 1 | 2 | 1 | PR + 직접 Git author |
| `t2c-lab` | 1 | 1 | 1 | PR + 직접 Git author |
| `thhan74` | 0 | 8 | 1 | 직접 Git author |

## 4. 사람 외 disposition

- `dependabot[bot]`: bot 1개로 분리했으며 사람 수에 넣지 않는다. 과거 `dependabot` 표기는 같은 bot으로
  정규화한다.
- AI assistant 공동작성 표기: identity SHA-256
  `882bfb45e29f2077f6b56531b00d578c3b3336729efa268f8cfec2a189d0a6a2`를
  `ai-assistant-coauthor-not-human-release-credit`로 제외했다. 사람 기여자로 세지 않는다.
- 별도 공개 credit을 요구하는 보안 제보자나 익명 disposition은 이 release range에서 확인되지 않았다.

## 5. 재현 절차와 보호 경계

원시 candidate JSON은 공개 Git commit의 이메일을 포함할 수 있으므로 `output/6584/`에만 두고 추적하지
않는다. 추적하는 override와 ledger는 이메일·실명 대신 SHA-256 identity key를 사용한다.

```bash
python3 scripts/release_contributor_audit.py candidates \
  --base v0.8.4 \
  --head 063041a2ced54085b5cf94c2e646ac7aa0e1960d \
  --output output/6584/contributor-candidates.json

python3 scripts/release_contributor_audit.py github \
  --candidates output/6584/contributor-candidates.json \
  --repository edwardkim/rhwp \
  --merged-base-ref devel \
  --merged-search 'merged:>=2026-08-12' \
  --output output/6584/contributor-github.json

python3 scripts/release_contributor_audit.py ledger \
  --candidates output/6584/contributor-candidates.json \
  --github output/6584/contributor-github.json \
  --overrides mydocs/tech/investigations/issue-6584/release_contributor_overrides.json \
  --require-resolved \
  --output mydocs/tech/investigations/issue-6584/release_contributor_ledger.json
```

같은 candidate·GitHub metadata·override 입력으로 ledger를 두 번 생성해 byte-for-byte `cmp`를 통과했다.
fixture unit test 10건과 Python 구문 검사도 통과했다. release base가 전진하면 기존 결과에 덧붙이지 않고 새
범위로 세 입력과 ledger를 모두 다시 생성해야 한다.

## 6. Stage R1 판정

Stage R1은 다음 조건으로 통과한다.

- 공개 사람 credit key 20개 모두 PR 또는 직접 Git provenance가 있다.
- 과거·종속 PR 언급 806개를 기여자로 과대계상하지 않았고 번호가 남지 않은 실제 merge PR 9개도 복원했다.
- 동일인 alias, bot, AI 공동작성 표기를 분리했고 미해결 번호·정체성이 0이다.
- 기계 원장 재실행 결과가 결정론적이다.

다음 Stage R2에서는 이 원장의 262개 PR provenance와 2,214개 커밋을 사용자 영향 변경군으로 분류한다.
