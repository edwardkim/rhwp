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

## Stage W9-Q2

- 공개 fixture: [`kerning_pair_fixture.hwpx`](fixtures/kerning_pair_fixture.hwpx)
- OpenType 경계: [`kerning_capability_boundary.json`](kerning_capability_boundary.json)
- 현행 제품·WASM 기준선: [`kerning_q2_fixture_baseline.json`](kerning_q2_fixture_baseline.json)
- 한컴 판정: [`kerning_q2_hancom_adjudication.json`](kerning_q2_hancom_adjudication.json)
- 수행 보고서: [`task_m100_4968_w9_q2.md`](../../../working/task_m100_4968_w9_q2.md)
- local-only 원장: `output/4968/w9-q2/hyperv-readback/`, mode `0600`

공개 fixture는 `AV To WA HH 가나다`와 ratio 100/90/80, spacing 0/-5/-10, K0/K1을 교차한다. body
K0/K1은 같은 stored/fresh lane에 있어 flag 효과를 분리하며, table-cell·text-box는 보조 context다. Noto Sans
KR의 공개 GPOS 경계는 `AV=-18`, `To=-76`, `WA=0`, `HH=0` design unit다.

HWP 2020 `11.0.0.9136`은 HWPML2X readback에서 22개 fixture context의 kerning flag·장평·자간·face를 전부
보존했지만 PDF body 9축의 K0/K1 position과 pair gap은 전부 같았다. 그러므로 이 관측은 한컴 버전 한정 음성
호환성 증거이며 K1 구현값의 정답은 OpenType capability 계약에서 가져온다. 적용 순서는 differential이 없어
한컴 출력에서 관측할 수 없었다.

```bash
python3 -m unittest -v scripts.tests.test_kerning_q2
node --test scripts/tests/kerning_q2_fixture_baseline.test.mjs
python3 scripts/kerning_q2_hancom_adjudication.py \
  --evidence-root output/4968/w9-q2/hyperv-readback \
  --output-root mydocs/tech/investigations/issue-4968 \
  --output kerning_q2_hancom_adjudication.json
node scripts/kerning_q2_fixture_baseline.mjs
```

OpenType 경계, 한컴 판정, 통합 Q2 baseline의 canonical SHA-256은 각각
`4d628135329b82ed401453eef709783329840a5d1e22068afd83a7d2e7927576`,
`8fce761f58c742a9a6d4fcf1cbfdf7a3037acd8f14f028e5f2c5cd41f64ff1ee`,
`8ad6b8562bfb19fc9c751d1d74012c334d821028640a1459e27a0b0b6bbbbda1`이다.

## Stage W9-Q3

- 의도 전달 보고서: [`task_m100_4968_w9_q3_1.md`](../../../working/task_m100_4968_w9_q3_1.md)
- capability provider 보고서:
  [`task_m100_4968_w9_q3_2.md`](../../../working/task_m100_4968_w9_q3_2.md)
- bounded run gate 보고서:
  [`task_m100_4968_w9_q3_3.md`](../../../working/task_m100_4968_w9_q3_3.md)
- integration 원본:
  `tests/cases/issue_4968_kerning_intent_plumbing.rs`,
  `tests/cases/issue_4968_kerning_capability_provider.rs`

Q3-1은 `ResolvedCharStyle.kerning`을 `TextStyle`과 공개 layer-tree 관측 경계까지 전달했다. false는
직렬화에서 생략하고 true만 노출하므로 K0 schema·position은 유지된다. Q3-2는 선택 완료된 exact face
bytes만 받는 bounded capability provider를 추가했다. GPOS `kern` pair lookup, horizontal legacy kern
format 0, unsupported를 기능 탐지하고 GPOS를 우선한다. source 없음·malformed·32 MiB 초과는 구조화된
이유로 fail-closed한다. Q3-3은 request와 capability를 결합하되 code point·glyph 4,096, 인접 pair
4,095 상한 안에서만 pair engine 진입을 허용한다. `eligible`은 최종 적용 판정이 아니며 원문은 trace에
남기지 않는다. 세 절편 모두 실제 pair advance는 아직 바꾸지 않는다.
