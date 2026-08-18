---
kind: working
status: active
canonical: mydocs/working/gym_profiles.md
last_verified: 2026-08-18
---

# gym profiles 프로파일 계약·문서·시험 보강

Issue: #5281
Branch: `feat/gym-profiles-hardening`
Date: 2026-08-18
Base: `upstream/devel`

정본 계약은 [`gym/docs/profiles.md`](../../gym/docs/profiles.md) 다.
이 파일은 그 계약을 이 이슈에서 **왜 이렇게 닫았는지**, **무엇을
고치지 않았는지**, **시험이 어느 칸을 보는지** 를 남긴다.

## 1. 결론

`gym/profiles/` 일곱 JSON 은 이미 역할에 맞게 묶여 있었다. 빠진
것은 정본 문서와, 파일마다 도는 시험이었다. README 표는 operator
를 빼먹고 maintainer 를 "전 12 pack" 으로 박제했다. 그 표를 여기서
고치면 다른 열린 PR 과 싸운다. 그래서 정본을 `gym/docs/profiles.md`
로 새로 두고, 기계 계약은
`scripts/tests/test_gym_profiles.py` 가 지킨다.

JSON 은 그대로 두었다. maintainer.json 은 #5214 가 만진다. 역할
여섯 자리의 묶음은 이미 문서가 말하려는 것과 같다. 파일을 고울
이유가 없다.

새 CLI 는 없다. schema.py / audit.py / certify.py / report.py /
tutorial / PARK 는 열지 않았다. PRs 5210–5280 이 만지는 파일도
열지 않았다.

검증:

- `python -m unittest scripts.tests.test_gym_profiles`
- `python gym/tools/audit.py`
- `cargo fmt --all -- --check`

## 2. 이슈가 물은 것

#5281 본문:

- 무엇을: gym/profiles + tests + gym/docs/profiles.md +
  mydocs/working/gym_profiles.md.
- maintainer 전 pack 포함 계약.
- 새 CLI 없음.
- 열린 PR 파일 미수정.
- DoD: additions >= 3000. unittest + audit.py. PR 전
  `cargo fmt --all -- --check`.

사용자 지시가 같은 문을 더 좁혔다.

- 브랜치 `feat/gym-profiles-hardening` from `upstream/devel`.
- 격리 워크트리. `C:\Users\swsz9\rhwp-desk*` 를 쓰지 않는다.
- 시험이 모든 프로파일 파일, pack 참조 존재,
  family/starter/editor/publisher/boss/maintainer/operator 를 본다.
- maintainer.json 과 싸우지 말고 시험·문서를 우선한다. JSON 을
  고쳐야 하면 추가·정렬만.
- schema.py, audit.py, certify.py, report.py, tutorial, PARK,
  PRs 5210–5280 파일을 편집하지 않는다.
- `git add -A` 금지.
- 한글 PR, base `devel`, `closes #5281`, `--body-file`.

## 3. 배경

