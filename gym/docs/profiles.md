---
kind: guide
status: active
canonical: gym/docs/profiles.md
last_verified: 2026-08-18
---

# gym 프로파일 계약

이 문서는 `gym/profiles/*.json` 일곱 자리의 **선택 계약**이다. 작업
기록·오탐 결정·시험 지도는
[`mydocs/working/gym_profiles.md`](../../mydocs/working/gym_profiles.md)
에 남긴다. 기계 계약은
`scripts/tests/test_gym_profiles.py` 가 고정한다.

프로파일은 pack 을 **고르는** 도구이지 점수를 뭉치는 도구가 아니다.
`python gym/score.py --profile editor` 는 editor 가 선언한 pack 만
채점한다. 총점은 그 pack 들의 합일 뿐이고, 능력 판독은 여전히 pack
별 점수가 한다. 가중치·만점·과제 화이트리스트는 프로파일 JSON 에
없다.

새 CLI 는 없다. `--profile <id>` 는 예전부터 `gym/score.py` 가 받던
자리다. 이 문서와 시험은 그 자리의 **파일 계약**을 고정한다.
`schema.py` · `audit.py` · `certify.py` · `report.py` · `tutorial/` ·
`PARK.md` 는 이 기둥이 고치지 않는다.

## 한 줄 결론

```bash
python gym/score.py --agent <이름> --profile family
python gym/score.py --agent <이름> --profile starter
python gym/score.py --agent <이름> --profile editor
python gym/score.py --agent <이름> --profile publisher
python gym/score.py --agent <이름> --profile operator
python gym/score.py --agent <이름> --profile boss
python gym/score.py --agent <이름> --profile maintainer
```

일곱 자리의 파일은 `gym/profiles/<id>.json` 이다. 파일명과 `id` 가
같고, `kind` 는 `gymProfile` 이며, `packs` 의 모든 id 는
`gym/packs/<id>/pack.json` 을 가진다.

## 1. 왜 프로파일인가

운동장은 pack 으로 능력 축을 가른다. pack 이 열일곱이 되면 "전부
달려라"는 입문도 아니고 편집 시험도 아니다. 처음 온 에이전트에게
`casual-rides` 네 과제만 주고, 문서를 고치는 자리에는
`text-editing` · `table-editing` · `objects-media` 를 주고, 배포
직전 자리에는 `serialization` · `layout-rendering` · `security` 를
준다.

그 선택을 명령줄에 매번 나열하면 빠뜨린다.

```bash
python gym/score.py --agent kim --pack text-editing --pack table-editing --pack objects-media --pack core-cli
```

프로파일은 그 나열을 이름 하나로 고정한다.

```bash
python gym/score.py --agent kim --profile editor
```

선택이 파일로 남으면 시험이 지킬 수 있다. 사람이 README 표만 고치고
JSON 을 안 고치면 시험이 red 가 된다. JSON 만 고치고 문서를 안
고치면 같은 시험이 red 가 된다. 그 쌍이 이 기둥의 전부다.

프로파일이 **하지 않는** 것:

- 점수를 한 숫자로 다시 가중하지 않는다.
- 과제 일부를 빼거나 티어를 바꾸지 않는다.
- 바이너리 요구 명령을 완화하지 않는다. pack 이 unavailable 이면
  프로파일을 써도 unavailable 이다.
- 리더보드 입장을 대신 판정하지 않는다. 입장 봉투는 "채점이 유효하게
  완주했는가"이지 "어느 프로파일인가"가 아니다.

## 2. 스키마

```json
{
  "schemaVersion": "1.0",
  "kind": "gymProfile",
  "id": "editor",
  "title": "편집자",
  "description": "문서를 실제로 고치는 능력 — 본문·표·개체 편집과 그 재검증.",
  "packs": [
    "core-cli",
    "text-editing",
    "table-editing",
    "objects-media"
  ]
}
```

| 키 | 형 | 계약 |
|---|---|---|
| `schemaVersion` | 문자열 | 지금 `"1.0"` 만. |
| `kind` | 문자열 | 지금 `"gymProfile"` 만. `gymPack` 이 아니다. |
| `id` | 문자열 | 파일명 stem 과 같다. `[a-z][a-z0-9-]*`. |
| `title` | 비어 있지 않은 문자열 | 사람용 짧은 이름. |
| `description` | 비어 있지 않은 문자열 | 한 줄로 역할을 가리킨다. |
| `packs` | 비어 있지 않은 문자열 배열 | 각 원소는 존재하는 pack id. 중복 금지. |

허용 키는 위 여섯 개다. `weights` · `score` · `max` · `tasks` ·
`exclude` 를 넣지 않는다. 선택기이지 채점기가 아니기 때문이다.

검증은 `gym.core.schema.validate_profile(profile, pack_ids, errors)`
가 한다. 이 기둥이 그 함수를 고치지 않는다. 시험은 현재 구현이
거절하는 칸(빈 packs, 없는 pack, 잘못된 kind)을 픽스처로 고정한다.

로드는 `gym.core.runner.load_profile(profile_id)` 가
`gym/profiles/<id>.json` 을 UTF-8 로 읽는다. 없는 id 는
`FileNotFoundError` 다. 이 기둥이 그 함수를 고치지 않는다.

`score_all(..., profile_id=...)` 는 프로파일이 있으면
`pack_ids = load_profile(profile_id)["packs"]` 로 덮어쓴다. 그 다음
채점은 pack 과 같다.

## 3. 일곱 자리

파일은 일곱 개다. 여덟 번째 JSON 을 넣으면 시험이 red 다. 새 자리를
만들 때는 `NAMED_PROFILE_IDS` 와 이 문서와 작업 기록을 같이 고친다.

### 3.1 `family` — 가족 코스

```json
"packs": ["casual-rides"]
```

부모님·친구와 함께 도는 입문존만. 키 제한이 없는 회전목마다.
`core-cli` 도 `self-description` 도 넣지 않는다. 도구의 결을 익히는
자리는 `starter` 다. 가족 코스는 "읽고 세기" 만 준다.

누가 타나:

- 운동장을 처음 구경하는 사람.
- 에이전트에게 "한글 문서를 셀 수 있느냐"만 묻는 자리.
- `INVITE.md` 가 가리키는 첫 초대.

누가 타지 않나:

- 명령을 조합해 문서를 고치려는 자리 → `editor`.
- 도구가 스스로를 설명하는지 보는 자리 → `starter`.
- 사다리 완주를 보려는 자리 → `boss`.

측정하는 것: 쪽수·검색·간단 조회처럼 실패해도 안전한 읽기.
측정하지 않는 것: 편집, 변환, 보안, 자동화.

`family` 와 `boss` 는 겹치지 않는다. 입문존과 보스존이 같은 pack 을
품으면 놀이공원 지도가 거짓이다.

