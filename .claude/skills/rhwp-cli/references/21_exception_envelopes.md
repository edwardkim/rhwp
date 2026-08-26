# 예외 봉투

실패는 추측하지 않고 아래 네 종류로 먼저 분류한다.
메시지는 `src/main.rs` 문자열을 그대로 인용한다. 의역하지 않는다.

| kind | 대표 stderr | exit | class |
|---|---|---|---|
| missing-file | `오류: 파일을 읽을 수 없습니다 - {path}: {os}` | 1 | runtime |
| bad-page-index | `오류: 페이지 번호가 범위를 벗어났습니다 (0~{max})` | 2 | usage |
| native-skia-missing | `오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.` | 2 | usage |
| load-fail | `오류: 문서 파싱 실패 - {msg}` | 1 | runtime |

## 부가 봉투 (같은 장에서 혼동 방지)

| kind | stderr | exit |
|---|---|---|
| no-input | `오류: 문서 파일 경로를 지정해주세요.` | 2 |
| need-password | `오류: 비밀번호가 필요한 암호 문서입니다` | 2 |
| wrong-password | `오류: 비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다.` | 1 |
| ir-diff-data | (stdout JSON, identical:false) | 3 |
| pdf-direct-no-skia | `direct PDF backend requires a build with the native-skia feature` | 1 |

export-png 기능 부재는 **2**, export-pdf `--backend direct` 기능 부재는 **1**.
코드를 하나로 합치지 말 것.

## 소비

- 실패 경로 stdout 은 0바이트. 빈 JSON 을 만들지 않는다.
- `--json` 을 붙였어도 실패면 jq 하지 않는다.
- missing-file 과 load-fail 은 둘 다 1 이지만 메시지가 갈라진다. 경로 vs 바이트.
- bad-page-index 는 한컴 쪽번호를 그대로 `-p` 에 넣은 실수가 흔하다.

## 픽스처

각 봉투는 `fixtures/envelopes/` 에 같은 id 로 있다. 시험이 stderrContains 와 exitCode 를 고정한다.

### `missing_file`

- kind: `missing-file`
- argv: `rhwp export-svg 없는파일.hwp -p 0`
- exit: 1 (runtime)
- stderr: `오류: 파일을 읽을 수 없습니다`
- 출처: src/main.rs fs::read → EXIT_RUNTIME
- 금지: 없는 경로를 성공으로 읽지 않는다. exit 0 이 아니다.

### `missing_file_info`

- kind: `missing-file`
- argv: `rhwp info 없는파일.hwp --json`
- exit: 1 (runtime)
- stderr: `오류: 파일을 읽을 수 없습니다`
- 출처: 같은 fs::read 계약
- 금지: --json 실패 경로의 stdout 은 비운다.

### `bad_page_index`

- kind: `bad-page-index`
- argv: `rhwp export-svg sample.hwp -p 99`
- exit: 2 (usage)
- stderr: `오류: 페이지 번호가 범위를 벗어났습니다 (0~`
- 출처: src/main.rs page >= page_count → EXIT_USAGE
- 금지: 페이지 범위 초과는 런타임(1)이 아니라 사용법(2).

### `bad_page_png`

- kind: `bad-page-index`
- argv: `rhwp export-png sample.hwp -p 99`
- exit: 2 (usage)
- stderr: `오류: 페이지 번호가 범위를 벗어났습니다 (0~`
- 출처: export-png 동일 검사
- 금지: 한컴 4쪽을 -p 4 로 넣으면 5번째를 찾고 범위 초과가 난다.

### `native_skia_missing`

- kind: `native-skia-missing`
- argv: `rhwp export-png sample.hwp`
- exit: 2 (usage)
- stderr: `오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.`
- 출처: src/main.rs #[cfg(not(feature = "native-skia"))] stub
- 금지: 기능 부재를 성공(0)이나 런타임(1)으로 읽지 않는다. 사용법(2).

### `native_skia_direct_pdf`

- kind: `native-skia-missing`
- argv: `rhwp export-pdf sample.hwp --backend direct`
- exit: 1 (runtime)
- stderr: `direct PDF backend requires a build with the native-skia feature`
- 출처: export-pdf --backend direct 스텁은 RenderError → exit 1
- 금지: export-png 스텁(exit 2)과 코드를 섞지 말 것. direct PDF 는 1.

### `load_fail`

- kind: `load-fail`
- argv: `rhwp info truncated.hwp`
- exit: 1 (runtime)
- stderr: `오류: 문서 파싱 실패 -`
- 출처: LoadError::Other → EXIT_RUNTIME
- 금지: 손상 OLE/잘린 파일을 빈 문서로 성공 처리하지 않는다.

### `load_fail_export`

- kind: `load-fail`
- argv: `rhwp export-text not_hwp.txt --json`
- exit: 1 (runtime)
- stderr: `오류: 문서 파싱 실패 -`
- 출처: detect_format + load_document 실패
- 금지: 확장자만 .hwp 인 텍스트를 본문으로 읽지 않는다.

### `need_password`

- kind: `need-password`
- argv: `rhwp info protected.hwp`
- exit: 2 (usage)
- stderr: `오류: 비밀번호가 필요한 암호 문서입니다`
- 출처: LoadError::NeedPassword → EXIT_USAGE
- 금지: 암호 문서를 로드 실패(1)와 같은 봉투로 묶지 말 것. 비밀번호 없음은 2.

### `wrong_password`

- kind: `wrong-password`
- argv: `rhwp info protected.hwp --password wrong`
- exit: 1 (runtime)
- stderr: `오류: 비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다.`
- 출처: LoadError::WrongPassword → EXIT_RUNTIME
- 금지: 틀린 비밀번호를 사용법(2)으로 재해석하지 않는다.

### `no_input`

- kind: `no-input`
- argv: `rhwp export-svg`
- exit: 2 (usage)
- stderr: `오류: 문서 파일 경로를 지정해주세요.`
- 출처: positional 누락
- 금지: 인자 없음을 런타임으로 읽지 않는다.

### `ir_diff_mismatch`

- kind: `ir-diff-data`
- argv: `rhwp ir-diff a.hwpx b.hwp --json`
- exit: 3 (ir-diff)
- stderr: (비거나 부수)
- 출처: #3274 --json 차이 = exit 3
- 금지: exit 3 을 크래시로 읽지 않는다. identical:false 가 데이터다.

## 표본 경로

- `samples/basic/KTX.hwp` — 기본 표·본문
- `samples/basic/treatise sample.hwp` — info 표 1개 vs export-tables 3개
- `공문.hwp` — 사용자가 준 경로. 상대 경로 함정
- `편람.hwp` — 대형. --max-chars 없이 export-text 금지 기본
- `oracle.hwp` — 한컴 저장본
- `generated.hwp` — rhwp 저장본
- `source.hwpx` — HWPX 원본

## 대화 예

- 사용자: 파일 없는데?
  - 명령: `그대로 실행`
  - 메모: exit 1 메시지
- 사용자: 99쪽
  - 명령: `-p 99`
  - 메모: exit 2 범위

## 재시도

같은 실패 봉투가 나오면 플래그를 발명하지 말고 입력을 고친다.
