# Task M100 #4964 — Stage W6-1 분리 전 행동 기준선

- **수행계획**: [`../plans/task_m100_4964.md`](../../plans/archives/task_m100_4964.md)
- **기준 source**: `upstream/devel@d1ad0eb8784dbc55f0796e2ba8775f7363247b91`
- **단계 목적**: 데이터 이동 전에 600개 composition·문자 폭·lookup 행동을 동결한다.
- **판정**: 통과

## 1. 입력

| 입력 | SHA-256 | 역할 |
| --- | --- | --- |
| `src/renderer/font_metrics_data.rs` | `7e38ab9a169407c7aecf9563be6ec4427f8f3deda990974e55e7adc4596fb72d` | 분리 전 metric·lookup source |
| `issue-4939/font_rule_baseline.json` | `a0fac05c3138471eb3e7404fc701f0053caa6c01a923afae60fd4da64064b466` | W1 600행·401명 기준선 |
| `tools/task2430/EVIDENCE.md` | `975e827841449182bc7154ba945835b5776e6f078db9bb1bae51f1359bd0dd5e` | 5개 measured overlay 환경·계보 |
| `tools/task2430/measured/preflight_report.tsv` | `d82042076d15cfcfd4520f1582d4a5c2586b46d7321b8158d6c12d8126f4abed` | 한컴 COM face readback |

private 10k corpus, local-only font root와 Hyper-V 한컴 환경은 입력으로 사용하지 않았다. W6-1은 기존
공개/추적 evidence를 재사용하며 조판을 재계측하지 않는다.

## 2. 기준선 결과

`font_metric_pre_split_baseline.json`이 기록한 분리 전 모집단은 다음과 같다.

| 항목 | 결과 |
| --- | ---: |
| metric entry | 600 |
| unique name | 401 |
| regular / bold / italic / bold-italic | 383 / 89 / 79 / 49 |
| historical generated region | index 0..594, 595개 |
| measured overlay region | index 595..599, 5개 |
| alias | 67 |
| lookup input name | 464 |
| style 조합 | 이름당 4개 |
| 검사한 entry-codepoint pair | 7,062,099 |

generated region은 source storage의 역사적 구분이다. 이 단계는 595개를 source-exact라고 선언하지
않으며, 원본 bytes·face index·name record가 없는 항목은 Stage W6-2에서 `unknown`으로 남긴다.

### 2.1 동결 hash

| projection | SHA-256 |
| --- | --- |
| composition | `d4cdac86b3c6ee55d8b1aa921d662f1fc1241c2809cb9c8ffe991d56a045e69a` |
| metric data | `025812eac4bad179c5b87e23b15fdf08a4e4fb3f19a6e453738e03110a140bcf` |
| exhaustive width | `2cd1389a14401f6488041af3c54ff0ba5e982d944acd0b5bb56147056e3a7d1b` |
| lookup | `bb3008f9dc379bd580119a6a658388732e94358db2039dbb02d78c28ec992fdf` |

폭 projection은 각 항목이 값을 저장하는 모든 Latin range codepoint와 모든 Hangul 음절
U+AC00..U+D7A3을 검사한다. Latin width 0과 Hangul table 부재는 runtime과 같이 `None`으로
직렬화한다. range 순서·경계와 Hangul 우선 분기는 metric data/composition hash가 함께 보호한다.

## 3. #2430 measured overlay 확인

다음 명령은 COM이나 원본 HFT를 다시 실행하지 않고 추적된 TSV에서 ASCII 95자를 재구성했다.

```bash
python3 tools/task2430/gen_metrics.py \
  --verify \
  --ladder-dir tools/task2430/measured
```

결과:

```text
한양신명조 → HanyangSinMyeongJo: 95/95 exact match — OK
한양중고딕 → HanyangJungGothic: 95/95 exact match — OK
한양견명조 → HanyangKyunMyeongJo: 95/95 exact match — OK
한양견고딕 → HanyangKyunGothic: 95/95 exact match — OK
휴먼명조 → HumanMyeongJo: 95/95 exact match — OK
```

## 4. negative contract

`scripts/tests/font_metric_lineage.test.mjs`는 다음 변조를 실제 source 문자열에 적용해 탐지 여부를
검사했다.

| 변조 | 기대 판정 | 결과 |
| --- | --- | --- |
| 선언 600을 599로 변경 | parse fail | 통과 |
| 첫 두 metric entry 순서 교환 | composition·lookup hash 변화 | 통과 |
| 첫 Latin width를 300에서 301로 변경 | metric·width hash 변화 | 통과 |
| 마지막 overlay identity 변경 | overlay population fail | 통과 |
| 같은 입력 두 번 분석 | canonical JSON 동일 | 통과 |

## 5. 검증

| 명령 | 결과 |
| --- | --- |
| `node scripts/font_metric_lineage.mjs --check` | 통과 |
| `node --test scripts/tests/font_metric_lineage.test.mjs scripts/tests/font_rule_ledger.test.mjs` | 16/16 통과 |
| `node scripts/font_rule_ledger.mjs boundary --sources mydocs/tech/investigations/issue-4939/font_rule_sources.json` | 통과 |
| `cargo test --profile release-test --lib font_metrics` | 9/9 통과, 0 실패 |
| `python3 tools/task2430/gen_metrics.py --verify --ladder-dir tools/task2430/measured` | 5 face 모두 95/95 일치 |

Rust test 실행 중 Cargo가 기존 `Cargo.lock`의 workspace package 두 항목 순서를 자동 정규화했지만,
W6와 무관한 파생 diff이므로 기준 checkout의 bytes로 복원해 변경에 포함하지 않았다.

## 6. Stage 판정과 다음 게이트

W1 모집단, #2430 overlay와 Rust legacy lookup이 같은 현재 상태를 가리킨다. 순서·폭·lookup을 바꾸는
변조도 모두 기준선에서 탐지되므로 Stage W6-2의 600행 lineage manifest inventory를 시작할 수 있다.

아직 metric data를 이동하거나 generator를 수정하지 않았다. Stage W6-2는 메인테이너 승인 후
시작하며, 600개 전부가 manifest row에 대응하지 않으면 Stage W6-3 물리 분리로 넘어가지 않는다.
