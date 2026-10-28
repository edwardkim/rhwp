---
kind: working
status: active
issue: 5616
---

# rhwp-q-font-trace — 쪽 글꼴 결정 추적 조회 (#5616)

작업 브랜치: `feat/q-font-trace` (`upstream/devel` 기준)
이슈: https://github.com/edwardkim/rhwp/issues/5616

## 1. 한 줄

에이전트가 쪽의 글꼴 결정 추적을 읽기 전용으로 꺼낸다. 기존
`DocumentCore::get_font_decision_trace_native` 만 부르고 문서를 고치지 않는다.

## 2. 왜 별도 바이너리인가

본 CLI(`src/main.rs`)와 `rhwp-agent` 는 여러 열린 PR 이 동시에 만지는 경합
지점이다. Cargo 는 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 을 고치지
않고도 `rhwp-q-font-trace` 가 선다.

만진 파일:

| 경로 | 역할 |
|------|------|
| `src/bin/rhwp-q-font-trace.rs` | CLI·JSON 봉투·같은 파일의 시험 |
| `mydocs/working/agent_q_font_trace.md` | 이 기록과 실측 JSON |

만지지 않은 것: `Cargo.toml`, `src/main.rs`, `src/bin/rhwp-agent/**`, `gym/`,
`crates/`, `Cargo.lock`. 편집 API 는 호출하지 않는다.

## 3. 계약

```
rhwp-q-font-trace <파일> --page <N> [--json]
```

- `--page` 는 0부터 세는 쪽 번호이며 필수다.
- `--json` 이면 stdout 에 JSON 봉투 하나만 낸다. 진단은 stderr.
- 문서는 `DocumentCore::from_bytes` 로 연다.
- 추적은 `core.get_font_decision_trace_native(page, "{}")` 의 JSON 을 파싱해
  `trace` 필드에 중첩한다.

봉투 필드:

| 필드 | 값 |
|------|-----|
| `schemaVersion` | `"1.0"` |
| `tool` | `rhwp-q-font-trace` |
| `command` | `font-trace` |
| `version` | 크레이트 버전 (`0.8.4`) |
| `untrustedContent` | `true` |
| `untrustedFields` | `["source", "trace"]` |
| `source` | 입력 경로 |
| `page` | 요청한 0-based 쪽 |
| `trace` | 코어가 낸 추적 JSON 객체 |

종료 코드:

| 코드 | 뜻 | 실측 |
|------|----|------|
| 0 | 성공 | `samples/form-01.hwp --page 0 --json` |
| 1 | 실행 오류(없는 파일·파싱 실패·쪽 범위 밖) | `--page 9999` → stderr `오류: 페이지 9999을(를) 찾을 수 없습니다` |
| 2 | 사용법(미지 플래그·`--page` 누락·파일 누락) | `--nope` · `--page` 없음 |

## 4. 검증

명령과 결과는 이 작업 트리에서 실행한 값이다.

```
git config core.autocrlf false
$env:CARGO_TARGET_DIR='C:\Users\swsz9\.rhwp-shared-target'
rustfmt --edition 2021 --config newline_style=Unix src/bin/rhwp-q-font-trace.rs
cargo test --bin rhwp-q-font-trace
cargo run --bin rhwp-q-font-trace -- --json --page 0 samples/form-01.hwp
cargo fmt --all -- --check
```

| 명령 | 결과 |
|------|------|
| `rustfmt --edition 2021 --config newline_style=Unix --check src/bin/rhwp-q-font-trace.rs` | 통과 |
| `cargo test --bin rhwp-q-font-trace` | 11 passed; 0 failed |
| `cargo run --bin rhwp-q-font-trace -- --json --page 0 samples/form-01.hwp` | exit 0, 아래 봉투 |
| `cargo fmt --all -- --check` | 통과 |
| 미지 플래그 `--nope` | exit 2 |
| `--page 9999` | exit 1 |
| `--page` 누락 | exit 2 |

