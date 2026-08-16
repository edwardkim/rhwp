---
kind: investigation
status: active
canonical: mydocs/tech/investigations/issue-4939/README.md
last_verified: 2026-08-16
---

# Issue #4939 W0 폰트 규칙 기준선 manifest

## 1. 결론

현재 font metric table, lookup 순서, source boundary와 공개 native/WASM fixture를
`795e7b5fac24cfef79017e9120516570851a03b2` 입력 commit에 고정했다. 같은 commit에서 candidate와
baseline을 각각 두 번 생성했으며 두 실행의 SHA-256이 일치했다.

이 manifest는 현재 동작의 관찰 기준선이다. alias·fallback의 의미가 옳다는 승인이나
`font_rule_baseline.json`의 runtime canonical registry 승격을 뜻하지 않는다.

## 2. 입력 폐합

| 항목 | 값 |
| --- | --- |
| source commit | `795e7b5fac24cfef79017e9120516570851a03b2` |
| generator version | `2.0.0` |
| hashed input | 21개 repository-relative 파일 |
| source owner / selector | 12 / 30 |
| candidate SHA-256 | `7a505a228fc6ded6fcc88679d2f3f3340cb40cef7d7fab4e914cdb07827394a4` |
| baseline SHA-256 | `a0fac05c3138471eb3e7404fc701f0053caa6c01a923afae60fd4da64064b466` |

각 candidate는 owner, symbol, literal selector, extraction mode, 현재 match count와 source file
SHA-256을 가진다. baseline 생성기는 candidate 수집 뒤 source digest가 달라지면 중단한다.
machine JSON에는 생성 시각, elapsed time, 사용자명과 절대 workspace path를 넣지 않았다.

Stage 2 candidate 30개는 mapping rule 30개라는 뜻이 아니다. source boundary selector 30개라는
뜻이며, 하나의 selector가 여러 finite mapping을 포함할 수 있다. 실제 행 확장과 전수성 판정은
Stage 3에서 수행한다.

### Stage 3 확장 뒤 해석

Stage 3은 같은 `font_rule_candidates.json`에 30개 boundary를 그대로 보존하고 `ruleCandidates`,
`dispositions`, `summary`를 추가했다. 따라서 현재 파일 전체 SHA-256은 Stage 2의 boundary-only
SHA-256과 다르다. 그러나 W0가 소비하는 30개 boundary projection은 바뀌지 않았으며, 확장된
파일을 입력으로 W0 baseline을 메모리에서 재생성한 결과 기존 baseline과 byte 동일했다.

- Stage 3 확장 candidate SHA-256:
  `0c5316fcb0bad11e7af17586062486fcf6a26206a478da1fd9bb641c1aa9474a`
- 재생성 W0 baseline SHA-256:
  `a0fac05c3138471eb3e7404fc701f0053caa6c01a923afae60fd4da64064b466`
- 기존 W0 baseline과 byte equal: `true`

## 3. metric table 기준선

| 항목 | 값 |
| --- | ---: |
| `FONT_METRICS` entry | 600 |
| unique name | 401 |
| regular | 383 |
| bold | 89 |
| italic | 79 |
| bold italic | 49 |
| 중복 `(name, bold, italic)` key | 0 |
| table projection SHA-256 | `bdfec76f6f83894d5c3616796614bed6cc3df622dd638d16d701275636c50f89` |

table projection은 물리 순서와 각 행의 name, bold, italic, em size, Latin range symbol·range 수,
Hangul metric symbol을 보존한다. 원본 advance array 전체를 JSON에 복제하지 않고 source digest와
구조 projection hash로 폐합한다.

## 4. lookup 계약

현재 `find_metric` 선택 사다리는 다음 순서다.

1. `name + bold + italic` 정확 일치
2. 같은 `name + bold`이면서 `italic=false`인 첫 행
3. 같은 `name`의 물리적 첫 행

| 항목 | 값 |
| --- | ---: |
| alias source→target | 67 |
| alias projection SHA-256 | `01becc0ed53257f4cc8238982f4417a5869dfb45f16f80d7b4846e7a88057995` |
| known input | 464 |
| style projection | 1,856 |
| lookup projection SHA-256 | `2012b85fc6de2c103346dbdf3c4d1c34c984d1ae1cc8a71ef06d1baf284fa9fd` |

