# M04-f CI

왕복 property 는 퍼지가 아니다. 싼 debug + 이름 필터.

```bash
python tools/proptest_roundtrip/gen_m04f_catalogs.py
python -m unittest tools.proptest_roundtrip.test_gen_m04f_catalogs
node --test scripts/tests/run-prop-roundtrip.test.mjs
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-prop-roundtrip.mjs --cargo-test
```

러너 `scripts/run-prop-roundtrip.mjs` 가 집는 원본:

- 필수: `prop_roundtrip_ci`
- 본체: `prop_hwpx_roundtrip`, `prop_hwp5_roundtrip`
- 계획 생성기: `prop_edit_plan`
- 고도화: `prop_m04f_catalog`, `prop_m04f_skip`, `prop_m04f_plans`, `prop_m04f_exceptions`, `prop_m04f_mutations`

원본이 없으면 skip (필수 `prop_roundtrip_ci` 제외).
기본 8 cases. 전체 화력은 `PROPTEST_CASES`.
nextest archive 5번째 shard 를 넣지 않는다.

카탈로그 시험은 JSONL 을 읽어 스키마·스킵 정직만 본다.
문서 parse→serialize 전수는 M04-2/3 의 기존 8 cases 가 맡는다.
