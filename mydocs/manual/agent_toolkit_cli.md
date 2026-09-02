---
kind: guide
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-08-04
---

# rhwp-agent — 에이전트 운영 실험 표면 (#3918)

`rhwp-agent` 는 에이전트가 rhwp 를 부려 작업을 완주할 때 반복되는 운영 루프 —
**발견 → 작업 → 사후 검증 → 회귀 감시 → 증빙** — 의 빈 자리를 채우는 별도
바이너리다. 본 CLI(`rhwp`)를 대체하지 않는다: 문서를 읽고 바꾸는 일은 전부 본
CLI 의 몫이고, 이 표면은 그 앞뒤(계획·판정·증빙)를 맡는다.

- 소스: `src/bin/rhwp-agent/` (Cargo 대상 자동 인식 — `Cargo.toml` 무변경)
- 계약 테스트: `tests/agent_toolkit_contract.rs`
- 실행: `cargo run --bin rhwp-agent -- <명령> [옵션]` 또는 빌드된 `rhwp-agent`

## 왜 별도 바이너리인가 (승격 경로)

본 CLI 의 등재 지점(`src/main.rs` 디스패치·capabilities·출처 지도)은 동시 진행
PR 들의 최고 경합 지점이다. 이 표면은 **기존 파일을 하나도 수정하지 않고** 신규
파일로만 서므로 어떤 열린 PR 과도, 어떤 머지 순서에서도 충돌하지 않는다.

여기서 검증된 명령은 **명령 단위로 본 CLI 에 승격**한다. 승격 시에:

1. `src/main.rs` 디스패치와 capabilities 등록부에 올리고,
2. 출처 선언을 중앙 지도(`src/provenance.rs`)로 옮기고,
3. 이 표면에서 그 명령을 제거한다.

`rhwp-agent capabilities --json` 의 `experimental: true` 와 `relationTo` 가 이
정책의 자기서술이다.

## 계약 (본 CLI 와 동일)