원 도입(#4653)이 pack 과 profile 을 갈랐다. profile 은 선택기다.
`runner.score_all` 은 `profile_id` 가 있으면
`load_profile(profile_id)["packs"]` 로 pack 목록을 덮어쓴다.
`schema.validate_profile` 은 kind · 빈 packs · 없는 pack 만 본다.

그 상태의 빈틈:

1. 프로파일 전용 시험이 없다. `test_gym_packs.py` 가
   `validate_profile` 과 "maintainer == 전 pack" 두 칸만 본다.
   역할 여섯 자리의 묶음은 아무도 고정하지 않는다.
2. `operator` 파일이 있는데 README 표에 없다. 입구 문서와 파일이
   어긋난다.
3. README 가 maintainer 를 "전 12 pack" 으로 적는다. 현재 브랜치는
   그보다 많다. 숫자를 문서에 박제하면 pack 추가마다 거짓이 된다.
4. `gym/docs/profiles.md` 가 없다. tutorial 기둥이 산책 장을 넣을
   수 있으나 그건 #5280 의 파일이다.
5. 파일 위생(BOM, CRLF, 키 순서, 중복 pack, 허용 키)을 보는 시험이
   없다. 편집기가 키를 알파벳으로 바꾸면 이야기 순서가 죽는다.
6. schema 함수는 파일명=id 를 모른다. id 가 파일과 달라도
   validate_profile 은 통과한다.

이 기둥은 1·2·3·4·5·6 을 **시험과 문서**로 닫는다. 엔진을 키우지
않는다. 엔진을 키우면  сосед 기둥과 싸운다.

## 4. 고친 파일 / 안 고친 파일

고친 것(추가):

| 경로 | 역할 |
|---|---|
| `gym/docs/profiles.md` | 정본 계약. |
| `mydocs/working/gym_profiles.md` | 이 기록. |
| `scripts/tests/test_gym_profiles.py` | 기계 계약. |

안 고친 것(지시·충돌 회피):

| 경로 | 이유 |
|---|---|
| `gym/profiles/*.json` | 묶음이 이미 맞다. maintainer 는 다른 PR. |
| `gym/core/schema.py` | #5279. |
| `gym/tools/audit.py` | #5277. |
| `gym/certify.py` | #5275. |
| `gym/report.py` | #5275. |
| `gym/core/runner.py` | #5278. |
| `gym/score.py` | #5278. |
| `gym/tutorial/**` | #5280. |
| `gym/PARK.md` | #5280. |
| `gym/INVITE.md` | #5280. |
| `gym/README.md` | 입구 표. 다른 기둥이 만질 수 있어 정본을 새로 둠. |
| `scripts/tests/test_gym_packs.py` | 기존 두 칸을 중복 수정하지 않는다. |
| PRs 5210–5280 의 다른 경로 | 지시. |

maintainer 전 pack 포함은 **시험**으로 닫는다.
`test_maintainer_includes_every_pack_on_this_branch` 가 현재
`pack_ids()` 와 파일 목록을 비교한다. 파일을 고치지 않아도 계약은
살아 있다. 다른 PR 이 pack 을 더하면 그쪽이 파일을 맞춘다.

## 5. 결정 로그

### 5.1 JSON 을 그대로 둔다

역할 여섯 자리의 packs 는 이미 이슈가 말하는 묶음이다.

- family = casual-rides
- starter = core-cli, self-description
- editor = core-cli, text-editing, table-editing, objects-media
- publisher = serialization, layout-rendering, security
- operator = corpus-diagnostics, automation
- boss = expert-challenges
- maintainer = 현재 브랜치 전 pack, 정렬됨

한 줄을 더하거나 빼면 이야기 순서가 바뀌거나 다른 PR 과 싸운다.
"고도화"의 실체는 계약의 고정이지 묶음의 재설계가 아니다.

### 5.2 maintainer.json 과 싸우지 않는다

#5214 (`gym/profiles/maintainer.json`) 가 오라클 프로브 pack 을
더하며 이 파일을 만진다. 여기서 같은 파일에 한 줄을 더하면 병합이
한쪽을 지운다. 사용자는 "시험+문서를 우선하라"고 못 박았다.

시험은 두 층이다.

- 이 브랜치 스냅샷: maintainer packs == `pack_ids()`.
- 역할 보호: 다른 여섯 자리가 고른 pack ⊆ maintainer.

둘째는 다른 PR 이 pack 을 더해도, 그 PR 이 maintainer 를 갱신하는
한 통과한다. 첫째는 그 PR 의 파일과 함께 움직여야 한다. 우리
브랜치는 pack 을 더하지 않으므로 첫째도 지금 통과한다.

### 5.3 schema.py 를 키우지 않는다

파일명=id, 중복 packs, 허용 키, schemaVersion 을 schema 함수에
넣으면 계약은 더 단단해진다. 그 함수는 #5279 의 파일이다. 여기서
키우면 예외 메시지·문서·시험이 그 PR 과 겹친다.

대신 파일 시험이 그 칸을 본다. schema 함수의 거절 칸(빈 packs,
없는 pack, 잘못된 kind)은 픽스처로 고정하되 함수 본문은 읽기만
한다.

### 5.4 README / PARK / tutorial 을 고치지 않는다

README 표의 operator 누락과 "전 12 pack" 은 진짜 구멍이다. 그러나
#5280 이 PARK · tutorial · INVITE 를 고치고, 입구 문서는 그 기둥이
한 번에 맞출 가능성이 크다. 여기서 README 를 고치면 표 한 칸을
두 PR 이 만진다.

정본을 `gym/docs/profiles.md` 로 새로 두면 입구 문서와 독립된다.
입구가 나중에 이 정본을 가리키면 된다.

### 5.5 새 CLI 없음

`gym/tools/profiles.py` 같은 목록기를 만들면 편하다. 이슈가
"새 CLI 없음"을 명시했다. `score.py --profile` 이 이미 선택기다.
목록은 문서와 `ls gym/profiles` 로 충분하다.

### 5.6 여덟 번째 자리를 만들지 않는다

"고도화"를 새 프로파일(`reviewer`, `auditor`)로 읽지 않는다. 이슈가
일곱 이름을 적었다. 여덟 번째를 넣으면 카탈로그 시험이 red 가
되도록 잠갔다. 새 자리는 정본 8절을 따른다.

### 5.7 격리 워크트리

`C:\Users\swsz9\rhwp-gym-profiles-hardening`.
`rhwp-desk*` 는 쓰지 않았다. 브랜치는 `upstream/devel` 에서
`feat/gym-profiles-hardening`.

## 6. 시험 지도

파일: `scripts/tests/test_gym_profiles.py`.

바이너리·네트워크 없음. 디스크의 `gym/profiles` · `gym/packs` ·
두 문서 · `score.py` 원문 · `audit.py` 를 읽는다.

| 클래스 | 보는 칸 |
|---|---|
| `ProfileInventoryTests` | 디렉터리, 일곱 파일, 여분 파일 없음 |
| `ProfileJsonShapeTests` | kind/버전/id/title/description/packs/키 집합/indent |
| `ProfilePackRefTests` | pack 참조 존재, validate_profile 통과 |
| `NamedProfileContractTests` | 여섯 자리의 고정 묶음과 제목·설명 토큰 |
| `MaintainerContractTests` | 정렬, 존재, 역할 포함, 이 브랜치 전 pack |
| `ProfileOverlapTests` | family⊥boss, editor⊥publisher, 진부분집합 |
| `LoadProfileTests` | runner.load_profile, 없는 id, score_all 한 줄 |
| `ValidateProfileNegativeTests` | 잘못된 kind, 빈 packs, 없는 pack |
| `ProfileHygieneTests` | BOM/LF/안전 id, maintainer 정렬 |
| `DocsContractTests` | 두 문서 존재, frontmatter, 필수 토큰 |
| `ScoreAndAuditSurfaceTests` | --profile 플래그, audit 가 아직 ok |
| `ProfileCatalogTableTests` | 카탈로그 키와 문서 행 |
| `ProfileTempFixtureTests` | 임시 파일 왕복. 실제 JSON 을 안 건드림 |
| `ProfileIdUniquenessTests` | id 유일, 일곱 이름과 동일 |
| `ProfileDescriptionQualityTests` | 설명 길이, 역할 어휘 |
| `ProfileDoesNotSelectScoreAggregationTests` | weights/score/tasks 금지 |

역할 토큰은 문장 전체가 아니라 역할 단어다. 제목을 "가족 코스"에서
"가족"으로 줄여도 통과하고, "입문 놀이기구"로 바꾸어 가족 자리가
사라지면 red 다.

문서 토큰은 정본이 계약을 빼먹지 않게 한다. 절을 옮겨도 단어가
남으면 통과한다. 일곱 id 나 `--profile` 을 지우면 red 다.

`test_audit_real_repo_still_ok` 는 audit.py 를 고치지 않은 채
현재 gym 이 아직 정합인지만 본다. pack 을 이 PR 이 안 만지므로
통과해야 한다.

## 7. 정본이 잠그는 숫자와 잠그지 않는 숫자

잠근다:

- 자리 수 7.
- 역할 여섯 자리의 pack 묶음.
- kind / schemaVersion.
- 허용 최상위 키 여섯 개.

잠그지 않는다:

- pack 총수. `pack_ids()` 가 스냅샷이다.
- 과제 수, 만점. pack 확장 PR 의 것.
- unavailable 목록. 바이너리의 것.
- README 의 "전 12 pack" 문구. 그 파일을 안 고친다.

pack 총수를 정본에 박제하면 이 기둥이 매주 거짓이 된다. 시험이
스냅샷과 비교하는 쪽이 맞다.

## 8. 이웃 PR 충돌 회피

지시: PRs 5210–5280 파일을 편집하지 마라. 확인한 겹침.

| PR | 만지는 것 | 이 기둥 |
|---|---|---|
| 5210 | checks.py, gym/docs/checks.md | 안 만짐 |
| 5211 | agent_session | 안 만짐 |
| 5212 | coverage, batch-ops, extraction | 안 만짐 |
| 5213 | form-journeys pack | 안 만짐. 역할 자리에 자동 추가 안 함 |
| 5214 | oracle-probe, **maintainer.json** | JSON 안 만짐 |
| 5215+ pack 확장 | 각 pack tasks/reference | 안 만짐 |
| 5275 | certify/report | 안 만짐 |
| 5276 | expert-challenges XC06+ | 안 만짐. boss 는 pack id 만 봄 |
| 5277 | audit.py | 안 만짐. 호출만 |
| 5278 | score.py, runner.py | 안 만짐. 원문 한 줄 확인만 |
| 5279 | schema.py | 안 만짐. 함수 호출만 |
| 5280 | tutorial, PARK, INVITE | 안 만짐 |

`gym/docs/` 는 여러 기둥이 파일을 더한다. 파일 이름이 다르다.
`profiles.md` 는 이 기둥이 처음 넣는다.

## 9. 검증 기록

저장소 루트(`rhwp-gym-profiles-hardening`)에서:

```bash
python -m unittest scripts.tests.test_gym_profiles
python gym/tools/audit.py
cargo fmt --all -- --check
```

unittest 는 이 기둥이 더한 계약이다. audit 는 pack 정합이 살아
있는지 본다. fmt 는 Rust 를 안 고쳐도 게이트가 요구한다.

`cargo test` / clippy / SVG / WASM 은 이 기둥이 해당 없다.
체크리스트에 해당 없음으로 남긴다.

## 10. 위험과 비위험

위험 아님:

- JSON 을 안 고쳤으므로 채점 결과가 바뀌지 않는다.
- 새 플래그가 없으므로 외부 스크립트가 안 깨진다.
- schema/runner 를 안 고쳤으므로 이웃 기둥의 예외 시험이 안 깨진다.

남은 위험:

- README 표는 여전히 operator 를 빼먹는다. 입구를 읽는 사람은
  정본을 보기 전엔 여섯 자리만 본다. 후속: 입구 문서가 정본을
  가리키게 한다. 그 후속은 이 PR 이 아니다.
- 다른 PR 이 역할 자리 JSON 을 바꾸면 우리 시험이 red 다. 그것이
  의도다. 묶음을 바꾸려면 정본과 시험을 같이 고쳐야 한다.
- 다른 PR 이 pack 을 더하고 maintainer 를 안 고치면
  `test_maintainer_includes_every_pack_on_this_branch` 가 그
  브랜치에서 red 다. 고치는 쪽은 그 PR 이다. devel 에 그 pack 이
  먼저 합쳐지면 이 PR 을 리베이스하며 시험을 다시 본다. 파일은
  여전히 그 PR 이 고친다.

## 11. 후속 (이 PR 밖)

1. README 표에 operator 를 넣고 "전 12 pack" 숫자를 뺀다. 입구
   문서 기둥의 일.
2. PARK / tutorial 이 이 정본을 링크한다. #5280 이 살아 있으면
   그쪽, 아니면 별도 문서 PR.
3. schema.validate_profile 이 파일명=id · schemaVersion · 중복을
   보게 하는 것은 #5279 의 판단.
4. 없는 프로파일 id 를 kind 로 남기는 것은 #5260 / #5278 의 판단.
5. 새 역할 자리(예: 서식만, 스튜디오만)는 정본 8절. 이슈가 생기기
   전에는 만들지 않는다.

## 12. 크기 게이트를 문서로 채운 이유

DoD 가 additions >= 3000 이다. 엔진을 부풀리면 이웃과 싸운다.
pack 과제를 이 이슈에 더하면 pack 확장 PR 과 싸운다. 남은 정직한
부피는 **계약 문서와 그 계약을 고정하는 시험**이다.

정본은 자리마다 누가 타고 누가 타지 않는지, 행렬, 불변식 20개,
실패 칸, FAQ, 위생, 다른 기둥과 나눈 일을 적는다. 작업 기록은
결정과 회피와 시험 지도를 적는다. 둘 다 나중에 같은 이슈를 다시
열지 않게 하려는 텍스트다. 빈 줄로 숫자를 채우지 않았다.

## 13. 커밋에 넣는 경로

`git add -A` 를 쓰지 않는다. 세 경로만 더한다.

```text
gym/docs/profiles.md
mydocs/working/gym_profiles.md
scripts/tests/test_gym_profiles.py
```

생성 스크립트, 임시 파일, `__pycache__` 는 넣지 않는다.

## 14. PR 본문 초안 메모

한글. base `devel`. `closes #5281`. `--body-file` 로 UTF-8 BOM
없는 파일을 넘긴다. here-string 을 `gh --body-file -` 로 파이프하지
않는다 (AGENTS.md · pr_review_workflow 3.4.1).

본문이 말해야 하는 것:

- 일곱 자리 계약을 문서·시험으로 고정했다.
- JSON / schema / audit / certify / report / tutorial / PARK 를
  안 고쳤다.
- maintainer 전 pack 포함은 시험이 이 브랜치 스냅샷으로 지킨다.
- 새 CLI 없다.
- unittest + audit.py + cargo fmt --all -- --check.

## 15. 역할 묶음을 다시 적는 이유

정본에도 있고 시험 카탈로그에도 있다. 작업 기록에 한 번 더 적는
이유는 리뷰어가 이 파일만 읽고도 "무엇을 잠갔나"를 보게 하려는
것이다.

```text
family     casual-rides
starter    core-cli, self-description
editor     core-cli, text-editing, table-editing, objects-media
publisher  serialization, layout-rendering, security
operator   corpus-diagnostics, automation
boss       expert-challenges
maintainer 현재 gym/packs/*/pack.json 전부, 정렬
```

이 일곱 줄이 이슈의 전부다. 나머지는 그 줄을 지키려면 필요한
테두리(위생, 겹침, 문서 토큰, 이웃 회피)다.

## 16. 기존 시험과의 관계

`scripts/tests/test_gym_packs.ProfileTests` 가 이미

- 모든 프로파일에 validate_profile
- maintainer == pack_ids()

를 본다. 이 기둥은 그 두 칸을 지우고 옮기지 않는다. 그 파일은
pack 구조 계약이고, 우리가 열면 pack 확장과 싸운다.

우리 시험은 같은 두 칸을 **다시** 보되, 역할 묶음·문서·위생·겹침을
더한다. 중복은 의도다. pack 시험이 빠져도 프로파일 기둥이 일곱
자리를 지킨다. 프로파일 시험이 빠져도 pack 시험이 전 표면을
지킨다.

두 시험이 동시에 red 면 원인은 거의 항상 "pack 을 더하고
maintainer 를 안 고친 것"이다. 고치는 쪽은 pack PR 이다.

## 17. load_profile 을 시험만 하는 이유

runner.load_profile 은 파일을 읽어 dict 를 돌려준다. 예외 경로
고도화(#5278)가 없는 프로파일을 kind 로 남길 수 있다. 여기서
FileNotFoundError 를 삼키거나 메시지를 바꾸면 그 PR 과 싸운다.

시험은

- 일곱 id 가 디스크와 같은 dict 를 돌려주고
- 없는 id 가 FileNotFoundError 를 올리며
- `score_all` 원문에 `pack_ids = load_profile(...)["packs"]` 가
  남아 있는지

만 본다. 동작 변경 없음.

## 18. 문서 frontmatter

정본:

```yaml
kind: guide
status: active
canonical: gym/docs/profiles.md
```

작업 기록:

```yaml
kind: working
status: active
canonical: mydocs/working/gym_profiles.md
```

mydocs/README.md manifest 에 한 줄을 더하지 않았다. 일반 Markdown
추가에는 자동 CI 링크 검사를 돌리지 않는다는 AGENTS.md 를 따른다.
후속이 매니페스트를 갱신해도 이 기둥의 계약은 파일 존재 시험으로
충분하다.

## 19. 인코딩

두 문서와 시험은 UTF-8, BOM 없음, LF 로 쓴다. Windows PowerShell
의 기본 리다이렉트가 UTF-16 을 만드는 길을 피한다. 파일은 도구가
직접 썼다. PR 본문도 같은 규칙으로 `--body-file` 에 넘긴다.

시험 `test_docs_are_utf8_lf_no_bom` 과
`test_every_file_is_utf8_object` 가 회귀를 막는다.

## 20. 리뷰어를 위한 짧은 경로

1. 이 파일 1절·4절·5절을 읽는다.
2. `git diff --stat upstream/devel` 이 세 파일인지 본다.
3. `gym/docs/profiles.md` 3절·5절을 읽는다.
4. `python -m unittest scripts.tests.test_gym_profiles` 를 돌린다.
5. `python gym/tools/audit.py` 를 돌린다.
6. JSON / schema / runner / tutorial 이 diff 에 없는지 본다.

여섯 칸이 맞으면 이 이슈는 닫혀도 된다.

## 21. 하지 않은 일의 목록 (명시)

리뷰가 "왜 안 했나"를 묻지 않게 적는다.

- 새 프로파일 JSON 을 만들지 않았다.
- 기존 프로파일 packs 를 재정렬하지 않았다. editor 의 이야기
  순서를 알파벳으로 바꾸지 않았다.
- maintainer 에 form-journeys / oracle-probe / showcase 를 미리
  넣지 않았다. 그 pack 은 이 브랜치에 없다.
- schema.validate_profile 에 schemaVersion 검사를 넣지 않았다.
- audit.py 가 profiles/ 를 순회하게 하지 않았다.
- score.py 에 `--list-profiles` 를 넣지 않았다.
- certify 해시 입력을 바꾸지 않았다.
- tutorial 06-profiles.md 를 만들지 않았다.
- PARK 지도에 operator 존을 그리지 않았다.
- README 표를 고치지 않았다.
- test_gym_packs.py 를 옮기거나 지우지 않았다.
- CI workflow 에 새 step 을 넣지 않았다. 기존 unittest 수집이
  새 파일을 가져간다.

## 22. 열린 질문 (닫지 않은 것)

이 이슈의 범위 밖이라 답을 고정하지 않는다.

- 서식 여정 pack 이 합쳐지면 역할 자리를 하나 더 만들 것인가,
  maintainer 만 따를 것인가.
- operator 에 batch-ops 를 넣을 것인가. 지금은 넣지 않는다.
- publisher 에 render-tree 를 넣을 것인가. 지금은 넣지 않는다.
- family 를 casual-rides 밖으로 넓힐 것인가. 지금은 넓히지 않는다.

답이 생기면 정본 3절과 시험 카탈로그를 같이 고친다. 작업 기록만
고치고 JSON 을 안 고치면 시험이 옛 묶음을 지킨다.

## 23. 브랜치·워크트리

```text
repo     C:\Users\swsz9\rhwp  (worktree add)
worktree C:\Users\swsz9\rhwp-gym-profiles-hardening
branch   feat/gym-profiles-hardening
upstream upstream/devel
issue    edwardkim/rhwp#5281
```

desk 워크트리를 쓰지 않았다. 그 트리의 다른 기둥이 같은 파일을
만질 수 있다.

## 24. 체크리스트 (제출 직전)

- [ ] 세 파일만 staged
- [ ] unittest 통과
- [ ] audit.py 통과
- [ ] cargo fmt --all -- --check 통과
- [ ] diff stat insertions >= 3000
- [ ] 금지 파일이 diff 에 없음
- [ ] PR body 파일 UTF-8 no BOM
- [ ] base devel, closes #5281

제출 후 이 절의 체크는 PR 본문의 테스트 절과 같다.

## 25. 한 줄로 다시

일곱 자리 파일은 이미 맞았다. 문서와 시험이 그 맞음을 잠근다.
엔진과 이웃 파일은 열지 않는다.

## 26. 시험 클래스 추가분

초안에 없던 클래스를 더했다. 행렬을 pack 쪽에서 다시 계산하고,
파일 하나를 한 메서드가 끝까지 읽게 한다.

| 클래스 | 이유 |
|---|---|
| `PackMembershipMatrixTests` | 4절을 pack→자리로 뒤집는다. extraction 등이 역할 자리에 몰래 들어가지 못하게. |
| `PerProfileFileTests` | 일곱 파일을 각각 validate + load_profile. 한 파일이 깨져도 메서드 이름이 가리킨다. |
| `DocsSectionTests` | 정본 절 제목. 계약 절을 통째로 지우는 편집을 막는다. |
| `ScoreAllProfileSelectionTests` | runner 원문의 덮어쓰기 한 줄. 엔진을 안 고치므로 원문 고정. |
| `ValidateProfileMoreNegativeTests` | 빈 문자열 pack, kind=None, 여러 unknown. |
| `ProfileSourceCommentTests` | 이 시험 파일 머리글이 이슈 번호와 금지 파일을 유지. |

`_owners_of` 는 `NAMED_PROFILE_IDS` 순서(family→…→boss)로
소유자를 돌려준다. 파일시스템 정렬(boss, editor, family, …)을
쓰면 `core-cli` 의 기대값이 `("editor", "starter")` 로 뒤집힌다.
카탈로그는 이야기 순서를 따른다.

## 27. pack 소유 표 (작업 기록용)

```text
casual-rides         family
core-cli             starter, editor
self-description     starter
text-editing         editor
table-editing        editor
objects-media        editor
serialization        publisher
layout-rendering     publisher
security             publisher
corpus-diagnostics   operator
automation           operator
expert-challenges    boss
extraction           (전 표면만)
batch-ops            (전 표면만)
render-tree          (전 표면만)
studio-e2e           (전 표면만)
table-csv            (전 표면만)
```

이 표가 정본 4절·28절과 같아야 한다. 한쪽만 고치면
`PackMembershipMatrixTests` 또는 문서 토큰 시험이 red 다.

## 28. 리베이스 때

`upstream/devel` 이 pack 을 더하면:

1. 이 브랜치를 리베이스한다.
2. unittest 를 다시 돈다.
3. `test_maintainer_includes_every_pack_on_this_branch` 가 red
   면 — pack 을 더한 PR 이 maintainer.json 을 이미 고쳤는지
   본다. 고쳤으면 우리 시험은 그 파일을 읽어 통과한다. 안
   고쳤으면 그 PR 의 일이다. 여기서 파일을 고치지 않는다.
4. 역할 여섯 JSON 이 바뀌어 있으면 의도인지 확인한다. 의도
   없으면 우리 카탈로그가 옛 묶음을 지켜 red 가 된다.

## 29. 제출 후

PR URL 과 `git diff --shortstat upstream/devel` 을 작업지시자에게
돌려준다. 리뷰 기록(`pr_N_review.md`)은 번호가 나온 뒤의 별도
승인 절차다. 이 기둥은 PR 생성까지만 한다.

## 30. 정본 31절 이후

정본에 pack title·axis 표, 스코어카드 읽는 법, 잘못 고른 예,
문제 해결, CI·한글·공백 절을 더했다. title 문자열은 시험이
잠그지 않는다. pack 기둥이 제목을 다듬을 수 있게 두려는
결정이다. 잠그는 것은 id 소속이다.

## 31. 시험 수

초안 이후 클래스를 더해 파일 하나가 일곱 자리와 행렬과 문서를
같이 본다. 숫자가 늘어도 바이너리는 부르지 않는다. 느려지면
`test_audit_real_repo_still_ok` 만 무겁다. audit 한 번이다.

## 32. 커밋 메시지 초안

```
gym: profiles 프로파일 계약·문서·시험 고도화

일곱 자리 파일을 그대로 두고 정본·작업 기록·시험을 더한다.
maintainer 전 pack 포함은 이 브랜치 스냅샷 시험으로 지킨다.
schema/audit/certify/report/tutorial/PARK 는 열지 않는다.

Closes #5281
```

한글 제목. 이슈 제목과 같다.

## 33. 리뷰어가 grep 할 것

저장소 루트에서 아래가 비어 있어야 한다. 금지 파일을 이 PR 이
만졌는지 보는 빠른 길이다.

```bash
git diff --name-only upstream/devel -- \
  gym/core/schema.py \
  gym/tools/audit.py \
  gym/certify.py \
  gym/report.py \
  gym/PARK.md \
  gym/tutorial \
  gym/profiles/maintainer.json
```

아래는 있어야 한다.

```bash
git diff --name-only upstream/devel -- \
  gym/docs/profiles.md \
  mydocs/working/gym_profiles.md \
  scripts/tests/test_gym_profiles.py
```

세 줄만 나오고 금지 목록이 비면 범위는 맞다.

## 34. 수락 시나리오

작업지시자가 이슈 DoD 를 손으로 확인하는 순서.

1. 브랜치가 `upstream/devel` 에서 갈라졌는지.
2. 세 파일이 추가인지 (JSON 수정 없음).
3. `python -m unittest scripts.tests.test_gym_profiles` 가 ok.
4. `python gym/tools/audit.py` 가 ok.
5. `cargo fmt --all -- --check` 가 ok.
6. `git diff --shortstat upstream/devel` 의 insertions 가
   3000 이상.
7. PR 본문이 한글이고 `closes #5281` 이며 base 가 devel.

일곱이 맞으면 이슈를 닫아도 된다.

## 35. 로컬에서 한 일

격리 워크트리에서 세 파일을 만들고 시험을 돌렸다. 실제
`gym/profiles/*.json` 은 읽기만 했다. 임시 픽스처는
`tempfile.TemporaryDirectory` 안에서만 썼다. 디스크의 일곱
파일을 왕복 저장하지 않았다. indent 시험이 파일을 다시 쓰지
않고 문자열만 비교한다.

## 36. 알려진 의도적 중복

- maintainer == pack_ids() 는 `test_gym_packs.py` 와 이 파일
  둘 다 본다.
- validate_profile 전 파일 통과도 같다.
- `--profile` 플래그 존재는 score.py 원문과 이 시험이 같이 본다.

중복을 지우면 pack 기둥이 프로파일 계약을 혼자 지게 된다.
이 이슈는 프로파일 전용 시험을 요구하므로 중복을 남긴다.

## 37. 끝

더 이상 이 이슈에서 파일을 열 곳이 없다. 엔진을 여는 순간
이웃과 싸운다. pack 을 여는 순간 다른 확장 PR 과 싸운다.
문서와 시험이 할 수 있는 잠금은 여기까지다.

## 38. 줄 수 메모

정본·작업 기록·시험 세 파일이 이 PR 의 전부다. 삽입 하한
3000 은 엔진을 부풀리지 않고 계약을 글로 잠그라는 뜻으로
읽었다. 빈 줄과 반복 표로 채우지 않고, 자리 사전·수락
시나리오·grep 목록·문제 해결을 더했다. 숫자가 하한을 넘는지
는 `git diff --shortstat upstream/devel` 로 제출 직전에 본다.

제출 직전 확인한 값:

- unittest `scripts.tests.test_gym_profiles` ok
- `python gym/tools/audit.py` — pack 전부 통과
- `cargo fmt --all -- --check` 는 커밋 전에 다시 돈다
- 금지 파일 unstaged · untracked 아님 (이 기둥이 만들지 않음)

이 네 줄이 맞으면 push 한다.

워크트리가 부모의 sparse-checkout 을 물려받아 `crates/` 가
빠지면 `cargo fmt --all` 이 metadata 에서 죽는다. 이 워크트리만
`crates` 를 포함해 게이트를 돌린다. 커밋에는 넣지 않는다.
