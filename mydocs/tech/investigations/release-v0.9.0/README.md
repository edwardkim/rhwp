---
kind: investigation
status: active
canonical: mydocs/tech/investigations/release-v0.9.0/README.md
last_verified: 2026-11-08
---

# 0.9.0 기여자 감사

릴리즈 마감 기준은 `v0.8.7` (`1a76570e833917d15817415a53c09ad61ab3203f`) 이후
`41e08ae19e360a32179d851ca2534c1195a8dced`까지다. 이 기준 이후에는 문서·릴리즈 기록의 저자도 기존 명단과 대조하며, 제품 변경이 추가되면 같은 절차로 갱신한다.

- 4,896개 커밋, 41개 Git identity, 이슈/PR 번호 후보 921개를 조사했다.
- Git 저자·공동 저자, 커밋/검토 파일의 PR 참조, merge SHA의 실제 도달 가능성,
  원 PR 작성자와 cherry-pick 원본을 함께 사용했다. GitHub의 `merged` 검색만으로 수용 여부를 정하지 않는다.
- [원장](contributor-ledger.json): 사람 33명, bot 1개, PR 출처 264건. 사람 identity·번호 판정 미해결 0건.
- [별칭 근거](contributor-overrides.json): 이전 원장의 hash 연결을 재사용하고 새 identity 3건을
  원 PR #8135·#8546·#7713과 보존된 원 SHA/저자로 해결했다. #858은 PR이 아닌 Discussion이다.
- cherry-pick 원본 881개 중 872개 조회 가능, 9개 조회 불가다. 공개 원장에 9개 SHA를 남겼으며
  이들을 검증 완료로 바꾸지 않았다. 통합된 커밋의 저자/공동 저자는 모두 해석됐다.
- GitHub 검색의 이력 포함 판정은 merge SHA가 기준 범위 안에 있는지로 제한한다. 단순히 이 기간에
  언급된 미수용 PR의 작성자를 추가하지 않는다. 간접 통합과 squash/cherry-pick 기여는 Git 저자도 확인한다.
- Dependabot과 AI 공동 저자는 사람 명단에서 분리한다. 원시 이메일·API 응답은 Git 제외 경로에만 저장했다.

재현 명령(저장소 루트):

```sh
python3 scripts/release_contributor_audit.py candidates --repo . --base v0.8.7 \
  --head 41e08ae19e360a32179d851ca2534c1195a8dced --output output/release-v0.9.0/contributor-candidates.json
python3 scripts/release_contributor_audit.py github \
  --candidates output/release-v0.9.0/contributor-candidates.json --repository edwardkim/rhwp \
  --merged-base-ref devel --merged-search 'merged:>=2026-10-06' \
  --output output/release-v0.9.0/contributor-github.json
python3 scripts/release_contributor_audit.py ledger \
  --candidates output/release-v0.9.0/contributor-candidates.json \
  --github output/release-v0.9.0/contributor-github.json \
  --overrides mydocs/tech/investigations/release-v0.9.0/contributor-overrides.json \
  --output mydocs/tech/investigations/release-v0.9.0/contributor-ledger.json --require-resolved
python3 -m unittest scripts.tests.test_release_contributor_audit scripts.tests.test_release_090_preparation
```

GraphQL이 삭제·이관된 번호에 `NOT_FOUND`를 반환해도 같은 응답의 정상 노드를 유지하도록
감사 도구를 보정했다. 해당 누락은 원장의 미해결 상태로 남으며 명시적인 판정이 필요하다.
권한·rate limit·서버 오류는 성공으로 숨기지 않는다. 한국어/영어 CHANGELOG와 두 릴리즈
노트가 원장 명단과 정확히 같은 순서·중복 없음인지는 자동 검사한다.

## 로컬 배포 검증

[검증 요약·artifact 해시](local-validation.json)와
[준비 결과 보고서](../../../report/release_0_9_0_preparation_report.md)에 실제 빌드·패키지 소비·브라우저 결과와 원격 단계의 남은 조건을 기록했다.
