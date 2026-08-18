# proptest 왕복 계층 도구

M04-f (#5465). devel 의 기존 `rhwp run` step 4종만 카탈로그로 펼친다.

```bash
python tools/proptest_roundtrip/gen_m04f_catalogs.py
python tools/proptest_roundtrip/test_gen_m04f_catalogs.py
```

생성물은 `tests/fixtures/proptest_m04f/`. DocumentCore 편집 API 를 발명하지 않는다.