같은 파일 `#[cfg(test)]` 가 사용법·쪽 범위·`form-01.hwp` 0쪽 봉투 키·편집
API 부재를 고정한다.

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-font-trace -- --json --page 0 samples/form-01.hwp
```

`samples/form-01.hwp` 0쪽은 글자 6개를 추적했다. `requestedFace` 는 `한컴바탕`,
메트릭 별칭은 `Haansoft Batang`, `layoutHash.value` 는
`fb9c00e83a6de14be87b11740b9330ce28074c446180f93a61dda21fc61ae6db` 다.
아래는 그 실행의 stdout 원문이다.

```json
{
  "command": "font-trace",
  "page": 0,
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-font-trace",
  "trace": {
    "backendSummary": {
      "canvas2d": {
        "reasons": [
          "studioSnapshotRequired"
        ],
        "status": "unsupported"
      },
      "canvaskit": {
        "reasons": [
          "studioSnapshotRequired"
        ],
        "status": "unsupported"
      },
      "layout": {
        "reasons": [],
        "status": "complete"
      },
      "native": {
        "reasons": [
          "nativeSkiaFeatureUnavailable"
        ],
        "status": "unsupported"
      }
    },
    "counts": {
      "charactersSeen": 6,
      "recordsEmitted": 6,
      "recordsOmitted": 0,
      "runsSeen": 15
    },
    "layoutHash": {
      "algorithm": "sha256",
      "value": "fb9c00e83a6de14be87b11740b9330ce28074c446180f93a61dda21fc61ae6db"
    },
    "normalizedHash": {
      "algorithm": "sha256",
      "value": "acc029282d4fdc1417db56e50bf4fc50ff56767c82e740695848f15a6f303a72"
    },
    "reasons": [
      {
        "code": "backendUnsupported",
        "detail": "Studio Canvas2D and CanvasKit observations require a current renderer snapshot."
      },
      {
        "code": "ledgerSourceDrift",
        "detail": "W1 candidate identities still join, but their recorded Rust source digests predate this Stage 2 trace-only refactor."
      },
      {
        "code": "sourceMappingMismatch",
        "detail": "At least one render-tree character could not be joined exactly to source IR coordinates."
      }
    ],
    "records": [
      {
        "document": {
          "altType": null,
          "embedded": null,
          "face": null,
          "inheritedLanguageSlot": null,
          "languageSlot": 0,
          "substFont": null
        },
        "layoutMetric": {
          "aliasResolvedFace": "Haansoft Batang",
          "baseAdvanceHwpunit": 1000,
          "characterMatch": "hit",
          "finalAdvanceHwpunit": 1000,
          "matchKind": "boldOnly",
          "metricEntry": 6,
          "requestedFace": "한컴바탕",
          "transforms": [],
          "widthSource": "embeddedMetric"
        },
        "layoutName": {
          "cssFamilyChain": [],
          "normalizedFace": null,
          "requestedFace": null,
          "steps": []
        },
        "oracle": {
          "knownLimitations": [
            "No oracle profile was supplied to this read-only query."
          ],
          "profileId": null,
          "status": "notProvided"
        },
        "paint": {
          "canvas2d": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "canvaskit": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "native": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "nativeSkiaFeatureUnavailable"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          }
        },
        "provenance": [
          {
            "candidateId": "candidate.28970f1984ed0e4ce06d",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.28970f1984ed0e4ce06d",
            "evidenceStatus": "inferred",
            "knownLimitations": [
              "The predicate selects advances; it does not prove glyph identity.",
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-measurement.28970f1984ed0e4ce06d",
            "sourceOwner": "rust-measurement"
          },
          {
            "candidateId": "candidate.7a2702136cc80f952133",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7a2702136cc80f952133",
            "evidenceStatus": "unknown",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-metric.7a2702136cc80f952133",
            "sourceOwner": "rust-metric"
          },
          {
            "candidateId": "candidate.7fd02f944f1f667fbd8a",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7fd02f944f1f667fbd8a",
            "evidenceStatus": "verified-by-test",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "metric-entry",
            "ruleId": "rule.rust-metric.7fd02f944f1f667fbd8a",
            "sourceOwner": "rust-metric"
          }
        ],
        "recordId": "page:0:run:12:char:0",
        "source": {
          "charOffset": null,
          "charShapeId": null,
          "character": "여",
          "codePoint": 50668,
          "nestedPath": [],
          "paragraphIndex": 10,
          "runIndex": 12,
          "sectionIndex": 0,
          "status": "unavailable"
        }
      },
      {
        "document": {
          "altType": null,
          "embedded": null,
          "face": null,
          "inheritedLanguageSlot": null,
          "languageSlot": 0,
          "substFont": null
        },
        "layoutMetric": {
          "aliasResolvedFace": "Haansoft Batang",
          "baseAdvanceHwpunit": 1000,
          "characterMatch": "hit",
          "finalAdvanceHwpunit": 1000,
          "matchKind": "boldOnly",
          "metricEntry": 6,
          "requestedFace": "한컴바탕",
          "transforms": [],
          "widthSource": "embeddedMetric"
        },
        "layoutName": {
          "cssFamilyChain": [],
          "normalizedFace": null,
          "requestedFace": null,
          "steps": []
        },
        "oracle": {
          "knownLimitations": [
            "No oracle profile was supplied to this read-only query."
          ],
          "profileId": null,
          "status": "notProvided"
        },
        "paint": {
          "canvas2d": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "canvaskit": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "native": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "nativeSkiaFeatureUnavailable"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          }
        },
        "provenance": [
          {
            "candidateId": "candidate.28970f1984ed0e4ce06d",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.28970f1984ed0e4ce06d",
            "evidenceStatus": "inferred",
            "knownLimitations": [
              "The predicate selects advances; it does not prove glyph identity.",
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-measurement.28970f1984ed0e4ce06d",
            "sourceOwner": "rust-measurement"
          },
          {
            "candidateId": "candidate.7a2702136cc80f952133",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7a2702136cc80f952133",
            "evidenceStatus": "unknown",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-metric.7a2702136cc80f952133",
            "sourceOwner": "rust-metric"
          },
          {
            "candidateId": "candidate.7fd02f944f1f667fbd8a",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7fd02f944f1f667fbd8a",
            "evidenceStatus": "verified-by-test",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "metric-entry",
            "ruleId": "rule.rust-metric.7fd02f944f1f667fbd8a",
            "sourceOwner": "rust-metric"
          }
        ],
        "recordId": "page:0:run:12:char:1",
        "source": {
          "charOffset": null,
          "charShapeId": null,
          "character": "기",
          "codePoint": 44592,
          "nestedPath": [],
          "paragraphIndex": 10,
          "runIndex": 12,
          "sectionIndex": 0,
          "status": "unavailable"
        }
      },
      {
        "document": {
          "altType": null,
          "embedded": null,
          "face": null,
          "inheritedLanguageSlot": null,
          "languageSlot": 0,
          "substFont": null
        },
        "layoutMetric": {
          "aliasResolvedFace": "Haansoft Batang",
          "baseAdvanceHwpunit": 1000,
          "characterMatch": "hit",
          "finalAdvanceHwpunit": 1000,
          "matchKind": "boldOnly",
          "metricEntry": 6,
          "requestedFace": "한컴바탕",
          "transforms": [],
          "widthSource": "embeddedMetric"
        },
        "layoutName": {
          "cssFamilyChain": [],
          "normalizedFace": null,
          "requestedFace": null,
          "steps": []
        },
        "oracle": {
          "knownLimitations": [
            "No oracle profile was supplied to this read-only query."
          ],
          "profileId": null,
          "status": "notProvided"
        },
        "paint": {
          "canvas2d": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "canvaskit": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "native": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "nativeSkiaFeatureUnavailable"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          }
        },
        "provenance": [
          {
            "candidateId": "candidate.28970f1984ed0e4ce06d",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.28970f1984ed0e4ce06d",
            "evidenceStatus": "inferred",
            "knownLimitations": [
              "The predicate selects advances; it does not prove glyph identity.",
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-measurement.28970f1984ed0e4ce06d",
            "sourceOwner": "rust-measurement"
          },
          {
            "candidateId": "candidate.7a2702136cc80f952133",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7a2702136cc80f952133",
            "evidenceStatus": "unknown",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-metric.7a2702136cc80f952133",
            "sourceOwner": "rust-metric"
          },
          {
            "candidateId": "candidate.7fd02f944f1f667fbd8a",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7fd02f944f1f667fbd8a",
            "evidenceStatus": "verified-by-test",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "metric-entry",
            "ruleId": "rule.rust-metric.7fd02f944f1f667fbd8a",
            "sourceOwner": "rust-metric"
          }
        ],
        "recordId": "page:0:run:12:char:2",
        "source": {
          "charOffset": null,
          "charShapeId": null,
          "character": "에",
          "codePoint": 50640,
          "nestedPath": [],
          "paragraphIndex": 10,
          "runIndex": 12,
          "sectionIndex": 0,
          "status": "unavailable"
        }
      },
      {
        "document": {
          "altType": null,
          "embedded": null,
          "face": null,
          "inheritedLanguageSlot": 0,
          "languageSlot": 0,
          "substFont": null
        },
        "layoutMetric": {
          "aliasResolvedFace": "Haansoft Batang",
          "baseAdvanceHwpunit": 500,
          "characterMatch": "hit",
          "finalAdvanceHwpunit": 500,
          "matchKind": "boldOnly",
          "metricEntry": 6,
          "requestedFace": "한컴바탕",
          "transforms": [],
          "widthSource": "metricHalfSpace"
        },
        "layoutName": {
          "cssFamilyChain": [],
          "normalizedFace": null,
          "requestedFace": null,
          "steps": []
        },
        "oracle": {
          "knownLimitations": [
            "No oracle profile was supplied to this read-only query."
          ],
          "profileId": null,
          "status": "notProvided"
        },
        "paint": {
          "canvas2d": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "canvaskit": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "native": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "nativeSkiaFeatureUnavailable"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          }
        },
        "provenance": [
          {
            "candidateId": "candidate.28970f1984ed0e4ce06d",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.28970f1984ed0e4ce06d",
            "evidenceStatus": "inferred",
            "knownLimitations": [
              "The predicate selects advances; it does not prove glyph identity.",
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-measurement.28970f1984ed0e4ce06d",
            "sourceOwner": "rust-measurement"
          },
          {
            "candidateId": "candidate.7a2702136cc80f952133",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7a2702136cc80f952133",
            "evidenceStatus": "unknown",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-metric.7a2702136cc80f952133",
            "sourceOwner": "rust-metric"
          },
          {
            "candidateId": "candidate.7fd02f944f1f667fbd8a",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7fd02f944f1f667fbd8a",
            "evidenceStatus": "verified-by-test",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "metric-entry",
            "ruleId": "rule.rust-metric.7fd02f944f1f667fbd8a",
            "sourceOwner": "rust-metric"
          }
        ],
        "recordId": "page:0:run:12:char:3",
        "source": {
          "charOffset": null,
          "charShapeId": null,
          "character": " ",
          "codePoint": 32,
          "nestedPath": [],
          "paragraphIndex": 10,
          "runIndex": 12,
          "sectionIndex": 0,
          "status": "unavailable"
        }
      },
      {
        "document": {
          "altType": null,
          "embedded": null,
          "face": null,
          "inheritedLanguageSlot": null,
          "languageSlot": 0,
          "substFont": null
        },
        "layoutMetric": {
          "aliasResolvedFace": "Haansoft Batang",
          "baseAdvanceHwpunit": 1000,
          "characterMatch": "hit",
          "finalAdvanceHwpunit": 1000,
          "matchKind": "boldOnly",
          "metricEntry": 6,
          "requestedFace": "한컴바탕",
          "transforms": [],
          "widthSource": "embeddedMetric"
        },
        "layoutName": {
          "cssFamilyChain": [],
          "normalizedFace": null,
          "requestedFace": null,
          "steps": []
        },
        "oracle": {
          "knownLimitations": [
            "No oracle profile was supplied to this read-only query."
          ],
          "profileId": null,
          "status": "notProvided"
        },
        "paint": {
          "canvas2d": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "canvaskit": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "native": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "nativeSkiaFeatureUnavailable"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          }
        },
        "provenance": [
          {
            "candidateId": "candidate.28970f1984ed0e4ce06d",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.28970f1984ed0e4ce06d",
            "evidenceStatus": "inferred",
            "knownLimitations": [
              "The predicate selects advances; it does not prove glyph identity.",
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-measurement.28970f1984ed0e4ce06d",
            "sourceOwner": "rust-measurement"
          },
          {
            "candidateId": "candidate.7a2702136cc80f952133",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7a2702136cc80f952133",
            "evidenceStatus": "unknown",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-metric.7a2702136cc80f952133",
            "sourceOwner": "rust-metric"
          },
          {
            "candidateId": "candidate.7fd02f944f1f667fbd8a",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7fd02f944f1f667fbd8a",
            "evidenceStatus": "verified-by-test",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "metric-entry",
            "ruleId": "rule.rust-metric.7fd02f944f1f667fbd8a",
            "sourceOwner": "rust-metric"
          }
        ],
        "recordId": "page:0:run:12:char:4",
        "source": {
          "charOffset": null,
          "charShapeId": null,
          "character": "입",
          "codePoint": 51077,
          "nestedPath": [],
          "paragraphIndex": 10,
          "runIndex": 12,
          "sectionIndex": 0,
          "status": "unavailable"
        }
      },
      {
        "document": {
          "altType": null,
          "embedded": null,
          "face": null,
          "inheritedLanguageSlot": null,
          "languageSlot": 0,
          "substFont": null
        },
        "layoutMetric": {
          "aliasResolvedFace": "Haansoft Batang",
          "baseAdvanceHwpunit": 1000,
          "characterMatch": "hit",
          "finalAdvanceHwpunit": 1000,
          "matchKind": "boldOnly",
          "metricEntry": 6,
          "requestedFace": "한컴바탕",
          "transforms": [],
          "widthSource": "embeddedMetric"
        },
        "layoutName": {
          "cssFamilyChain": [],
          "normalizedFace": null,
          "requestedFace": null,
          "steps": []
        },
        "oracle": {
          "knownLimitations": [
            "No oracle profile was supplied to this read-only query."
          ],
          "profileId": null,
          "status": "notProvided"
        },
        "paint": {
          "canvas2d": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "canvaskit": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "studioSnapshotRequired"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          },
          "native": {
            "candidates": [],
            "capabilities": [],
            "certainty": "unsupported",
            "failures": [
              "nativeSkiaFeatureUnavailable"
            ],
            "requested": "한컴바탕",
            "resolved": null,
            "source": null,
            "status": "unsupported"
          }
        },
        "provenance": [
          {
            "candidateId": "candidate.28970f1984ed0e4ce06d",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.28970f1984ed0e4ce06d",
            "evidenceStatus": "inferred",
            "knownLimitations": [
              "The predicate selects advances; it does not prove glyph identity.",
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-measurement.28970f1984ed0e4ce06d",
            "sourceOwner": "rust-measurement"
          },
          {
            "candidateId": "candidate.7a2702136cc80f952133",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7a2702136cc80f952133",
            "evidenceStatus": "unknown",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "unknown",
            "ruleId": "rule.rust-metric.7a2702136cc80f952133",
            "sourceOwner": "rust-metric"
          },
          {
            "candidateId": "candidate.7fd02f944f1f667fbd8a",
            "evidenceAnchor": "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#candidate.7fd02f944f1f667fbd8a",
            "evidenceStatus": "verified-by-test",
            "knownLimitations": [
              "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            ],
            "reason": "ledgerSourceDrift",
            "relationType": "metric-entry",
            "ruleId": "rule.rust-metric.7fd02f944f1f667fbd8a",
            "sourceOwner": "rust-metric"
          }
        ],
        "recordId": "page:0:run:12:char:5",
        "source": {
          "charOffset": null,
          "charShapeId": null,
          "character": "력",
          "codePoint": 47141,
          "nestedPath": [],
          "paragraphIndex": 10,
          "runIndex": 12,
          "sectionIndex": 0,
          "status": "unavailable"
        }
      }
    ],
    "schemaVersion": 1,
    "scope": {
      "appliedLimits": {
        "maxCharacters": 1024
      },
      "pageIndex": 0,
      "requestedLimits": {
        "maxCharacters": 1024
      }
    },
    "status": "complete"
  },
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "trace"
  ],
  "version": "0.8.4"
}
```
