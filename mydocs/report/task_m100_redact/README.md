---
kind: report
status: active
canonical: mydocs/report/task_m100_redact/README.md
last_verified: 2026-08-02
---

# 로드맵 #3719 §6-11 처리 기록 — `edit redact` · `edit sanitize` (공개 전 정리)

## 문제

문서를 공개·제출하기 직전에 두 가지를 반드시 지워야 한다.

1. **본문 안의 개인정보** — 계약서·신청서·회의록에 남은 주민등록번호·전화번호·
   이메일·카드번호. 사람이 눈으로 찾으면 반드시 빠뜨린다(표 셀·글상자 안은 특히).
2. **문서 메타데이터** — 작성자·최종수정자·작성일시·미리보기. 본문을 아무리 다듬어도
   파일 속성에 실명이 남고, **미리보기에는 본문에서 이미 지운 문장이 남아 있다**.
   PDF 로 내보내면 사라지지만 HWP/HWPX 원본을 그대로 배포하면 전부 따라간다.

rhwp 에는 이 두 작업을 위한 출구가 없었다. `export-text` 로 뽑아 눈으로 훑고
`replace-text` 로 하나씩 치는 방법뿐이었는데, ① 주소(몇 쪽 어느 셀)를 잃고
② 무엇을 놓쳤는지 알 수 없으며 ③ 메타데이터는 손도 못 댄다.

## 설계 원칙 — 왜 이렇게 **좁혔는가**

이 명령의 핵심 위험은 기능 부족이 아니라 **오탐**이다. 마스킹은 되돌릴 수 없고,
오탐 하나가 본문의 금액·일련번호·문서번호를 영구히 `******` 로 만든다. 사용자는
원본을 이미 덮어썼을 수 있다.

그래서 판정 규칙을 다음 순서로 세웠다.

> **형태가 맞아도 검증을 통과하지 못하면 탐지하지 않는다.**

| 종류 | 형태 | 추가 검증 | 근거 |
| --- | --- | --- | --- |
| 주민등록번호 | `######-#######` | ① 생년월일 실재(윤년 포함) ② 성별/세기 코드 1~8 ③ mod 11 검증 숫자 | 형태만 보면 `123456-1234567` 같은 **예시 문자열**까지 지운다. 검증 숫자만 걸면 임의 숫자쌍 11개 중 1개가 우연히 통과하므로 세기 코드까지 함께 건다 |
| 카드번호 | `4-4-4-4`(`-`/공백), Amex `4-6-5`, 연속 15·16자리 | Luhn | Luhn 없이는 16자리 회계 숫자·계좌번호를 전부 삼킨다 |
| 전화번호 | `01[016789]-3~4자리-4자리`, `02-3~4자리-4자리` | (하이픈 필수) | 하이픈 없는 `01012345678` 은 문서번호와 형태가 같다 |
| 이메일 | `지역부@라벨(.라벨)+` | 라벨 2개 이상 + 최상위 도메인이 영문 2자 이상 | `a@localhost`·`user@example.12` 는 주소가 아니다 |

공통으로 **숫자 경계 검사**를 건다 — 앞뒤가 숫자면 더 긴 토큰의 일부로 보고 버린다.
22자리 계좌번호 안에서 16자리 부분열을 카드로 오인하지 않기 위해서다.

### 의도적으로 **하지 않는** 것

- **02 외의 지역번호(031·051·064…)는 탐지하지 않는다.** 목록을 넣으면 커버리지는
  오르지만, `0XX-XXX-XXXX` 형태의 회계 코드·민원 접수번호와 구별할 근거가 사라진다.
  로드맵이 지정한 형태(`010-####-####`·`02-###-####`)로 좁히고, 넓히려면 실문서
  오탐 실측을 근거로 별도 이슈에서 확장한다.
- **13·14·19자리 카드는 탐지하지 않는다.** 자릿수를 넓힐수록 Luhn 만으로는
  1/10 확률의 우연 통과가 그대로 오탐이 된다.
- **여권번호·계좌번호·주소는 v1 범위 밖이다.** 체크섬이 없거나(계좌) 형태가 기관마다
  달라(여권) 보수적 판정이 불가능하다. 넣으면 "오탐 0" 원칙 자체가 무너진다.

놓치는 쪽으로 기운 설계이므로 **`--dry-run` 이 권장 첫 단계**다. 무엇이 지워질지
전부 보여준 뒤에 사용자가 적용을 결정한다.

## 구현

### 1) 탐지 코어 — `src/document_core/queries/pii_scan.rs` (신규, 662줄)

**읽기 전용 판정만** 한다. 문서를 바꾸지 않는다.

- `detect_values(text, kinds) -> Vec<(PiiKind, String)>` — 순수 함수. 문서 없이
  규칙 자체를 테스트할 수 있게 공개했다(오탐 회귀 케이스의 1차 방어선).
- `DocumentCore::scan_pii(kinds, mask) -> Vec<PiiFinding>` — 본문·표 셀·글상자를
  순회해 값을 모으고, **주소는 [`grep`](../../../src/document_core/queries/grep.rs) 을
  재사용**해 붙인다. 구역·문단·**페이지**·문자 오프셋이 매치마다 따라온다
  (분할 표의 행별 페이지 보정까지 grep 이 이미 처리한다).
