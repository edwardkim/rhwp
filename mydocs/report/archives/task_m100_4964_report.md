---
kind: report
status: completed
canonical: mydocs/report/archives/task_m100_4964_report.md
last_verified: 2026-08-23
---

# Task M100 #4964 — W6 metric 계보 분리 최종 보고

Issue: #4964

- **메인테이너 최종 승인**: 2026-08-23

## 1. 결론

#4964 W6는 단일 `FONT_METRICS` source에 섞여 있던 historical generated data, #2430
measured/manual overlay와 runtime lookup의 소유권을 행동 변화 없이 분리했다. 600개 항목은
generated 595개와 overlay 5개로 구성되며, 논리 iterator는 기존 순서를 그대로 유지한다.

이 작업은 metric 값을 교정하거나 fallback 정책을 새로 만든 것이 아니다. 원본 font와 당시 실행
manifest가 없는 595개 항목을 source-exact로 과장하지 않고 `unknown` 계보로 보존했고, 확인 가능한
증거만 W1·W5·#2430 및 추적 font에 연결했다. 따라서 후행 W7은 데이터의 역사적 위치와 검증 수준을
구분해 canonical registry와 backend projection을 설계할 수 있다.

## 2. 최종 구조

| 산출물 | 소유권 |
| --- | --- |
| `src/renderer/font_metrics_data.rs` | type·폭 조회·alias·first-match lookup/index facade |
| `src/renderer/font_metrics_generated.rs` | historical generated region 0..594 |
| `src/renderer/font_metrics_overlays.rs` | #2430 measured/manual overlay 595..599 |
| `font_metric_lineage_manifest.json` | 600행 provenance·evidence·semantic hash 정본 |
| `font_metric_lineage_manifest.schema.json` | manifest fail-closed schema |
| `font_metric_pre_split_baseline.json` | 분리 전 행동과 exhaustive projection 기준선 |
| `scripts/font_metric_lineage.mjs` | Rust composition·manifest 결정론 검사기 |
| `font-metric-gen` | 명시적 plan 기반 generated fragment·metadata 생성기 |

runtime은 generated와 overlay iterator를 순서대로 연결한다. 정렬·dedupe·style 정규화·숫자 교정은
없으며 기존 600개 배열을 복사해 새로운 정책 registry를 만들지 않는다.

## 3. 계보 결과

| 분류 | 수 | 판정 |
| --- | ---: | --- |
| 전체 metric entry | 600 | 안정 ID와 current index 전부 존재 |
| historical generated | 595 | origin `unknown`, 위치만으로 exact를 주장하지 않음 |
| measured overlay | 5 | #2430 475/475 측정 정합성 확인 |
| fully source-exact | 0 | 원본·face·전체 생성 manifest 없는 항목을 승격하지 않음 |
| partially byte-verified source | 1 | Noto Sans KR, printable ASCII 범위 한정 |
| W1 metric entry link | 600 | 전 항목 연결 |
| W5 Oracle Profile link | 2 | 실제 관찰 relation만 연결 |

manifest entry digest는
`054f4725162ddc95c4b00e00186955b1d7f10599d401f66f75b6dd52a1147032`다. 분리 전후
composition·metric·width·lookup hash도 모두 동일하다.

## 4. generator 경계

generator는 더 이상 core lookup 파일 전체를 쓰지 않는다. 입력 plan이 각 source의 order, 상대 경로,
face index와 예상 identity를 명시해야 하며 다음을 fail-closed한다.

- 암묵적 directory sort·dedupe를 통한 생성
- 절대 경로와 `..` 입력·evidence 경로
- 실제 SFNT identity와 plan identity 불일치
- evidence 없는 verified provenance·license
- core, overlay 또는 불완전 canary의 canonical generated DB overwrite

허용 출력은 generated Rust data fragment와 별도 provenance metadata뿐이다. metadata에는 source
digest, face index, SFNT naming record, license/provenance evidence, Hangul compression error와
generator·plan·output hash가 들어간다. 공개 TTF와 합성 TTC canary는 같은 plan에서 byte-identical
결과를 냈다. 기존 595개 전체 원본 plan은 복원되지 않았으므로 canonical generated DB를 재생성하지
않았다.

## 5. 행동 불변 검증

| 검증 | 결과 |
| --- | --- |
| W1·W5·lineage·generator Node contract | 39/39 |
| Rust font metric unit·legacy lookup | 9/9 |
| full nextest | 8,073/8,073, 정책 skip 39 |
| #2430 overlay | 5 face, 475/475 exact |
| Native Skia 지정 renderer | 2/2와 4/4 |
| native release·Docker WASM build | 통과 |
| native/WASM SVG parity | 공개 7문서 167쪽, mismatch 0 |
| Clippy `--all-targets -D warnings` | 통과 |
| doc tests | 8 통과, 3 ignore |
| format·diff check | 통과 |

native/WASM parity가 보호한 공개 문서는 `exam_kor`, `exam_eng`, `exam_math`, `exam_science`,
`synam-001`, `aift`, `2010-01-06`이다. private corpus와 한컴/Hyper-V Oracle은 기존 evidence의 입력
identity가 바뀌지 않았으므로 재실행하지 않았다.

상세 명령, 산출물 hash와 공유 target·generated suite 검증 경계는
[Stage W6-5 보고](../../working/task_m100_4964_w6_stage5.md)에 기록했다.

## 6. W6가 해결한 것과 해결하지 않은 것

W6가 해결한 것은 "어디에서 온 값인지 모르는 상태"와 "어떤 정책으로 쓰는지"가 같은 파일에 섞여
있던 구조다. 이제 generated, overlay, lookup과 provenance의 소유자가 분리되고, 증거 수준을 기계가
검사한다.

W6가 해결하지 않은 것은 다음과 같다.

- 595개 unknown origin의 원본 font·face·license 복원
- 잘못된 metric 숫자의 교정 또는 신규 font 추가
- fallback alias·우선순위 변경
- Rust·TypeScript·CanvasKit의 canonical registry와 projection 통합
- 공공 HWP 편집 습관의 장평·자간·kerning 대응 정책

이 항목들은 W6 결과를 입력으로 삼되 별도 이슈·계획·승인으로 처리해야 한다. 특히 W7은 manifest를
runtime truth로 바로 import하지 않고, verified·partial·unknown 상태를 보존하는 projection 계약부터
설계해야 한다.

## 7. 완료 판정과 남은 절차

수행계획의 12개 보호 불변식과 완료 조건을 모두 충족했다. 신규 font bytes, private corpus 식별 자료,
ignored build·SVG 비교 산출물은 Git 변경에 포함하지 않았다.

메인테이너는 2026-08-23 이 최종 보고서를 승인했다. 최신 `upstream/devel`과의 충돌·검증 범위를
다시 확인한 뒤 remote push와 PR 생성은 각각 별도 승인을 받아 진행한다. CI 성공 뒤
self-review·merge·#4964 close와 상위 #4960의 W6 상태 갱신도 프로젝트 절차에 따른다.
