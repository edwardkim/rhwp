---
kind: working
status: active
issue: 5641
---

# rhwp-q-page-items — 쪽 조판 항목 조회 (#5641)

작업 브랜치: `feat/q-page-items` (`upstream/devel` 기준)
대상 바이너리: `src/bin/rhwp-q-page-items.rs`
이슈: [#5641](https://github.com/edwardkim/rhwp/issues/5641)

## 1. 한 줄

에이전트가 쪽의 조판 항목(문단·표·도형·미주 등)을 읽기 전용으로 꺼낸다.
기존 `DocumentCore::dump_page_items_json` 만 부르고 문서를 고치지 않는다.

## 2. 왜 별도 바이너리인가

본 CLI(`src/main.rs`)의 `dump-pages` 와 `rhwp-agent` 는 여러 열린 PR 이
동시에 만지는 경합 지점이다. Cargo 는 `src/bin/*.rs` 를 자동 인식하므로
`Cargo.toml` 을 고치지 않고도 `rhwp-q-page-items` 가 선다.

만진 파일:

| 경로 | 역할 |
|------|------|
| `src/bin/rhwp-q-page-items.rs` | CLI·JSON 봉투·같은 파일의 시험 |
| `mydocs/working/agent_q_page_items.md` | 이 기록과 실측 JSON |

만지지 않은 것: `Cargo.toml`, `src/main.rs`, `src/bin/rhwp-agent/**`, `gym/`,
`crates/`, `Cargo.lock`. 편집 API 는 호출하지 않는다.

## 3. 계약

```
rhwp-q-page-items <파일> [--page <N>] [--json]
```

- `--page` 는 0부터 세는 쪽 번호이며 **생략 가능**하다.
- 생략하면 API 의 기본 모양 `dump_page_items_json(None)` 으로 **모든 쪽**을
  덤프한다. `pageFilter` 는 JSON `null` 이다.
- `--page N` 이면 `dump_page_items_json(Some(N))` 으로 그 쪽만 덤프한다.
- `--json` 이면 stdout 에 JSON 봉투 하나만 낸다. 진단은 stderr.
- 문서는 `DocumentCore::from_bytes` 로 연다.

봉투 필드:

| 필드 | 값 |
|------|-----|
| `schemaVersion` | `"1.0"` |
| `tool` | `rhwp-q-page-items` |
| `command` | `page-items` |
| `version` | 크레이트 버전 (`0.8.4`) |
| `untrustedContent` | `true` |
| `untrustedFields` | `["source", "pages"]` |
| `source` | 입력 경로 |
| `pageCount` | 문서 쪽 수 |
| `pageFilter` | 요청한 0-based 쪽, 생략 시 `null` |
| `pages` | 코어가 낸 조판 항목 배열 |

종료 코드:

| 코드 | 뜻 | 실측 |
|------|----|------|
| 0 | 성공 | `samples/form-01.hwp --page 0 --json` |
| 1 | 실행 오류(없는 파일·파싱 실패·쪽 범위 밖) | `--page 9999` → stderr `오류: 페이지 번호가 범위를 벗어났습니다 (0~0)` |
| 2 | 사용법(미지 플래그·파일 누락·`--page` 값 없음) | `--nope` · 파일 없음 |

## 4. 검증

명령과 결과는 이 작업 트리에서 실행한 값이다.

```
git config core.autocrlf false
$env:CARGO_TARGET_DIR='C:\Users\swsz9\.rhwp-shared-target'
rustfmt --edition 2021 --config newline_style=Unix src/bin/rhwp-q-page-items.rs
cargo test --bin rhwp-q-page-items
cargo run --bin rhwp-q-page-items -- --json --page 0 samples/form-01.hwp
cargo fmt --all -- --check
```

| 명령 | 결과 |
|------|------|
| `rustfmt --edition 2021 --config newline_style=Unix --check src/bin/rhwp-q-page-items.rs` | 통과 |
| `cargo test --bin rhwp-q-page-items` | 14 passed; 0 failed |
| `cargo run --bin rhwp-q-page-items -- --json --page 0 samples/form-01.hwp` | exit 0, 아래 봉투 |
| `cargo fmt --all -- --check` | 통과 |
| 미지 플래그 `--nope` | exit 2 |
| `--page 9999` | exit 1 |
| `--page` 생략 | 모든 쪽 (`pageFilter: null`) |

같은 파일 `#[cfg(test)]` 가 사용법·쪽 범위·`form-01.hwp` 0쪽 봉투 키·
`--page` 생략 시 전체 덤프·`exam-kor-2p.hwp` 2쪽·편집 API 부재를 고정한다.

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-page-items -- --json --page 0 samples/form-01.hwp
```

환경: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

`samples/form-01.hwp` 는 1쪽이다. `--page 0` 이면 `pageFilter: 0`, `pages` 길이 1.
0쪽 단은 문단 13개(`itemCount: 13`, 전부 `kind: "fullParagraph"`)이며
`extras` 는 빈 배열이다. 아래는 그 실행의 stdout 원문이다.

```json
{
  "command": "page-items",
  "pageCount": 1,
  "pageFilter": 0,
  "pages": [
    {
      "bodyArea": {
        "height": 876.8533333333334,
        "width": 566.9333333333334,
        "x": 113.38666666666667,
        "y": 132.26666666666668
      },
      "columns": [
        {
          "hwpUsedHeight": 335.81333333333333,
          "index": 0,
          "itemCount": 13,
          "items": [
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 26.453333333333333,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 34.45333333333333
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 1686,
                  "columnStart": 0,
                  "lineHeight": 1984,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1984,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 1686,
                  "columnStart": 0,
                  "lineHeight": 1984,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1984,
                  "textStart": 0
                }
              },
              "paraIndex": 0,
              "textPreview": "",
              "vpos": {
                "first": 0,
                "last": 0,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 13.333333333333334,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 21.333333333333336
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                }
              },
              "paraIndex": 1,
              "textPreview": "",
              "vpos": {
                "first": 2584,
                "last": 2584,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 26.453333333333333,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 34.45333333333333
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 1686,
                  "columnStart": 0,
                  "lineHeight": 1984,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1984,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 1686,
                  "columnStart": 0,
                  "lineHeight": 1984,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1984,
                  "textStart": 0
                }
              },
              "paraIndex": 2,
              "textPreview": "",
              "vpos": {
                "first": 4184,
                "last": 4184,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 13.333333333333334,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 21.333333333333336
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                }
              },
              "paraIndex": 3,
              "textPreview": "",
              "vpos": {
                "first": 6768,
                "last": 6768,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 19.333333333333332,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 27.333333333333332
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 1233,
                  "columnStart": 0,
                  "lineHeight": 1450,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1450,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 1233,
                  "columnStart": 0,
                  "lineHeight": 1450,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1450,
                  "textStart": 0
                }
              },
              "paraIndex": 4,
              "textPreview": "",
              "vpos": {
                "first": 8368,
                "last": 8368,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 13.333333333333334,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 21.333333333333336
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                }
              },
              "paraIndex": 5,
              "textPreview": "",
              "vpos": {
                "first": 10418,
                "last": 10418,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 26.453333333333333,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 34.45333333333333
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 1686,
                  "columnStart": 0,
                  "lineHeight": 1984,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1984,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 1686,
                  "columnStart": 0,
                  "lineHeight": 1984,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1984,
                  "textStart": 0
                }
              },
              "paraIndex": 6,
              "textPreview": "",
              "vpos": {
                "first": 12018,
                "last": 12018,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 13.333333333333334,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 21.333333333333336
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                }
              },
              "paraIndex": 7,
              "textPreview": "",
              "vpos": {
                "first": 14602,
                "last": 14602,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 26.453333333333333,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 34.45333333333333
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 1686,
                  "columnStart": 0,
                  "lineHeight": 1984,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1984,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 1686,
                  "columnStart": 0,
                  "lineHeight": 1984,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1984,
                  "textStart": 0
                }
              },
              "paraIndex": 8,
              "textPreview": "",
              "vpos": {
                "first": 16202,
                "last": 16202,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 13.333333333333334,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 21.333333333333336
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                }
              },
              "paraIndex": 9,
              "textPreview": "",
              "vpos": {
                "first": 18786,
                "last": 18786,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 13.333333333333334,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 21.333333333333336
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                }
              },
              "paraIndex": 10,
              "textPreview": "",
              "vpos": {
                "first": 20386,
                "last": 20386,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 13.333333333333334,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 21.333333333333336
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                }
              },
              "paraIndex": 11,
              "textPreview": "",
              "vpos": {
                "first": 21986,
                "last": 21986,
                "resets": [],
                "rewinds": []
              }
            },
            {
              "endnoteSource": null,
              "height": {
                "lineHeightSum": 13.333333333333334,
                "lineSpacingSum": 8.0,
                "spacingAfter": 0.0,
                "spacingBefore": 0.0,
                "total": 21.333333333333336
              },
              "isEndnote": false,
              "kind": "fullParagraph",
              "lineSegs": {
                "count": 1,
                "first": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                },
                "last": {
                  "baselineDistance": 850,
                  "columnStart": 0,
                  "lineHeight": 1000,
                  "lineSpacing": 600,
                  "segmentWidth": 42520,
                  "textHeight": 1000,
                  "textStart": 0
                }
              },
              "paraIndex": 12,
              "textPreview": "",
              "vpos": {
                "first": 23586,
                "last": 23586,
                "resets": [],
                "rewinds": []
              }
            }
          ],
          "usedDiff": 0.0,
          "usedHeight": 335.81333333333333,
          "zoneYOffset": 0.0
        }
      ],
      "displayPage": 1,
      "extras": [],
      "pageIndex": 0,
      "pageNumber": 1,
      "section": 0
    }
  ],
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-page-items",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "pages"
  ],
  "version": "0.8.4"
}
```

사용법 오류 실측 (종료 코드 2, stdout 비어 있음):

```
$ rhwp-q-page-items --nope samples/form-01.hwp
오류: 알 수 없는 옵션입니다 - --nope
사용법: rhwp-q-page-items <파일> [--page <N>] [--json]
```

쪽 범위 초과는 실행 오류 1 이다.

```
$ rhwp-q-page-items --page 9999 samples/form-01.hwp
오류: 페이지 번호가 범위를 벗어났습니다 (0~0)
```

`--page` 생략은 사용법 오류가 아니다. `dump_page_items_json(None)` 이 모든
쪽을 내고 `pageFilter` 는 `null` 이다.

## 6. 시험

```
cargo test --bin rhwp-q-page-items
```

결과: `14 passed; 0 failed`.

- `form_sample_page_zero_emits_items` — 봉투 필드와 0쪽 1건
- `form_sample_omitted_page_dumps_all` — `pageFilter: null`, 쪽 수와 배열 길이 일치
- `exam_kor_omitted_page_dumps_two_when_present` — 2쪽 표본의 전체 덤프
- `--page` 위치·`--page=`·`--json` 파싱
- `--page` 값 누락 / 비정수 / 음수 / 파일 누락 / 미지 플래그 → 2
- 9999쪽 → 1
- 편집 API 부재

## 7. fmt

```
cargo fmt --all
cargo fmt --all -- --check
```

통과. rustfmt `newline_style = Unix`.

## 8. 만진 것 / 만지지 않은 것

만진 것:

| 경로 | 역할 |
|------|------|
| `src/bin/rhwp-q-page-items.rs` | CLI + 같은 파일 단위 시험 |
| `mydocs/working/agent_q_page_items.md` | 이 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 구현
- `dump_page_items_json` 본체
