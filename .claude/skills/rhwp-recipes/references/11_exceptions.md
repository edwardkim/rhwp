# 예외 세 갈래

이슈: #5331. 라우터 장 `11_exceptions.md`.
정본 디렉터리: `mydocs/manual/recipes/`.
gym 이 아니고 새 CLI 도 없다. 07·08 을 만들지 않는다.

라우터가 레시피를 고르지 못하고 멈추는 경우는 셋뿐이다.

| id | 종류 | 정지 | 행동 |
| --- | --- | --- | --- |
| E01 | 레시피 파일 없음 | R03 | 중단. 발명 금지 |
| E02 | last_verified stale | R04 | 날짜를 보여주고 중단 |
| E03 | 두 장과 동시에 맞음 | R05 | 둘을 보여주고 고르게 함 |

상세: [21_missing_recipe.md](21_missing_recipe.md), [20_stale_last_verified.md](20_stale_last_verified.md), [22_two_recipe_match.md](22_two_recipe_match.md).