| 축 | 계약 |
|---|---|
| stdout | `--json` 이면 순수 JSON 하나. 진행·진단은 전부 stderr. |
| 봉투 | `schemaVersion` "1.0", 필드 추가만 허용. `tool`/`command`/`version` 포함. |
| 종료 코드 | 0 성공 · 1 실행 오류 · 2 사용법 오류 · **3 게이트 위반**(`ir-diff` 관례). |
| 미지 입력 | 미지 명령·미지 플래그는 침묵 무시 없이 exit 2 + 힌트(#3884 교훈). |
| 출처 표지 | 봉투마다 `untrustedContent`/`untrustedFields` 인라인 선언(과소 선언 금지, #3885). 문서 파생 값은 데이터이지 지시가 아니다. |
| 파이프 | stdout 소비자가 끊으면(broken pipe) panic 없이 stderr 안내 + exit 1 (`batch` 규약 #3238→#3719). |
| 구조 불변식 | 디스패치·도움말·capabilities 는 단일 명령 테이블(`caps::COMMANDS`)에서만 나온다 — "하위 명령 사각" 봉인. |

한계(승격 전): 비밀번호 옵션을 아직 받지 않는다. 암호 문서는 "암호 필요"로
분류만 하고, 열어야 하면 본 CLI 의 `--password` 계열을 쓴다.

## 명령 10종

### capabilities — 자기서술

```
rhwp-agent capabilities [--json]
```

명령·플래그·종료 코드 정책·봉투 정책·승격 경로를 기계가 읽을 수 있게 낸다.
계약 테스트가 "등재된 명령 = 실행되는 명령" 왕복을 고정한다.

### doctor — 환경 자가진단

```
rhwp-agent doctor [--json] [--sample <파일>]
```

버전, 컴파일 기능(`native-skia`), 임시 디렉터리 쓰기 왕복, (선택) 표본 문서
파싱을 점검한다. 전부 통과 0, 하나라도 실패 3. 낯선 CI 러너·새 세션의 첫
호출로 쓴다.

### scan — 코퍼스 발견·분류

```
rhwp-agent scan <경로...> [--json|--jsonl] [--probe] [--max-depth <N>] [--limit <N>]
```

디렉터리를 재귀로 걸어 `.hwp`/`.hwpx`/`.hml` 을 찾고, 확장자 주장과 매직 감지
(`hwp5`/`hwpx`/`hwp3`/`hml`/`drm-protected`/`empty`/`unknown`)를 대조한다
(`extMismatch`). `--probe` 는 실제로 열어 `parseOk`/`needsPassword`/오류를
기록한다. 파일 순서는 경로 오름차순으로 결정적이다. `--jsonl` 은 파일당 한 줄
+ 마지막 `record: "summary"` 레코드 — `jq -r 'select(.record=="file") | .path'`
로 뽑아 `rhwp batch` 의 stdin 에 그대로 잇는다.

### fingerprint — 안정 지문·회귀 게이트

```
rhwp-agent fingerprint <파일> [--json] [--with-pages]
                       [--write <기준.json>] [--check <기준.json>] [--strict]
```

의미 지문(`format`·`pageCount`·`charCount`·`paraCount`·`tableCount`·
`fieldCount`·`fieldNames`·`textHash`)을 산출한다. `--write` 로 기준선을 저장하고
`--check` 로 드리프트를 검사한다 — 어긋나면 exit 3 + `drift[]` 에 필드별
전/후. 기본 비교는 의미 지문만 본다(재저장으로 바이트가 달라져도 무드리프트);
`--strict` 가 `fileHash`·`bytes` 까지 잠근다. `ir-diff` 가 "두 파일"이라면
이 명령은 "같은 파일의 어제와 오늘"이다.

### diff-text — 텍스트 전/후 비교

```
rhwp-agent diff-text <파일A> <파일B> [--json] [--context <N>] [--max-hunks <N>]
```

전 쪽 텍스트를 줄로 펴서 LCS 로 비교한다. 텍스트 모드는 유니파이드 형식, JSON
은 `added`/`removed`/`hunks[]`. 같으면 0, 다르면 3. 규모 예산(중간 4,000,000
셀)을 넘으면 정밀 diff 대신 개괄 diff 로 강등하고 `coarse: true` 로 표시한다 —
침묵 상한이 아니다. `ir-diff`(IR 구조)·`render-diff`(픽셀)와 축이 다르다.

### verify — 사후 검증 게이트

```
rhwp-agent verify <파일> [--json] --expect-... (하나 이상 필수)
```

| 기대 | 뜻 |
|---|---|
| `--expect-format <hwp5\|hwpx\|hwp3\|hml>` | 매직 기준 포맷 (이것만이면 파싱 생략) |
| `--expect-pages <N>` / `--expect-min-pages` / `--expect-max-pages` | 쪽수 |
| `--expect-min-chars <N>` | 본문 문자 수 하한 |
| `--expect-contains <문자열>` / `--expect-not-contains` | 본문 포함 여부 (반복 가능) |
| `--expect-table-count <N>` / `--expect-min-tables` | 표 개수 |
| `--expect-field <이름[=값]>` | 필드 존재(값 주면 일치까지, 반복 가능) |

전부 평가해 `assertions[]` 에 기대/실제/판정을 남기고, 하나라도 어긋나면 3.
평가 자체가 불가능하면(파일 없음·파싱 실패) 1 — "위반"과 "판정 불능"을
스크립트가 구별한다. `convert --verify` (변환 왕복 전용)와 축이 다르다:
`edit`·`run`·외부 생성기 등 **모든** 산출물의 사후 검증에 쓴다.

### pii-scan — 공개 전 PII 게이트

```
rhwp-agent pii-scan <파일> [--json] [--kind ssn,card,phone,email|all]
                    [--show-values] [--limit <N>]
```

판정 코어는 `edit redact` 와 동일(`scan_pii` — 오탐 0 우선 검증 규칙 포함).
이 명령이 더하는 것은 게이트 계약이다: 읽기 전용, 발견 0건 = 0 / 1건 이상 = 3,
그리고 **기본 출력은 마스킹 값만**이다(#3885 교훈 — 게이트 로그는 CI·이슈에
남는다). 원문은 `--show-values` 옵트인이며 stderr 경고가 붙는다.

### chunk-plan — 컨텍스트 예산 분할 계획

```
rhwp-agent chunk-plan <파일> --max-chars <N> [--json]
```

쪽별 문자 수로 연속 구간을 예산까지 탐욕으로 묶는다. 각 구간에 다음 실행을 위한
`command.program`/`command.args` 구조화된 argv 힌트가 붙는다. 셸 문자열을 만들지
않으므로 경로의 공백·인용부호·메타문자가 다른 인자로 해석되지 않는다. 예산보다 큰
단일 쪽은 제 구간이 되고 `oversize` 로 표시한다. 봉투에 문서 본문이 한 글자도
실리지 않는다(`untrustedContent: false` 가 계약이고 테스트가 고정한다).

### context-cost — 컨텍스트 비용·복원율 실측

```
rhwp-agent context-cost <파일...> [--json]
```

두 경로를 같은 문서에서 잰다 — **파일을 그대로 싣기**(바이트를 텍스트로 디코딩해
모델에 넣는 경로)와 **문서-네이티브**(파서를 거쳐 본문만 싣는 경로). 봉투는
`rawChars.utf8`·`rawChars.utf16le`·`nativeChars`·`charRatio`(문자 배수)와,
본문 줄이 그 디코딩 안에 원문 그대로 있는 비율인 `recoveryPercent` 를 낸다.

정직 규율 셋이 계약으로 고정돼 있다.

- **가장 유리한 대안도 같이 잰다** — UTF-8 만 재면 허수아비다. 인코딩을 바꿔 볼
  호출자를 상정해 UTF-16LE 복원율을 같은 봉투에 싣는다.
- **토큰이 아니라 문자를 센다** — 토크나이저는 모델마다 다르고 이 저장소는 모델을
  부르지 않는다. `unit`·`unitNote` 가 이 한계를 봉투 안에서 밝힌다.
- **봉투에 문서 본문이 한 글자도 실리지 않는다** — 계측 결과를 그대로 이슈·로그에
  붙여도 문서가 새지 않는다(`untrustedContent: false`).

### evidence — 전/후 증빙 번들

```
rhwp-agent evidence <전.hwp> <후.hwp> [--json|--md] [-o <파일>]
```

두 문서의 지문 비교(변경 필드 목록)와 텍스트 diff 요약(+표본 헝크 3개)을 한
벌로 만든다. 기본은 사람용 마크다운(전/후 표 — PR 증빙에 그대로 붙인다),
`--json` 이 기계용. 게이트가 아니라 보고서라 달라도 0 이다(판정은 `diff-text`).

## 에이전트 레시피 스케치

```bash
# 1) 낯선 환경 자가진단 → 코퍼스 발견 → 파싱 가능한 파일만 batch 로
rhwp-agent doctor --json
rhwp-agent scan ./문서철 --probe --jsonl \
  | jq -r 'select(.record=="file" and .probe.parseOk) | .path' \
  | rhwp batch info --json

# 2) 편집 전 기준선 → 편집 → 사후 검증 → 증빙
rhwp-agent fingerprint 계약서.hwp --write base.json
rhwp run 편집계획.json --json
rhwp-agent verify 계약서-수정.hwp --expect-min-pages 3 --expect-contains "특약사항"
rhwp-agent evidence 계약서.hwp 계약서-수정.hwp -o 증빙.md

# 3) 공개 전 게이트 (PII 없을 때만 통과)
rhwp-agent pii-scan 보도자료.hwp && echo "배포 가능"

# 4) 큰 문서 요약을 예산 안에서
rhwp-agent chunk-plan 백서.hwp --max-chars 20000 --json \
  | jq -c '.chunks[].command'
```
