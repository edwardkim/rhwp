---
kind: report
status: active
canonical: mydocs/report/task_m100_3885_report.md
last_verified: 2026-08-04
---

# #3885 처리 기록 — 출처 표지가 빠진 봉투 4건 (redact 가 원문 개인정보를 표지 없이 싣는다)

- Issue: [#3885](https://github.com/edwardkim/rhwp/issues/3885) — [#3787 S1] 출처 표지가
  "항상 실린다"는데 실제로는 빠진 봉투가 있다
- 브랜치 `task/3885-envelope-provenance-marker`

## 증상

`capabilities` 의 `jsonContract.provenance.policy` 는 *"표지는 항상 실린다 — 문서를 열지 않는
명령의 봉투도 `untrustedContent:false` 를 명시한다"* 라고 선언하는데, 표지가 아예 없는 봉투가
4건 있었다.

| 명령 | 봉투가 담는 문서 파생 값 |
|---|---|
| `edit redact --dry-run --json` | **`findings[].raw` — 원문 개인정보** |
| `edit sanitize --json` | `removed[].before` — 지운 메타데이터 원문 |
| `export-ir-schema` | (문서를 열지 않음 — `false` 를 명시해야 하는 쪽) |
| `export-capabilities-schema` | (동일) |

`redact` 가 가장 나쁘다. `--dry-run` 은 **무엇을 지울지 먼저 보여주는 것**이 권장 흐름이라
봉투의 `findings[].raw` 에 주민등록번호·카드번호·전화·이메일이 그대로 들어간다. 가장 민감한
값을 싣는 봉투가 "이건 문서에서 온 값이다"라는 표지를 빠뜨리면 #3787 S1 이 세운 계약의 목적이
정확히 그 지점에서 무너진다.

## 근인 — 가드가 본 것과 보지 않은 것

이슈가 지시한 대로 **가드가 왜 못 잡았는지부터** 확인했다.

`provenance_map_covers_every_json_command` 는 지도가 명령을 **등재**했는지만 본다.

```rust
let missing: Vec<&String> = declared.iter()
    .filter(|n| !commands.contains_key(n.as_str()))
    .collect();
```

`edit` 은 지도에 있다(`src/provenance.rs`). `export-ir-schema`·`export-capabilities-schema` 도
있다. **등재는 전부 통과한다.** 그런데 실행 결과 봉투에 표지가 실리는지는 아무도 보지 않았다.
지도와 봉투는 다른 문제인데 가드가 하나만 본 것이다.

두 번째 사각도 있었다. 스윕 레시피의 `edit` 은 `set-cell` 하나뿐이라 `redact`·`sanitize` 는
실행조차 되지 않았다. 표지가 있었어도 이 경로는 검사 밖이었다.

## 수정

**가드부터 넓혔다** (이슈가 지정한 순서).

- `every_executed_envelope_actually_carries_the_marker` — 스윕이 실제로 실행한 모든 봉투에
  `untrustedContent` 키가 있는지 본다. 값이 아니라 **키의 존재**를 본다: 키가 없으면 소비자는
  "안전하다"와 "이 빌드는 표지를 모른다"를 구별할 수 없고, 후자라면 다른 봉투의 표지도 못 믿는다.
- `document_free_schema_commands_still_state_false` — `SWEEP_EXEMPT` 는 "문서 오라클을 만들 수
  없다"는 뜻이지 "표지를 안 달아도 된다"가 아니다. 두 스키마 명령을 직접 실행해
  `untrustedContent:false` 가 **명시**되는지 본다.
- `edit redact --dry-run --json` 레시피 추가 — 이 경로가 스윕에 들어와야 위 가드가 의미를 갖는다.

**그다음 표지를 달았다.**

- `edit redact` / `edit sanitize` — `provenance::marked(envelope, "edit")`.
- `export-ir-schema` / `export-capabilities-schema` — 봉투 모드에만 단다. `--bare` 는 봉투가
  아니라 스키마 본문이라 JSON Schema 검증기에 그대로 먹이는 용도이므로 이물을 섞지 않는다.

**지도에 두 필드를 선언했다.** 표지만 달면 `marked` 가 지도를 보고 판정하므로, 선언이 없으면
`findings[].raw` 를 싣고도 `untrustedContent:false` 로 나간다 — 표지가 없는 것보다 나쁘다.

- `findings[].raw` — redact 가 찾아낸 원문 개인정보
- `removed[].before` — sanitize 가 지우기 전 메타데이터 원문

## 판단이 필요했던 지점

이슈가 짚은 대로 `findings[].raw` 는 "문서 파생"과 별개로 **"로그에 남기지 마라"** 가 더 중요한
경고다. 이번에는 출처 표지만 달았다 — 민감도 축을 새로 만드는 것은 봉투 스키마 변경이라 별도
판단이 필요하고, 표지 누락은 그와 무관하게 지금 계약 위반이기 때문이다. `--no-raw`(#3841)가
이미 원문을 뺄 수단을 제공하므로, 민감도 신호는 그 위에서 논의하는 편이 낫다.

## 검증

- `rustfmt` 로 변경 파일 3개 포맷 확인.
- 이 PC 는 MSVC 링커(`dbghelp.lib`) 손상으로 `cargo test` 가 아예 돌지 않는다. **CI 가 유일한
  판정자다.**
- 새 가드는 red→green 을 로컬에서 보이지 못했다. 다만 가드가 보는 대상(`untrustedContent` 키)과
  수정이 만드는 것(`marked` 호출)이 직결돼 있고, 수정 전 상태에서 그 키가 없다는 것은
  `git grep` 으로 확인했다(4개 출력부 어디에도 `marked` 가 없었다).
