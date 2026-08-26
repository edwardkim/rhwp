# M04-f proptest 왕복 픽스처

이슈 #5465. `tools/proptest_roundtrip/gen_m04f_catalogs.py` 가 다시 쓴다.
기존 run step 4종만. DocumentCore mutation 발명 금지.

- `catalogs/` 픽스처·스킵·유효/무효 계획·예외·변형·조건
- `cases/` action 별 변형
- `matrices/` fixture×step · skip 분포
- `reports/` 요약·정직 표·CI

카탈로그를 손으로 고치지 말고 생성기를 돌린다.
