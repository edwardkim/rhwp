# Issue #4968 W9 kerning 조사 증적

이 디렉터리는 kerning flag의 measurement·shaping end-to-end 연결을 위한 공개·비식별 증적만 보존한다.
10k corpus의 문서명·경로·본문·문서 hash와 개별 식별 목록은 포함하지 않는다.

## Stage W9-Q0

- 공개 정본: [`kerning_cohort_baseline.json`](kerning_cohort_baseline.json)
- projector: [`scripts/font_kerning_cohort.py`](../../../../scripts/font_kerning_cohort.py)
- 계약 테스트:
  [`scripts/tests/test_font_kerning_cohort.py`](../../../../scripts/tests/test_font_kerning_cohort.py)
- local-only 원장: `output/4968/w9-q0/kerning_private_cohort.json`, mode `0600`

projector는 W3의 기존 10,000-entry checkpoint journal만 streaming한다. corpus source file을 다시 열지 않고
문서별 kerning usage를 final aggregate와 전항 대사한다. 공개 JSON은 format, face, metric face, 장평·자간,
context와 stored/fresh lane의 aggregate만 담는다.

실행 결과는 157문서(HWP 115, HWPX 42), 175,466자다. W8 face와의 겹침은 rank 8
`KoPubWorld바탕체 Light` 1문서·42자뿐이고 rank 1·7은 0이다. #4967의 세 face가 모두 `no-change`로
종결됐으므로 W9 pair positioning 진입 게이트를 충족한다.

```bash
python3 -m unittest scripts.tests.test_font_kerning_cohort

python3 scripts/font_kerning_cohort.py \
  --manifest output/poc/font-metric-coverage/full-manifest-stage4-c-r2.json \
  --coverage output/poc/font-metric-coverage/final-stage4-c-10k-r3.json \
  --journal output/poc/font-metric-coverage/checkpoint-stage4-c-10k-r3/journal.ndjson \
  --w5-fixture mydocs/tech/investigations/issue-4963/fixtures/oracle_typesetting_fixture.manifest.json \
  --noto-font ttfs/opensource/NotoSansKR-Regular.ttf \
  --w8-rank1 mydocs/tech/investigations/issue-4967/rank1_metric_hypothesis.json \
  --w8-rank7 mydocs/tech/investigations/issue-4967/rank7_private_qualification.json \
  --w8-rank8 mydocs/tech/investigations/issue-4967/rank8_private_qualification.json \
  --private-output output/4968/w9-q0/kerning_private_cohort.json \
  --public-output mydocs/tech/investigations/issue-4968/kerning_cohort_baseline.json
```

공개 정본의 canonical SHA-256은
`95309e457f78de38f2b1470b05e6f0fe97f00684ffff5eddd4d82c6438fb71e6`이다.

## Stage W9-Q1

- 공개 정본: [`kerning_q1_baseline.json`](kerning_q1_baseline.json)
- 기준선 도구: [`scripts/kerning_q1_baseline.mjs`](../../../../scripts/kerning_q1_baseline.mjs)
- 계약 테스트:
  [`scripts/tests/kerning_q1_baseline.test.mjs`](../../../../scripts/tests/kerning_q1_baseline.test.mjs)
- 수행 보고서: [`task_m100_4968_w9_q1.md`](../../../working/task_m100_4968_w9_q1.md)

W5 공개 fixture의 kerning-off body matrix 9개 run에서 native와 Docker WASM positions가 전항 일치했고,
SVG는 400,536 bytes로 byte-exact 일치했다. layer-tree 원문에는 synthetic paragraph sentinel의 target
pointer-width 표현 차이가 20건 존재한다. 양쪽 raw hash를 보존하고 그 sentinel만 `para:MAX`로 정규화했을
때 전체 tree가 일치해야 통과하도록 했다.

Q1은 제품 조판을 바꾸지 않는다. request·capability·disposition·source provenance·fallback reason의 최소
계약과 32 MiB/4,096 상한을 동결하고, `rustybuzz`를 Q2·Q3 검증 조건이 붙은 공통 엔진 후보로 선택했다.

```bash
docker compose --env-file .env.docker run --rm wasm
cargo build --release --bin rhwp --bin rhwp-q-kit
node --test scripts/tests/kerning_q1_baseline.test.mjs
node scripts/kerning_q1_baseline.mjs
```

공개 정본의 canonical SHA-256은
`74f0310aa61cb4464b7176a4f53b4154aab3b828bbf5608ed7ebc9644689d499`다.
