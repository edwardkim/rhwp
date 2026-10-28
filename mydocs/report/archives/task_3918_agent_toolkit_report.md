# [#3918] rhwp-agent 실험 표면 — 처리결과 보고서

- 이슈: #3918
- 브랜치: `wip/agent-toolkit` (upstream/devel `df26fe8de` 기준)
- 변경 범위: **기존 파일 0개 수정 — 신규 파일만 추가** (소스 11 + 테스트 1 + 문서 2)

## 1. 배경과 판단

에이전트 운영 루프(발견 → 작업 → 사후 검증 → 회귀 감시 → 증빙)에 전용 표면이
없다는 공백 8건을 이슈 #3918 에 정리했다. 본 CLI 에 명령을 더하는 정공법은
등재 지점(`src/main.rs` 디스패치·capabilities·출처 지도)이 열린 PR 들(#3897·
#3903·#3808 등)과 정면 충돌한다. 그래서:

- `src/bin/rhwp-agent/` **신규 디렉터리 바이너리** — Cargo 대상 자동 인식이라
  `Cargo.toml` 도 수정하지 않는다. 라이브러리 공개 API 만 사용.
- 실험 표면임을 자기서술(`experimental: true`)하고, 검증되면 **명령 단위로 본
  CLI 승격**(그때 capabilities·출처 지도 등재) — 승격 경로를 문서·봉투에 명시.

## 2. 구현 — 명령 9종

| 명령 | 핵심 | 재사용한 코어 |
|---|---|---|
| `capabilities` | 자기서술 (단일 테이블 = 디스패치 = 도움말) | — |
| `doctor` | 환경 자가진단, 실패 시 exit 3 | `DocumentCore` 적재 |
| `scan` | 재귀 발견·매직/확장자 대조·`--probe` 파싱 시도·JSONL | `parser::detect_format` |
| `fingerprint` | 의미 지문·`--write`/`--check` 드리프트 게이트 | 페이지 텍스트·`extract_tables`·`collect_all_fields`·blake3 |
| `diff-text` | 줄 단위 LCS diff, 다르면 exit 3, 규모 초과는 `coarse` 표시 | 페이지 텍스트 |
| `verify` | `--expect-*` 11종 사후 검증 게이트 | 페이지 텍스트·표·필드 질의 |
| `pii-scan` | 읽기 전용 PII 게이트, **기본 마스킹 값만** | `DocumentCore::scan_pii` (redact 와 동일 코어) |
| `chunk-plan` | 쪽 문자 수 기반 분할 계획 (본문 무탑재 봉투) | 페이지 텍스트 길이 |
| `evidence` | 전/후 지문 비교 + diff 요약 번들 (md/JSON) | fingerprint·diff-text 코어 공유 |

구조 불변식: 디스패치·도움말·capabilities 가 전부 `caps::COMMANDS` 단일
테이블에서 나온다 — 테이블에 없는 명령은 실행 불가, 있으면 자동 등재.
"하위 명령 사각"(#3884 계열)의 재발을 검사가 아니라 구조로 막았다.

구현 중 이 표면 자신에서 같은 부류의 결함 1건을 발견·수정했다:
`scan --jsonl | head` 에서 stdout broken pipe 가 panic 을 냈다. `batch` 의
규약(#3238→#3719)대로 stderr 안내 + exit 1 로 통일하고, 전 명령의 stdout 을
공통 경로(`outln!`/`outp!`)로 이관해 재발 지점을 없앴다.

## 3. 검증 실측

### 계약 테스트 (`tests/agent_toolkit_contract.rs`, 13건)

```
test result: ok. 13 passed; 0 failed; 0 ignored  (debug · release-test 프로필 각각)
```

- 등재↔실행 왕복: capabilities 의 전 명령이 실제 디스패치되고 맨몸 호출 계약
  (자기완결 명령 0 / 인자 필요 명령 2)을 지킨다.
- 봉투: 순수 JSON·`schemaVersion`·`untrustedContent↔untrustedFields` 정합.
- 게이트: fingerprint `--check` 훼손 기준선 exit 3 + `drift[]` 필드 지목,
  diff-text 같음 0/다름 3, verify 위반 3/기대 0개는 2, pii-scan 발견 시 3.
- PII 원문 비노출: `--show-values` 없이 봉투에 `raw` 키 부재를 단정.
- scan 분류: 진짜 HWP3 / 확장자 사칭 쓰레기 / 빈 파일 3종 임시 코퍼스에서
  `magicFormat`·`extMismatch`·`probe.parseOk` 판정과 경로 오름차순 결정성.

### 실물 코퍼스 실측 (`samples/`, 675파일)

```
$ rhwp-agent scan samples --probe --json   (stdout 순수 JSON 유지, 진단은 stderr)
total 675 · byFormat {hml 2, hwp3 16, hwp5 383, hwpx 274}
extMismatch 9 · probeFailed 3 (전건 needsPassword=true — 암호 표본)
```

```
$ rhwp-agent fingerprint samples/hwp3-sample.hwp
hwp3 · 16쪽 · 21,523자 · 195문단 · 표 6 · 필드 0 · textHash f722227cd2d7…
두 번 실행 textHash 동일(결정성) · --check 원본 exit 0 · 훼손 기준선 exit 3
```

```
$ rhwp-agent diff-text samples/hwp3-sample.hwp samples/hwpx/form-01.hwpx --json
added 4 · removed 622 · hunks 1 · exit 3   (같은 파일끼리는 exit 0)
$ rhwp-agent pii-scan samples/hwp3-sample.hwp --json
total 1 (email 1) · 마스킹 값만 탑재 · exit 3
$ rhwp-agent chunk-plan samples/hwp3-sample.hwp --max-chars 5000 --json
5구간(1..4/5..8/9..11/12..14/15..16) · oversize 0 · untrustedContent false
$ rhwp-agent doctor --sample samples/hwp3-sample.hwp
4/4 통과 (version·features·tmpWrite·sampleParse 16쪽 91ms) · exit 0
```

### CI 3종

- `cargo fmt --all -- --check`: 신규 파일 전부 지적 0 (rustfmt 적용 완료)
- `cargo clippy --bin rhwp-agent --test agent_toolkit_contract -- -D warnings`: 경고 0
- `cargo test --profile release-test --test agent_toolkit_contract`: 13/13 통과

## 4. 한계 (의도된 범위 밖)

- 비밀번호 옵션 없음 — 암호 문서는 분류만. 승격 시 본 CLI 전역 인증 pre-scan 을 따른다.
- 출처 표지는 명령별 인라인 선언 — 중앙 지도(`provenance.rs`) 등재는 열린 PR 과의
  충돌을 피해 승격 시점으로 미룬다.
- `llms.txt`·`cli_commands.md` 등재도 같은 이유로 머지 후속(한 줄 포인터)으로 미룬다.
- diff-text 는 줄 단위 LCS — 단어 단위 정밀도·이동 감지는 다음 단계.

## 5. 후속 제안

1. 실사용 피드백 후 명령 단위 승격 (1순위 후보: `verify`·`pii-scan` — 게이트 수요가 가장 넓다)
2. `scan` ↔ `batch` 직결 레시피를 recipes 시리즈에 추가
3. `fingerprint` 기준선을 CI 아티팩트로 쓰는 회귀 감시 워크플로 실험