- `mask_value(raw, mask)` — 영숫자만 마스킹 문자로 바꾸고 구분자(`-`·`@`·`.`)는
  남긴다. **문자 수가 보존된다** — 길이가 바뀌면 줄바꿈이 밀려 조판이 흔들린다.

정규식 의존성을 새로 넣지 않았다. 체크섬·경계 검사를 어차피 손으로 해야 하므로
문자 스캐너가 규칙을 더 정확히 표현한다.

### 2) `rhwp edit redact` — 개인정보 마스킹

```
rhwp edit redact <파일> [--kind ssn|phone|email|card|all] [--mask <문자>]
                        [--dry-run] [--verify] [-o <출력>|--in-place] [--json]
```

- **새 편집 로직이 없다.** 실제 변경은 검증된 `replace_all_native` 를 값 단위로
  호출한다. 짧은 값이 긴 값의 부분열일 때 원문을 깨뜨리지 않도록 **긴 값부터** 친다.
- **원본 보호**: `-o` 도 `--in-place` 도 없으면 `exit 2` 로 거부한다. 다른 edit
  명령처럼 `_redacted.hwp` 기본 이름을 만들지도 **않는다** — 되돌릴 수 없는 작업에서
  "어디에 무엇이 생겼는지 모르는 상태"를 만들지 않기 위해서다. `-o` 가 원본 자신을
  가리켜도 같은 사고이므로 같은 코드로 거부한다(`fs::canonicalize` 비교).
- `--mask` 는 **영숫자가 아닌 한 글자**만 받는다. 두 글자면 자릿수 보존이 깨지고,
  영숫자면 마스킹인지 본문인지 구별할 수 없다. 조용히 자르지 않고 `exit 2`.
- 쓰기는 `atomic_file::write_atomically` — `--in-place` 도중 실패해도 원본이
  잘린 채 남지 않는다.
- `--dry-run` 은 파일을 **만들지 않는다**. 봉투에 `findings[{kind, raw, masked,
  section, paragraph, page, charOffset}]`.