known input은 metric table의 401개 고유 이름, alias 좌변 가운데 아직 포함되지 않은 이름과
미등록 sentinel의 합집합이다. 각 입력에 `(bold, italic)` 4개 조합을 적용해 선택된 entry index,
metric name과 `boldFallback`을 projection했다. Rust의 보존된 `legacy_find_metric`과 현행 index의
전수 등가는 focused test가 별도로 보호한다.

## 5. source boundary 기준선

| owner | selector |
| --- | ---: |
| `rust-style-resolution` | 4 |
| `rust-metric` | 3 |
| `rust-measurement` | 2 |
| `rust-paint-chain` | 3 |
| `native-skia` | 2 |
| `paint-resource` | 2 |
| `studio-substitution` | 2 |
| `studio-supply` | 2 |
| `studio-detection` | 3 |
| `studio-canvas-patch` | 2 |
| `asset-authority` | 3 |
| `tests-history` | 2 |

30개 candidate projection SHA-256은
`8bf52e13ae5350337be125ecf665446392e54d69b6c39e39c6b44339e3b62af7`이다.

## 6. 공개 native/WASM parity 기준선

fresh native release와 WASM을 source commit에서 빌드해 7개 공개 fixture의 167페이지를 비교했다.

| fixture | SHA-256 | pages | 결과 |
| --- | --- | ---: | --- |
| `samples/exam_kor.hwp` | `0315576fb25dd29ad3b6b188ee2539d0e8d31c15b74847be801c2186a97aac69` | 20 | match |
| `samples/exam_eng.hwp` | `7a5755a2f773fce4d295cbfeb1c5d722edb02c7f920bb067fa56940e8cd6a05b` | 8 | match |
| `samples/exam_math.hwp` | `e40e3d675373c8efb3a844fc71f209600d3b0db987a04b3808b8e74a6b1671fe` | 20 | match |
| `samples/exam_science.hwp` | `22d29786a80d68a9b2ad9294c2dab4915e0eced941e790e37390b14312b8b6a8` | 4 | match |
| `samples/synam-001.hwp` | `1dce9356ec316407b6c684d5a11190a44bb26da643a7749626763e781ab0c13b` | 35 | match |
| `samples/aift.hwp` | `a3e94e613a7d3dad0ee11e2df8f9572a5b7c2d704602960c2075b5fd22df995c` | 74 | match |
| `samples/2010-01-06.hwp` | `d2562d9219fc1d491dd6b9f6d787314153246efb79e18c42c63830ac22194958` | 6 | match |
| 합계 | — | 167 | 불일치 0 |

하네스 원본 보고서는 로컬 `output/`에만 남긴다. 그 보고서의 절대 workspace path를 저장소
산출물로 복제하지 않고, repository-relative fixture path·digest·페이지 결과만 이 문서와
baseline JSON에 정규화했다.

## 7. 실행 환경과 build 산출물

| 항목 | 값 |
| --- | --- |
| OS | Linux `6.6.114.1-microsoft-standard-WSL2`, x86_64 |
| Node.js | `v24.15.0` |
| rustc | `1.93.1 (01f6ddf75 2026-02-11)` |
| cargo | `1.93.1 (083ac5135 2025-12-15)` |
| wasm-pack | `0.15.0` |
| native version | `rhwp v0.8.4` |
| WASM version | `0.8.4` |
| native binary SHA-256 | `dedabb18064973f7483a71bb8e8a707011bd1b2df379979db19746caa6d88b30` |
| `pkg/rhwp.js` SHA-256 | `20d445f1e9c424a7d72d94bfe17032608bfad4d9a1af2a56275f00b91162cb2c` |
| `pkg/rhwp_bg.wasm` SHA-256 | `ebacf1dc16f13ab26b901fe10082ea958a739f3b31d209a9947d31797b75cbb8` |

build 산출물 digest는 실행 재현 근거이지 Git 추적 대상이 아니다.

## 8. 검증 결과

- Font Rule Ledger 계약: 10 passed, 0 failed
- Rust `font_metrics`: 9 passed, 0 failed, 4,038 filtered out
- Studio font contract: 33 passed, 0 failed
- frontend font asset: 6 passed, 0 failed
- native/WASM SVG parity: 7 documents, 167 pages, 0 mismatch
- candidate와 baseline 2회 생성 SHA-256 일치
- private corpus 사용 없음

정확한 명령과 RED→GREEN 흐름은
[Stage 2 working 보고서](../../../working/task_m100_4939_stage2.md)에 기록한다.
