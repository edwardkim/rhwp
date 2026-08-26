# M04-f proptest 왕복 변형·예외 고도화

이슈 #5465. devel 의 기존 `rhwp run` step 4종만 카탈로그·스킵 정직 표·예외로 펼친다.

- 생성기: `tools/proptest_roundtrip/gen_m04f_catalogs.py`
- 픽스처: `tests/fixtures/proptest_m04f/`
- 계약: `tests/cases/prop_m04f_*.rs`

DocumentCore 편집 API · canvaskit_policy · pdf · page-count serializer ·
layout-anomaly · oracle_public · render_backend · gym 은 만지지 않는다.