- `--verify`(#3702)·`changedPages`(#3712) 는 기존 편집 명령과 같은 경로를 쓴다.

> `findings[].raw` 는 **원문 개인정보 그 자체**다. help·도구 설명·문서에 "로그에
> 남기지 말 것"을 명시했다. 감사에 필요해서 넣었지만 새는 곳이기도 하다.

### 3) `rhwp edit sanitize` — 메타데이터 제거

```
rhwp edit sanitize <파일> [--keep-preview] [-o <출력>] [--json]
```

본문은 건드리지 않는다. 지우는 것은 셋이다.

1. **OLE 요약 정보** (`\x05HwpSummaryInformation`) — title·subject·author·keywords·
   comments·lastSavedBy·revisionNumber·dateString(문자열)과 createdAt·lastSavedAt·
   lastPrintedAt(FILETIME).
   속성 오프셋 표가 **절대 위치**를 담고 있어 크기를 줄이면 나머지 속성이 전부
   어긋난다. 그래서 **바이트 길이를 바꾸지 않는다** — 문자열은 `cch=1`(NUL 하나)로
   만들고 남은 자리를 0으로 덮으며, FILETIME 은 0(미설정)으로 만든다.
2. **HWPX 저작자 메타** (`Contents/content.hpf` 의 `<opf:metadata>`) — 직렬화기가
   원본에서 그대로 splice 하는 유일한 저작자 경로다
   ([`content.rs`](../../../src/serializer/hwpx/content.rs)). 중립 블록으로 바꾼다.
3. **미리보기** — PrvText·PrvImage(ZIP 엔트리와 HWP5 계약 스트림 양쪽).

`removed[{field, before}]` 로 무엇을 지웠는지 남긴다. 조용히 지우면 감사가 불가능하다.

#### 거짓 보고를 막은 두 지점

- **미리보기 텍스트는 "지금 본문과 다를 때만" 지우고 보고한다.** HWP5 직렬화기는
  PrvText 가 비면 본문 앞부분으로 다시 채운다(`supplement_preview`). 본문과 같은
  미리보기는 유출이 아니라 파생물이므로 매번 "지웠다"고 보고하면 감사 기록이
  거짓말이 된다. **다를 때**가 진짜 사고다 — 본문에서 지운 문장이 미리보기에만
  남아 있는 경우.
- **HWPX 원본의 `/HwpSummaryInformation` 은 HWPX 로 저장할 때 건드리지 않는다.**
  그 스트림은 파일에 없던 계약 fallback 상수이고(`parser::hwpx::contract_streams`)
  HWPX 산출물에도 실리지 않는다. 없던 것을 지웠다고 보고하지 않기 위해 출력 형식이
  HWP5(변환)일 때만 처리한다.

결과적으로 **두 번째 실행은 `removedCount: 0`** 이다 — 첫 실행이 실제로 지웠다는 증거다.

### 4) 자기서술 (드리프트 가드)

- MCP 도구 2종 `hwp_redact`·`hwp_sanitize` — `inputSchema` 에 `type`/`properties`/
  `required` 배열, 선언한 속성 전부를 `cli.args`·`optionalArgs.when` 에 배선.
- `capabilities.commands[edit].flags` 에 `--kind`·`--mask`·`--in-place`·
  `--keep-preview`·`--verify` 추가, `outputFields` 에 새 봉투 필드 추가.
- `--help` 에 두 하위 명령 절 추가(capabilities↔help 양방향 가드).

## 실측 (evidence.txt 원문)

`samples/field-01.hwp` 의 누름틀에 **가공** 값을 심어 만든 문서로 확인했다
(실제 개인정보는 저장소에 넣지 않는다). 유효한 값 4개와 검증에 실패하는 미끼 2개를
같은 문서에 함께 넣었다.

| 심은 값 | 판정 | 근거 |
| --- | --- | --- |
| `900101-1234568` | 탐지 | mod 11 검증 숫자 통과 |
| `900101-1234567` | **미탐지** | 검증 숫자 불일치 |
| `4111-1111-1111-1111` | 탐지 | Luhn 통과 |
| `1234-5678-9012-3456` | **미탐지** | Luhn 실패(합 64) |
| `010-1234-5678` | 탐지 | 이동전화 형태 |
| `hong@example.com` | 탐지 | 라벨 2개 + 영문 TLD |

- `--dry-run --json` → `findingCount: 4`, 미끼 2개는 `findings` 에 없음, 산출 파일 미생성.
- 실제 마스킹 → `redactedCount: 4`, `changedPages: [0]`, `--verify` `identical: true`.
- 산출 본문: `카드 ****-****-****-**** / 미끼 900101-1234567 / 미끼 1234-5678-9012-3456`
  — **미끼는 훼손되지 않았다**.
- `-o`·`--in-place` 없이 실행 → `exit 2`, stdout 0바이트, 원본 바이트 불변.
- `sanitize`(HWP5) → `removedCount: 10`(title·author·dateString·keywords·lastSavedBy·
  revisionNumber·createdAt·lastSavedAt·preview.text·preview.image), 재실행 `0`.
- `sanitize`(HWPX) → `removedCount: 3`(hwpx.metadata·preview.text·preview.image),
  재실행 `0`, `outputFormat: hwpx`.
- HWPX→HWP5 변환 sanitize → `removedCount: 9`(요약 정보 6종 포함).
- 본문 무변경: `export-text` 전 페이지 문자 단위 동일 — HWP5 3쪽, HWPX 23쪽.
- `--keep-preview` 산출물은 `thumbnail` 추출 exit 0, 기본(제거) 산출물은 exit 1.
- FILETIME 변환 교차 검증: `dateString` "2026년 3월 9일 월요일 오전 3:24:42"(KST) ↔
  `createdAt` `2026-03-08T18:24:42Z` — 9시간 차로 일치.

## 검증

- `cargo build --release --bin rhwp` 통과.
- 신규 `tests/redact_sanitize_contract.rs` **11건 green** — 오탐 0·자릿수 보존·원본
  보호(exit 2)·dry-run 무산출·`--mask` 검증·`--kind` 필터·sanitize 본문 불변·
  `--keep-preview`·형식 보존·실패 경로 stdout 0바이트·자기서술 배선.
- `pii_scan` 단위 테스트 **15건 green** — 탐지 규칙마다 오탐 회귀 케이스(세기 코드
  범위·윤년·구분자 혼용·자릿수 범위·숫자 경계·전화 지역번호·이메일 라벨/TLD).
- 무회귀: `cli_json_contract` 26건 · `mcp_server_contract` 22건.
- 드리프트 가드(preflight) 전부 통과 — MCP inputSchema 모양(27개 도구) · 선언 속성
  ↔ CLI 배선 · capabilities ↔ `--help` 상호 커버(54개 명령) · `--json` 명령 ↔ MCP
  도구 커버 · 선언 flags 실재(65개) · 실패 경로 stdout 0바이트.
- `cargo clippy -- -D warnings` 0 · rustfmt clean.

## 남은 것

- **지역번호 확장**: 02 외 지역번호는 실문서 오탐 실측을 근거로 별도 이슈에서 판단한다.
- **여권번호·계좌번호**: 체크섬이 없어 보수적 판정이 불가능하다. 사용자 지정 정규식
  축(`--pattern`)을 열어 주는 편이 안전하다 — 판정 책임이 사용자에게 넘어간다.
- **`findings[].raw` 마스킹 옵션**: 감사 기록을 남기되 원문을 감추고 싶은 수요가
  있을 수 있다(`--no-raw`). v1 은 로드맵 명세대로 원문을 넣고 문서로 경고했다.
- **DocumentSummaryInformation**: 한컴이 회사명(company)을 별도 속성 집합에 쓰는
  문서가 있으면 그 스트림도 대상에 넣어야 한다. 실측 샘플에서는 발견되지 않았다.