### 3.2 `starter` — 입문

```json
"packs": ["core-cli", "self-description"]
```

처음 온 에이전트가 도구의 결을 익히는 최소 코스. `core-cli` 는
조사·추출·편집·검증의 최소 표면이고, `self-description` 은 도구가
스스로를 설명하는 계약이다. 두 개를 같이 도는 이유: 명령을 쓰기
전에 명령이 무엇을 말하는지 읽어야 한다.

`casual-rides` 를 넣지 않는다. 가족 코스와 입문 도구 코스는 다른
축이다. 부모님과 함께 도는 자리와, 에이전트가 capabilities 를 읽는
자리를 한 묶음으로 만들면 둘 다 애매해진다.

`starter` 의 `core-cli` 는 `editor` 에도 들어 있다. 편집자는 입문
코어를 지나온 자리이기 때문이다. `self-description` 은 editor 에
넣지 않는다. 편집 능력과 자기서술 능력은 다른 축이다.

### 3.3 `editor` — 편집자

```json
"packs": ["core-cli", "text-editing", "table-editing", "objects-media"]
```

문서를 실제로 고치는 능력. 본문·표·개체와 그 재검증. `core-cli` 는
탐색과 검증의 바닥이다. 세 편집 pack 만 주고 코어를 빼면 "어디에
있는지 모르는 채 고치기"가 된다.

넣지 않는 것:

- `serialization` · `layout-rendering` · `security` → `publisher`.
- `corpus-diagnostics` · `automation` → `operator`.
- `expert-challenges` → `boss`.
- `table-csv` · `studio-e2e` → 전 표면(`maintainer`)에만. 편집자의
  최소 코스가 스튜디오 e2e 까지 끌고 가지 않는다.

`editor` 와 `publisher` 는 pack 을 공유하지 않는다. 고치는 능력과
내보내기 전 확인은 다른 시험이다. 한 에이전트가 둘 다 필요하면
두 번 돌리거나 `maintainer` 를 탄다.

### 3.4 `publisher` — 배포자

```json
"packs": ["serialization", "layout-rendering", "security"]
```

내보내고 배포하기 전에 확인해야 하는 것들. 형식 왕복, 조판 판정,
은닉·주입·유니코드·PII. 배포 직전 관문이지 편집 코스가 아니다.

`security` 를 editor 에 넣지 않는 이유: 보안 스윕은 고치는 능력과
다른 축이다. 편집자가 배포 직전 스윕까지 자동으로 끌어가면 편집
실패와 보안 실패가 한 점수에 섞인다. 프로파일은 점수를 뭉치지
않더라도, 한 번의 `--profile` 이 두 축을 동시에 열면 사람이 표를
읽을 때 다시 섞는다.

`layout-rendering` 을 editor 에 넣지 않는 이유: 조판 판정은 편집의
결과가 아니라 배포물의 성질이다.

### 3.5 `operator` — 운영자

```json
"packs": ["corpus-diagnostics", "automation"]
```

무더기를 다루고 이상을 좁히는 능력. 코퍼스 스윕·진단·계획·캡슐·
서명·앵커·정산·감사. README 표에 빠져 있던 자리다. 파일은 원래
있었고, 이 기둥이 문서와 시험을 파일에 맞춘다.

`family` · `publisher` · `boss` 와 겹치지 않는다. 운영자는 입문
놀이기구도, 배포 직전 스윕도, 보스 어트랙션도 아니다.

`batch-ops` 를 넣지 않는다. 다문서 대량 처리는 전 표면의 일부이고,
운영자의 최소 코스는 진단과 자동화 사다리다. 대량 처리까지 필요하면
`maintainer` 또는 `--pack batch-ops` 를 더한다.

### 3.6 `boss` — 보스 코스

```json
"packs": ["expert-challenges"]
```

사다리 완주급 고난도만. 한 단만 틀려도 판정이 막히는 자리.
`family` 와 정반대 극단이다. 두 자리가 같은 pack 을 품으면 키
제한이 거짓이다.

역할 다섯 자리(`starter` · `editor` · `publisher` · `operator` ·
`family`)는 `expert-challenges` 를 품지 않는다. 보스는 초대가
아니라 도전이다.

### 3.7 `maintainer` — 메인테이너

전 표면. 검증 사다리와 자기서술까지 포함한 완주 코스.
`packs` 는 **현재 브랜치의 `gym/packs/*/pack.json` 전부**이고,
알파벳 정렬이다.

다른 여섯 자리가 고른 pack 은 모두 maintainer 에 들어 있다.
역할 여섯 자리의 합집합은 maintainer 의 진부분집합이다. 전 표면이
역할 코스의 합과 같으면, 역할에 안 넣은 pack
(`extraction` · `batch-ops` · `render-tree` · `studio-e2e` ·
`table-csv` 등)이 입구를 잃는다.

이 기둥은 `maintainer.json` 을 고치지 않는다. 다른 열린 PR 이
pack 을 더하면 그쪽에서 정렬·추가한다. 시험은 이 브랜치의
`packs/` 스냅샷과 파일이 같은지만 본다. 파일을 여기서 고울 이유가
없다.

정렬 규칙: maintainer 만 알파벳이다. 역할 여섯 자리는 사람이 읽는
순서(코어 → 본문 → 표 → 개체, 변환 → 조판 → 보안, 진단 → 자동화)를
유지한다. 알파벳으로 맞추면 `editor` 가
`core-cli, objects-media, table-editing, text-editing` 이 되어
문서의 이야기 순서가 깨진다.

## 4. 자리 × pack 행렬

아래 표는 역할 여섯 자리의 고정 묶음이다. maintainer 는 현재
브랜치의 전 pack 이라 행을 따로 적지 않는다.

| pack | family | starter | editor | publisher | operator | boss |
|---|---|---|---|---|---|---|
| `casual-rides` | ● | | | | | |
| `core-cli` | | ● | ● | | | |
| `self-description` | | ● | | | | |
| `text-editing` | | | ● | | | |
| `table-editing` | | | ● | | | |
| `objects-media` | | | ● | | | |
| `serialization` | | | | ● | | |
| `layout-rendering` | | | | ● | | |
| `security` | | | | ● | | |
| `corpus-diagnostics` | | | | | ● | |
| `automation` | | | | | ● | |
| `expert-challenges` | | | | | | ● |

역할 여섯 자리에 안 올라간 pack 은 전 표면에서만 고른다.

| pack | 전 표면에만 있는 이유 |
|---|---|
| `extraction` | 조회 축의 확장. 입문·편집·배포의 최소 코스가 아니다. |
| `batch-ops` | 다문서 대량 처리. 운영 최소 코스(진단+사다리)의 다음 층. |
| `render-tree` | 렌더 트리 구조 추출. 조판 판정(`layout-rendering`)과 다른 축. |
| `studio-e2e` | 스튜디오 e2e 에서 파생한 CLI 계약. 편집 최소 코스보다 좁고 깊다. |
| `table-csv` | 표 CSV 왕복. `table-editing` 의 좌표 편집과 다른 왕복 계약. |

다른 PR 이 `form-journeys` · `work-receipt` · `oracle-probe` · `showcase` 같은 pack
을 더하면, 그 PR 이 `maintainer.json` 에 정렬·추가한다. 이 문서는
그 파일을 여기서 고치지 않는다고 못 박는다. 행렬의 역할 여섯 칸은
그 pack 을 자동으로 끌어오지 않는다.

## 5. 불변식

시험이 red 로 막는 것.

1. `gym/profiles/` 아래 JSON 이 일곱 개다. 다른 확장자 파일이 없다.
2. 파일명 stem = `id` = `NAMED_PROFILE_IDS` 원소.
3. `kind` 는 `gymProfile`, `schemaVersion` 은 `1.0`.
4. `title` · `description` 은 비어 있지 않다.
5. `packs` 는 비어 있지 않은 문자열 배열이고 중복이 없다.
6. 모든 pack 참조는 `gym/packs/<id>/pack.json` 을 가진다.
7. `schema.validate_profile` 이 일곱 파일 모두에 빈 오류 목록을 준다.
8. 역할 여섯 자리의 `packs` 는 위 3절·4절 표와 같다.
9. `family` ∩ `boss` = ∅.
10. `editor` ∩ `publisher` = ∅.
11. `operator` ∩ `publisher` = ∅.
12. `operator` ∩ `family` = ∅.
13. 역할 다섯 + family 는 `expert-challenges` 를 품지 않는다.
14. 어느 두 자리도 같은 `packs` 배열을 갖지 않는다.
15. maintainer 의 packs 는 정렬되어 있고, 현재 브랜치 pack 전부와
    같다. 다른 여섯 자리의 합은 그 진부분집합이다.
16. 프로파일 JSON 에 `weights` / `score` / `max` / `tasks` /
    `exclude` 가 없다.
17. 파일은 UTF-8, BOM 없음, LF, 끝 개행, indent 2.
18. `id` 는 `^[a-z][a-z0-9-]*$`.
19. `gym/score.py` 는 `--profile` 을 받고
    `runner.score_all(..., profile_id=)` 로 넘긴다.
20. `gym/docs/profiles.md` 와 `mydocs/working/gym_profiles.md` 가
    일곱 id 와 역할 pack 이름을 품는다.

## 6. 사용

```bash
# 전 pack — 프로파일 없음
python gym/score.py --agent <이름>

# 자리 하나
python gym/score.py --agent <이름> --profile editor

# pack 을 직접 고른다. 프로파일과 같이 주면 프로파일이 이긴다
# (score_all 이 profile_id 가 있을 때 pack_ids 를 덮어쓴다).
python gym/score.py --agent <이름> --pack security

# 제출 루트·바이너리·산출 폴더
python gym/score.py --agent <이름> --profile publisher \
    --submissions gym/submissions/<이름> \
    --bin target/debug/rhwp \
    --out gym/out/<이름>-publisher
```

종료 코드는 채점기의 계약이다. 0=만점, 3=그 외. 프로파일이 종료
코드를 바꾸지 않는다. unavailable pack 은 0점이 아니다.

없는 프로파일 id 는 지금 `load_profile` 이 `FileNotFoundError` 를
올린다. 이 기둥은 그 칸을 새 kind 로 바꾸지 않는다. 예외 경로
고도화는 score/runner 기둥(#5260)의 자리다. 파일을 여기서 고치면
그 PR 과 싸운다.

## 7. 자리를 고르는 법

| 질문 | 자리 |
|---|---|
| 부모님과 5분 동안 읽고 셀 수 있나? | `family` |
| 도구가 무엇을 할 수 있는지 스스로 말하나? | `starter` |
| 본문·표·개체를 좌표로 고치고 재검증하나? | `editor` |
| 저장·조판·보안을 배포 전에 확인하나? | `publisher` |
| 폴더를 훑고 사다리로 이상을 좁히나? | `operator` |
| 한 단만 틀려도 막히는 완주를 버티나? | `boss` |
| 지금 운동장에 있는 pack 을 하나도 빼지 않나? | `maintainer` |

두 자리가 필요하면 두 번 돈다. 한 프로파일에 두 축을 합치지 않는다.
합치면 표가 다시 섞인다.

에이전트  inviter 는 `family` 를 먼저 권한다. 담력이 붙으면
`starter` → `editor` 또는 `publisher` 또는 `operator` → `boss`.
`maintainer` 는 매일의 기본값이 아니다. 전 표면은 시간이 많고
회귀를 볼 때 탄다.

## 8. 새 프로파일을 넣는 법

새 CLI 플래그를 만들지 않는다. 파일 하나가 자리 하나다.

1. 역할이 기존 일곱과 다른지 먼저 묻는다. 편집의 부분집합이면
   `--pack` 으로 충분하다.
2. `gym/profiles/<id>.json` 을 만든다. `id` = 파일명.
   `kind` = `gymProfile`. `schemaVersion` = `1.0`.
3. `packs` 는 존재하는 pack 만. 중복 없이. 역할 자리면 이야기
   순서, 전 표면이면 알파벳.
4. 이 문서 3절에 자리를 적고 4절 행렬에 열을 더한다.
5. `scripts/tests/test_gym_profiles.py` 의 `NAMED_PROFILE_IDS` ·
   `NAMED_PACKS` · 제목/설명 토큰을 같이 고친다.
6. 작업 기록에 "왜 기존 자리로 부족했나"를 남긴다.
7. `python -m unittest scripts.tests.test_gym_profiles` 가 통과하는지
   본다.
8. `python gym/tools/audit.py` 는 pack 정합을 본다. 프로파일 파일은
   audit 의 주 대상이 아니지만, pack 참조가 깨지면 이 시험이 먼저
   red 다.

여덟 번째 자리를 넣으면서 시험을 안 고치면
`test_no_unexpected_profile_files` 가 red 다. 그것이 의도다.

## 9. 새 pack 을 자리에 넣는 법

1. pack 자체가 `audit.py` 를 통과한다 (기준풀이 짝, 전역 id).
2. 역할 여섯 자리 중 어디에 들어가는지 이 문서 3절로 고른다.
   어디에도 안 들어가면 maintainer 만 따른다.
3. 역할 자리에 넣을 때는 `NAMED_PACKS` 와 3절·4절을 같이 고친다.
4. maintainer 는 현재 브랜치의 전 pack 과 같아야 한다. **이 기둥이
   그 파일을 고치지 않는다.** pack 을 더하는 PR 이 정렬된 한 줄을
   추가한다.
5. 역할 자리에 넣은 pack 은 maintainer 에도 있어야 한다. 시험
   `test_maintainer_covers_every_other_profile_pack` 이 지킨다.

다른 열린 PR 이 `form-journeys` 를 넣고 maintainer 에 한 줄을 더하면
그 줄은 그 PR 의 것이다. 여기서 같은 줄을 더하면 병합이 싸운다.
그래서 이 기둥은 시험을 현재 스냅샷에 고정하고 파일은 그대로 둔다.

## 10. 실패 칸

프로파일 기둥이 직접 채점하지 않는 실패. 그래도 자리가 어떻게
보이는지는 여기 적는다.

| 증상 | 원인 | 이 기둥이 보는가 |
|---|---|---|
| `FileNotFoundError: .../ghost.json` | `--profile ghost` | load_profile 계약. 엔진은 안 고친다. |
| `없는 pack 참조` | JSON 이 사라진 pack 을 가리킴 | 시험이 red. |
| pack `unavailable` | 바이너리에 요구 명령 없음 | 채점기 계약. 프로파일은 완화하지 않는다. |
| 점수 0 | 과제를 틀림 | 채점기. 프로파일 JSON 에 score 가 없다. |
| 입장 deny | 채점된 pack 0 | 입장 봉투. 프로파일 id 와 무관. |
| audit red | 기준풀이 짝·id 충돌 | pack 정합. 프로파일 파일이 직접 원인은 아니다. |
| 문서만 고침 | JSON 과 표가 어긋남 | 문서 토큰 시험이 red. |
| JSON 만 고침 | 문서가 옛 묶음을 말함 | 같은 시험이 red. |
| 여덟 번째 파일 | 카탈로그 미갱신 | inventory 시험이 red. |
| BOM / CRLF | 편집기가 파일을 바꿈 | hygiene 시험이 red. |

`validate_profile` 이 지금 거절하는 것:

- `kind` 가 `gymProfile` 이 아님.
- `packs` 가 없거나 비었음.
- `packs` 원소가 `pack_ids` 집합에 없음.

지금 거절하지 않는 것 (이 기둥이 schema.py 를 고치지 않으므로
시험도 강요하지 않음):

- `schemaVersion` 누락 — 파일 시험은 일곱 파일이 `1.0` 인지만 본다.
- `id` 와 파일명 불일치 — 파일 시험이 본다. schema 함수는 파일명을
  모른다.
- 중복 pack — 파일 시험이 본다.
- 알 수 없는 최상위 키 — 파일 시험이 본다.

스키마 함수를 여기서 키우면 #5279 와 싸운다. 그래서 파일 시험이
그 칸을 대신 지킨다.

## 11. 다른 기둥과 나눈 일

| 기둥 | 보는 것 | 이 문서가 보지 않는 것 |
|---|---|---|
| `schema.py` | kind/packs/없는 pack | 파일명=id, 정렬, 일곱 자리 카탈로그 |
| `audit.py` | pack 정합·기준풀이·전역 id | 프로파일 파일 |
| `score.py` / `runner.py` | 채점·unavailable·입장 | 어느 자리가 어느 pack 을 고르는가 |
| `certify.py` / `report.py` | 인증·리포트 산출 | 프로파일 카탈로그 |
| `tutorial/` · `PARK.md` | 사람용 산책 | 기계 계약. 이 기둥이 그 파일을 안 고친다. |
| pack 확장 PR | 과제·기준풀이 | 역할 여섯 자리의 고정 묶음 |

tutorial 기둥(#5280)이 `gym/tutorial/06-profiles.md` 를 넣을 수
있다. 그 파일은 입문 산책이다. 이 문서는 계약이다. 둘을 한 PR 에서
고치면 싸운다. 그래서 이 기둥은 `gym/docs/profiles.md` 만 정본으로
삼는다.

## 12. 파일 위생

- UTF-8, BOM 없음.
- LF (`\n`). CRLF 금지.
- 끝 줄 개행 하나.
- JSON indent 2, `ensure_ascii=False`.
- 키 순서: `schemaVersion`, `kind`, `id`, `title`, `description`,
  `packs`.
- `id` 는 소문자·숫자·하이픈.
- maintainer 의 packs 만 알파벳 정렬.
- 역할 여섯 자리는 3절에 적은 순서.

편집기가 저장할 때 키를 알파벳으로 바꾸면
`test_json_indent_is_two_spaces` 가 red 다. 그것이 의도다. 키 순서가
문서의 이야기와 같아야 사람이 파일을 읽을 수 있다.

## 13. 예제 — 자리가 고르는 것

같은 제출 폴더를 세 번 채점한다고 가정한다.

```text
gym/submissions/kim/
  casual-rides/CR01/answer.json
  core-cli/T01/answer.json
  text-editing/TE02/answer.json
  security/SE01/answer.json
  expert-challenges/XC01/...
```

`--profile family` 는 `casual-rides` 만 본다. `T01` 이 있어도
점수에 안 들어간다.

`--profile editor` 는 `core-cli` · `text-editing` · `table-editing` ·
`objects-media` 를 본다. `SE01` 은 안 본다.

`--profile publisher` 는 `serialization` · `layout-rendering` ·
`security` 를 본다. `TE02` 는 안 본다.

`--profile boss` 는 `expert-challenges` 만 본다.

`--profile maintainer` 는 현재 브랜치의 pack 전부를 본다.

점수는 매번 pack 별로 남는다. family 의 4점과 editor 의 본문 점수를
더해 "kim 의 종합"을 만들지 않는다. 종합이 필요하면 사람이 pack
표를 읽거나, 전 표면을 한 번 더 돈다.

## 14. 예제 — 잘못된 자리 선언

없는 pack:

```json
"packs": ["core-cli", "form-journeys"]
```

이 브랜치에 `form-journeys` 가 없으면 `validate_profile` 이
`없는 pack 참조` 를 남기고 파일 시험이 red 다. 그 pack 을 넣는 PR
이 먼저 합쳐지면, 그 PR 이 maintainer 에 한 줄을 더한다. 역할
자리에 넣을지는 그 PR 의 문서가 정한다. 이 기둥은 자동으로
`editor` 에 넣지 않는다.

빈 packs:

```json
"packs": []
```

선택은 최소 한 pack 이다. 빈 자리는 자리가 아니다.

가중치:

```json
"packs": ["core-cli"],
"weights": { "core-cli": 2.0 }
```

허용 키가 아니다. 파일 시험이 red 다. 점수를 뭉치고 싶으면
프로파일이 아니라 다른 기둥을 연다.

## 15. FAQ

**Q. README 표에 operator 가 없다.**
A. 파일은 있다. 이 문서가 정본이다. README 는 다른 PR 이 만질 수
있어 여기서 고치지 않는다.

**Q. README 가 maintainer 를 "전 12 pack" 이라고 한다.**
A. 예전의 수다. 현재 브랜치의 pack 수가 곧 전 표면이다. 숫자는
이 문서가 박제하지 않는다. 시험은 `pack_ids()` 와 비교한다.

**Q. 왜 schema.py 를 키우지 않나?**
A. #5279 가 그 파일의 예외·문서·시험을 고도화한다. 여기서 같은
함수를 키우면 싸운다. 파일 시험이 id=파일명·중복·키 집합을 대신
지킨다.

**Q. 왜 audit.py 가 프로파일을 안 보나?**
A. audit 는 pack 정합이다. 프로파일은 선택기다. 없는 pack 참조는
이 시험이 본다. audit 를 여기서 고치면 #5277 과 싸운다.

**Q. tutorial 의 프로파일 장을 같이 고치나?**
A. 아니다. #5280 의 파일이다. 이 문서는 `gym/docs/profiles.md` 다.

**Q. `--profile` 과 `--pack` 을 같이 주면?**
A. 지금 엔진은 profile_id 가 있으면 pack_ids 를 덮어쓴다. 프로파일이
이긴다. 이 기둥이 그 우선순위를 바꾸지 않는다.

**Q. 프로파일 JSON 에 과제 목록을 넣고 싶다.**
A. 넣지 않는다. 과제는 pack 의 것이다. 프로파일이 과제를 고르면
pack 의 완결성이 깨진다.

**Q. family 에 core-cli 를 넣고 싶다.**
A. 넣지 않는다. 가족 코스는 읽고 세기다. 코어 CLI 는 starter 다.

**Q. editor 에 security 를 넣고 싶다.**
A. 넣지 않는다. 배포 전 스윕은 publisher 다. 둘 다 필요하면 두 번
돈다.

**Q. maintainer 가 새 pack 을 빼먹었다.**
A. 이 브랜치에서 시험이 red 다. 고치는 쪽은 pack 을 더한 PR 이다.
이 기둥은 그 파일을 안 건드린다.

**Q. 여덟 번째 프로파일 이름은?**
A. 지금은 없다. 생기면 8절을 따른다.

**Q. 점수가 프로파일마다 다른 만점을 가진다.**
A. 맞다. 고른 pack 의 티어 합이 만점이다. 프로파일 JSON 이 만점을
선언하지 않는다.

**Q. Windows 에서 프로파일 파일이 CRLF 로 바뀐다.**
A. 시험이 red 다. `.gitattributes` 나 편집기 설정을 LF 로 둔다.
이 기둥이 gitattributes 를 바꾸지 않는다.

**Q. 프로파일을 지우면?**
A. 일곱 자리가 깨지므로 시험이 red 다. 자리를 없애는 것은 이
문서와 시험을 같이 내리는 별도 결정이다.

## 16. 명령 치트시트

```bash
# 계약 시험
python -m unittest scripts.tests.test_gym_profiles

# pack 정합 (프로파일 기둥이 고치지 않는 도구)
python gym/tools/audit.py

# 자리별 채점
python gym/score.py --agent demo --profile family
python gym/score.py --agent demo --profile starter
python gym/score.py --agent demo --profile editor
python gym/score.py --agent demo --profile publisher
python gym/score.py --agent demo --profile operator
python gym/score.py --agent demo --profile boss
python gym/score.py --agent demo --profile maintainer

# 파일 목록
ls gym/profiles
```

시험은 바이너리 없이 돈다. 채점은 바이너리가 필요하다. 이 기둥의
DoD 는 시험과 audit 이지 전 pack 만점이 아니다.

## 17. 문서 역할

| 파일 | 역할 |
|---|---|
| `gym/docs/profiles.md` | 정본. 자리·행렬·불변식·FAQ. |
| `mydocs/working/gym_profiles.md` | 작업 기록. 왜 파일을 안 고쳤나. 시험 지도. |
| `gym/profiles/*.json` | 기계가 읽는 자리 선언. |
| `scripts/tests/test_gym_profiles.py` | 위 세 층이 같은 말을 하는지. |
| `gym/README.md` | 운동장 입구. 이 기둥이 고치지 않는다. |
| `gym/tutorial/**` · `gym/PARK.md` | 산책. 이 기둥이 고치지 않는다. |

정본과 작업 기록은 서로를 가리킨다. 정본은 계약을 말하고, 작업
기록은 이슈 #5281 의 결정과 검증을 말한다. 계약을 바꾸면 정본을
먼저 고치고 시험을 고친다. 결정만 바꾸면 작업 기록만 고친다.

## 18. 역할 자리의 이야기 순서

사람이 JSON 을 위에서 아래로 읽는다. 순서는 알파벳이 아니라
이야기다.

`starter`: 도구를 읽고(`self-description` 이 아니라 코어를 먼저),
그다음 자기서술. 파일이 `core-cli`, `self-description` 인 이유다.

`editor`: 바닥(`core-cli`) → 본문 → 표 → 개체. 문서를 고치는 손이
실제로 거치는 순서다.

`publisher`: 형식(`serialization`) → 눈에 보이는 조판 → 배포 전
보안. 내보내기의 순서다.

`operator`: 이상을 찾고(`corpus-diagnostics`) 사다리로 닫는다
(`automation`).

이 순서를 알파벳으로 바꾸지 마라. maintainer 만 정렬한다. 전 표면은
이야기가 아니라 목록이다.

## 19. 자리와 티어

프로파일은 티어를 고르지 않는다. `family` 가 고르는
`casual-rides` 는 티어 1 이 많고, `boss` 가 고르는
`expert-challenges` 는 티어 4~5 다. 그것은 pack 의 성질이지
프로파일 필드의 성질이 아니다.

`tierMin` / `tierMax` 를 프로파일에 넣지 않는다. 티어로 자르고
싶으면 pack 을 가르거나 과제를 옮긴다. 프로파일은 pack 묶음만
고른다.

## 20. 자리와 unavailable

오래된 바이너리로 `publisher` 를 돌리면 `security` 의 어떤 명령이
없을 수 있다. 그때 그 pack 은 unavailable 이지 0점이 아니다.
프로파일 JSON 이 요구 명령을 낮추지 않는다. 낮추면 "이 자리에서는
보안을 안 봐도 된다"가 되어 배포자 자리가 거짓이다.

전 pack 이 unavailable 이면 입장 봉투는 deny 다. 프로파일 id 가
`maintainer` 여도 같다. 입장은 채점 완주이지 자리 이름이 아니다.

## 21. 자리와 리더보드

리더보드는 프로파일 id 를 등급으로 쓰지 않는다. 같은 에이전트가
`family` 만점과 `boss` 0점을 같이 가질 수 있다. 두 카드는 다른
선택이다. 한 카드의 총점으로 다른 카드를 덮지 않는다.

클레임에 프로파일을 남기고 싶으면 스코어카드의 `profile` 필드를
본다. `score_all` 이 `profile_id` 를 카드에 넣는다. 이 기둥이 그
필드 이름을 바꾸지 않는다.

## 22. 자리와 기준풀이

기준풀이는 pack 의 `reference/` 다. 프로파일은 기준풀이를 고르지
않는다. `--profile editor` 로 채점해도 `text-editing/reference/` 가
쓰인다. 프로파일 폴더에 reference 를 두지 않는다.

`build_baseline.py` 는 `--pack` 을 받는다. 프로파일로 기준풀이를
일괄 생성하고 싶으면 프로파일 JSON 의 packs 를 읽어 pack 마다
호출한다. 이 기둥은 그 래퍼 CLI 를 만들지 않는다.

## 23. 자리와 certify / report

`certify.py` 는 packs/core/profiles/tools 트리를 인증 입력으로
본다. 프로파일 파일을 고쳐 인증 해시를 흔들고 싶지 않으면, 이
기둥처럼 JSON 을 그대로 두고 문서·시험만 더한다. 역할 묶음이
이미 맞기 때문이다.

`report.py` 는 스코어카드를 읽는다. 프로파일 id 는 카드에 이미
있다. 리포트 형식을 여기서 바꾸지 않는다.

## 24. 회귀를 막는 최소 명령

저장소 루트에서:

```bash
python -m unittest scripts.tests.test_gym_profiles
python gym/tools/audit.py
```

첫 명령이 이 기둥의 계약이다. 둘째는 pack 정합이 프로파일 참조와
같이 살아 있는지 본다. 둘 다 바이너리 없이 돈다.

Rust 코드는 이 기둥이 건드리지 않는다. `cargo fmt --all -- --check`
는 PR 게이트로 한 번 확인한다. 포맷할 대상이 없어도 명령은
성공해야 한다.

## 25.  copilot · 에이전트를 위한 짧은 규칙

1. 프로파일 JSON 을 고치기 전에 이 문서 3절을 읽는다.
2. maintainer.json 을 고치려면 pack 을 더하는 PR 인지 확인한다.
   이 기둥의 PR 이 아니다.
3. schema.py / audit.py / certify.py / report.py / tutorial / PARK
   를 이 이슈에서 열지 않는다.
4. 새 자리는 파일 + 이 문서 + 시험 카탈로그를 한 커밋에서 맞춘다.
5. 점수를 뭉치는 키를 JSON 에 넣지 않는다.
6. 문서와 JSON 중 하나만 고치지 않는다.

이 여섯 줄만 지켜도 #5281 의 계약은 유지된다.

## 26. 용어

| 말 | 뜻 |
|---|---|
| 자리 | 프로파일 파일 하나. family 같은 이름. |
| 묶음 | 그 자리가 고르는 pack id 배열. |
| 역할 자리 | family 를 포함한 여섯 자리. 전 표면이 아닌 것. |
| 전 표면 | maintainer. 현재 브랜치 pack 전부. |
| 선택기 | pack 목록만 고르고 점수를 다시 계산하지 않는 도구. |
| 정본 | 이 파일. 기계 카탈로그의 사람용 쌍. |

"코스" · "존" · "어트랙션" 은 놀이공원 은유다. 기계 필드 이름은
`profile` 과 `packs` 만 쓴다.

## 27. 변경 이력

| 날짜 | 내용 |
|---|---|
| 2026-08-18 | #5281. 일곱 자리 계약·행렬·시험을 정본으로 고정. JSON 은 그대로. |

이후 변경은 이 표에 한 줄을 더하고 시험을 맞춘다.

## 28. pack 이 어느 자리에 들어가는가

4절 행렬을 pack 쪽에서 다시 읽는다. 새 pack 을 넣을 때 "어느
열에 ● 를 찍을까"를 이 절이 답한다.

### 28.1 `casual-rides`

가족 코스만. 읽고 세기. starter 에 넣지 않는다. 도구의 결을
익히는 자리와 부모님의 자리는 다르다.

### 28.2 `core-cli`

starter 와 editor. 조사·추출·편집·검증의 바닥. publisher 에
넣지 않는다. 배포 직전 확인은 코어 CLI 전체가 아니라 변환·조판·
보안이다. operator 에도 넣지 않는다. 운영자는 진단과 사다리다.

### 28.3 `self-description`

starter 만. 도구가 스스로를 설명하는 계약. editor 에 넣지 않는
이유: 편집 능력과 자기서술 능력은 다른 축이다. 편집자가 명령을
이미 안다면 자기서술은 그 자리의 만점에 넣을 일이 아니다.

### 28.4 `text-editing` · `table-editing` · `objects-media`

editor 만. 본문·표·개체. publisher 에 넣지 않는다. 고친 결과를
내보내는 확인은 다른 자리다. family 에 넣지 않는다. 입문존은
고치지 않는다.

### 28.5 `serialization` · `layout-rendering` · `security`

publisher 만. 형식 왕복, 조판 판정, 배포 전 스윕. editor 에
섞지 않는다. 한 번의 `--profile` 이 두 축을 열면 표가 다시
섞인다.

### 28.6 `corpus-diagnostics` · `automation`

operator 만. 스윕과 사다리. family · publisher · boss 와 겹치지
않는다. batch-ops 를 여기에 넣지 않는다. 대량 처리는 전 표면의
다음 층이다.

### 28.7 `expert-challenges`

boss 만. 역할 다섯 + family 는 이 pack 을 품지 않는다.

### 28.8 전 표면에서만 고르는 pack

`extraction`, `batch-ops`, `render-tree`, `studio-e2e`,
`table-csv`. 역할 여섯 자리의 최소 코스가 아니다. 필요하면
`--pack` 으로 더하거나 maintainer 를 탄다.

다른 PR 이 새 pack 을 더하면 기본값은 이 칸이다. 역할 자리에
넣으려면 정본 3절·4절과 `NAMED_PACKS` 를 같이 고친다.

## 29. 자리별 예상 질문

에이전트가 `--profile` 을 고를 때 자주 묻는 말.

"한글 문서를 열 수 있냐?" → family. 쪽수를 세고 단어를 찾는다.

"명령이 뭐가 있냐?" → starter. capabilities 와 코어 조회.

"이 표의 셀을 고칠 수 있냐?" → editor.

"이 파일을 저장해도 글자가 안 바뀌냐?" → publisher 의
serialization.

"배포 전에 숨은 글·주입을 보냐?" → publisher 의 security.

"폴더 200개를 훑고 이상한 쪽만 남기냐?" → operator.

"사다리 한 단을 틀리면 막히냐?" → boss.

"지금 운동장에 있는 것을 빼먹지 않았냐?" → maintainer.

질문이 두 개면 자리를 두 번 탄다.

## 30. 리뷰 체크 — 정본 쪽

정본을 고치는 PR 이 나중에 이 파일을 열면 아래를 같이 맞춘다.

1. 3절의 packs 배열과 실제 JSON 이 같다.
2. 4절 행렬의 ● 가 3절과 같다.
3. 5절 불변식 번호가 시험 클래스와 어긋나지 않는다. 번호를
   끼워 넣으면 시험 문서 토큰도 본다.
4. 8절·9절의 절차가 "새 CLI 없음"과 모순되지 않는다.
5. 11절 표의 이웃 기둥이 아직 그 파일을 소유하는지 확인한다.

이 다섯은 작업 기록 20절의 리뷰 경로와 짝이다.

## 31. 자리 가 가리키는 pack 의 축

프로파일은 pack id 만 적는다. 사람이 그 id 가 무슨 축인지 잊으면
자리를 잘못 고른다. 아래는 현재 브랜치 pack.json 의 title · axis
를 자리 옆에 붙인 표다. pack 의 과제 수·만점은 여기 박제하지
않는다. pack 확장 PR 이 그 숫자를 바꾼다.

| pack | title | axis | 자리 |
|---|---|---|---|
| `casual-rides` | 입문 놀이기구 (누구나) | 입문 (읽고 세기) | family |
| `core-cli` | 코어 CLI | 조사·추출·편집·검증 (운동장 최소 코어) | starter, editor |
| `self-description` | 자기서술 표면 | 자기서술 (도구가 스스로를 설명하는 계약) | starter |
| `text-editing` | 본문 편집 | 편집 (탐색→치환→재검증) | editor |
| `table-editing` | 표 편집 | 편집 (표 좌표 지정) | editor |
| `objects-media` | 개체·미디어 | 발견 (필드·개체·렌더 산출물) | editor |
| `serialization` | 저장·변환 | 변환 (형식 왕복·IR 대조) | publisher |
| `layout-rendering` | 조판·렌더링 | 검증 (조판 판정·렌더 산출) | publisher |
| `security` | 보안 스윕 | 보안 (은닉·주입·유니코드·PII) | publisher |
| `corpus-diagnostics` | 코퍼스·진단 | 진단 (폴더 스윕·쪽 덤프·비교 판정) | operator |
| `automation` | 자동화·검증 사다리 | 자동화 (계획·캡슐·서명·앵커·정산·감사) | operator |
| `expert-challenges` | 보스 어트랙션 (고난도 완주) | 자동화 (사다리 완주) | boss |
| `extraction` | 데이터 추출 (읽기) | 조회 (문서에서 데이터를 뽑아내는 능력) | 전 표면 |
| `batch-ops` | 다문서 대량 처리 (batch) | 자동화 (다문서 대량 처리) | 전 표면 |
| `render-tree` | 렌더 트리 구조 추출 | 조회 (렌더 트리 구조 추출) | 전 표면 |
| `studio-e2e` | 스튜디오 e2e 문서 계약 | 편집 (studio e2e 에서 파생한 CLI 검증 가능 문서 계약) | 전 표면 |
| `table-csv` | 표 CSV 왕복 (되쓰기) | 편집 (표를 CSV로 뽑아 고쳐 되넣기) | 전 표면 |

title · axis 문자열이 pack.json 에서 바뀌면 이 표를 고친다. 시험은
id 소속만 잠그고 title 문자열은 잠그지 않는다. pack 기둥이 제목을
다듬을 권리를 남겨 둔다.

## 32. 스코어카드를 자리로 읽는 법

`scorecard.json` 의 `profile` 필드는 고른 자리 id 또는 `null`
(전 pack)이다. `packs[]` 는 그 선택이 고른 것만 담는다.

family 카드에서 `expert-challenges` 행을 찾지 마라. 그 자리는 그
pack 을 고르지 않았다. 없는 행을 "0점"으로 읽으면 거짓이다.
고르지 않은 pack 은 점수가 아니라 부재다.

publisher 카드의 `security` 가 unavailable 이면 배포자 자리의 한
축이 그 바이너리에 없다. 나머지 두 pack 만점으로 "publisher
만점"이라고 부르지 마라. 만점은 고른 pack 전부가 scored 이고
과제를 맞춘 때다.

maintainer 카드의 총점은 편의값이다. 어느 축이 모자란지는 pack
행을 본다. 프로파일이 총점을 다시 가중하지 않으므로, 카드의
`total.score` 는 채점된 pack 티어 합일 뿐이다.

## 33. 자리를 잘못 고른 예

편집 과제를 family 로 채점한다. 카드에 text-editing 이 없다.
실패가 아니라 선택 실수다. `--profile editor` 로 다시 돈다.

보안 스윕을 editor 로 채점한다. 같은 실수다. publisher 다.

보스 과제를 starter 로 채점한다. starter 는 core-cli 와
self-description 만 본다. XC 과제는 카드에 없다.

전 표면을 원하면서 editor 를 두 번 돌린다. editor 는 전 표면이
아니다. maintainer 를 탄다.

한 카드에 두 자리를 합치려고 JSON 에 packs 를 이어 붙인다.
그러면 그 파일은 여덟 번째 자리가 되거나 기존 자리의 계약을
깨뜨린다. 두 번 돌아라.

## 34. 문제 해결

**시험이 `없는 pack 참조` 로 red.**
JSON 이 이 브랜치에 없는 pack id 를 가리킨다. id 오타를 고치거나,
그 pack 을 넣는 PR 이 합쳐지기를 기다린다. 이 기둥에서 pack 을
만들지 않는다.

**시험이 `새 프로파일 파일` 로 red.**
여덟 번째 JSON 을 넣었다. `NAMED_PROFILE_IDS` 와 정본 3절을 같이
고치지 않으면 통과하지 않는다.

**시험이 `indent/키 순서 불일치` 로 red.**
편집기가 키를 알파벳으로 바꿨거나 indent 가 4 다. 키 순서는
`schemaVersion, kind, id, title, description, packs` 다.

**시험이 `CRLF` 로 red.**
LF 로 다시 저장한다.

**시험이 `maintainer 가 전 pack 과 다르다` 로 red.**
이 브랜치에 pack 이 더해졌거나 빠졌다. pack 을 더한 쪽이
maintainer.json 을 정렬·추가한다. 이 기둥은 그 파일을 안 고친다.

**audit.py 가 red.**
프로파일 기둥이 pack 을 안 만졌는데 audit 가 red 면, 작업 트리에
다른 pack 변경이 섞인 것이다. 이 기둥의 staged 경로를 확인한다.

**`load_profile` 이 FileNotFoundError.**
`--profile` 철자를 본다. 일곱 이름만 있다. 대문자를 쓰지 않는다.

**채점이 전 pack 을 돈다.**
`--profile` 을 빼먹었다. 플래그 없이 돌면 선택기가 동작하지
않는다.

## 35. 자리와 CI

이 기둥은 CI workflow 를 고치지 않는다. 저장소의 기존 Python
unittest 수집이 `scripts/tests/test_gym_*.py` 를 가져간다. 새
파일을 넣는 것만으로 CI 가 계약을 돈다.

audit.py 는 별도 job 이거나 다른 시험이 호출한다. 이 기둥의
`test_audit_real_repo_still_ok` 가 같은 도구를 한 번 더 호출해
pack 정합이 살아 있는지 본다.

## 36. 자리와 한글

title · description 은 한글이다. JSON 은 UTF-8, `ensure_ascii` 가
아니다. `\uXXXX` 로 이스케이프하면 indent 시험이 red 다. 사람이
파일을 읽을 수 있어야 한다.

id 는 영문 소문자다. `--profile 편집자` 는 파일이 없다.

## 37. 자리와 공백

id 와 pack id 에 공백을 넣지 않는다. `core cli` 는 없는 pack 이다.
하이픈만 쓴다. `core-cli`.

description 앞뒤 공백만 있는 문자열은 빈 설명이다. 파일 시험이
`strip()` 후 비었는지 본다.

## 38. 닫는 말

프로파일은 이름 하나로 pack 을 고르는 작은 파일이다. 작아서
깨지기 쉽다. 표와 JSON 이 하루만 어긋나도 입문 에이전트가 보스
과제를 받거나, 배포자가 편집 점수만 보고 나간다. 이 문서와
`scripts/tests/test_gym_profiles.py` 가 그 하루를 막는다.

엔진을 키우지 않아도 계약은 잠글 수 있다. 그것이 #5281 의
닫는 방식이다.

## 39. `--pack` 과 `--profile` 을 같이 줄 때

지금 엔진은 `profile_id` 가 있으면 `pack_ids` 인자를 버린다.

```text
score_all(..., pack_ids=["security"], profile_id="editor")
→ editor 의 packs 를 채점한다. security 는 사라진다.
```

의도적으로 보안만 보고 싶은데 `--profile editor` 를 같이 붙이면
편집 자리만 돈다. 플래그를 하나만 써라. 이 기둥이 우선순위를
바꾸지 않는다. 바꾸는 순간 score/runner 기둥과 싸운다.

문서와 시험은 그 한 줄이 아직 원문에 있는지만 본다.

## 40. 자리별 최소 재현

바이너리가 있을 때 사람이 자리를 눈으로 확인하는 최소 명령.
이 기둥의 DoD 는 아니다. 재현 메모다.

```bash
python gym/score.py --agent probe --profile family --out /tmp/pf
python gym/score.py --agent probe --profile starter --out /tmp/ps
python gym/score.py --agent probe --profile editor --out /tmp/pe
python gym/score.py --agent probe --profile publisher --out /tmp/pp
python gym/score.py --agent probe --profile operator --out /tmp/po
python gym/score.py --agent probe --profile boss --out /tmp/pb
python gym/score.py --agent probe --profile maintainer --out /tmp/pm
```

각 `scorecard.json` 의 `profile` 필드와 `packs[].id` 가 3절과
같은지 본다. 제출이 비어 있으면 점수는 낮고, 선택 목록은
같아야 한다. 선택 목록이 다르면 JSON 과 엔진이 어긋난 것이다.

Windows 에서는 `/tmp` 대신 `%TEMP%\gym-profiles` 를 쓴다.

## 41. 파일이 일곱인 이유

네 자리(starter/editor/publisher/maintainer)만 있으면 입문존과
보스존과 운영자가 입구를 잃는다. 여섯 자리에서 family 를 빼면
초대가 코어 CLI 로 떨어진다. operator 를 빼면 진단·사다리가
전 표면에만 남는다. boss 를 빼면 고난도가 편집 점수에 섞일
유혹이 생긴다. 일곱은 최소 집합이다. 여덟은 아직 필요 없다.

## 42. 이 문서를 고치는 PR 에게

절을 더할 때는 번호를 이어라. 절을 지울 때는
`DocsSectionTests.REQUIRED_HEADINGS` 를 같이 본다. pack 소속을
바꾸면 `NAMED_PACKS` · `PACK_ROLE_OWNERS` · 4절 · 28절 · 31절을
한 커밋에서 맞춘다. 하나만 고치면 시험이 red 다. 그것이 이
문서가 존재하는 이유다.

## 43. 자리 이름 사전

외부 문서가 다른 말로 자리를 부를 때 이 표로 되돌린다.

| 다른 말 | 자리 id |
|---|---|
| 가족 코스, 입문존, 부모님 코스, kiddie | `family` |
| 입문, 초보, 최소 코스, onboarding | `starter` |
| 편집자, 편집 코스, scribe | `editor` |
| 배포자, 배포 전, release 전 확인 | `publisher` |
| 운영자, 진단, 사다리 운영 | `operator` |
| 보스, 보스존, 고난도, dragon | `boss` |
| 메인테이너, 전 표면, 완주, 전부 | `maintainer` |

다른 말을 JSON `id` 로 쓰지 않는다. `--profile 가족` 은 파일이
없다. `--profile family` 다.

## 44. 관련 이슈

| 이슈 | 관계 |
|---|---|
| #5281 | 이 문서. |
| #4653 | pack·profile 원 도입. |
| #5260 / #5278 | score/runner 예외. 이 기둥이 엔진을 안 연다. |
| #5277 | audit 고도화. 이 기둥이 audit.py 를 안 연다. |
| #5279 | schema 고도화. 이 기둥이 schema.py 를 안 연다. |
| #5275 | certify/report. 안 연다. |
| #5280 | tutorial/PARK. 안 연다. |
| #5214 | maintainer.json. 안 연다. |

관련 이슈를 닫지 않는다. `closes #5281` 만.

이 표에 없는 gym 이슈(pack 과제 확장, 커버리지, 세션 트레이스)는
프로파일 소속을 바꾸지 않는 한 이 문서의 대상이 아니다. 새 pack
이 합쳐지면 maintainer 시험이 그 스냅샷을 따라간다. 역할 여섯
칸에 ● 를 찍을지는 그때의 정본 PR 이 정한다.

## 45. 라이선스·소유

프로파일 JSON 과 이 문서는 저장소 라이선스를 따른다. 자리 이름은
놀이공원 은유일 뿐이고 상표 주장이 아니다. `family` · `boss` 는
기계 id 다.
