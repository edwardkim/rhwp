# 예외 경로 — 불량 샘플 (`bad_sample`)

증상: 바이너리는 있는데 자가검증이 돌지 못하거나 파서가 거절한다.
종료 코드 **1**, `exceptions[].kind=="bad_sample"`.

통과를 위조하지 않는다. 텍스트 파일을 `.hwp` 로 바꿔 저장한 입력을
성공으로 보고하면 이후 모든 레시피가 거짓이 된다.

## 분류가 막는 입력

| kind | 예 |
|---|---|
| `missing` | `samples/` 없는 축소 체크아웃, 잘못된 `--sample` |
| `empty` | 0바이트 |
| `too_small` | 몇 바이트짜리 더미 |
| `not_document` | `text_named_hwp.hwp` 픽스처 |
| `avoid` | `gym/`, `output/`, `samples/broken/` |

## 처방

```bash
python tools/agent_onboarding/rhwp_doctor.py --sample samples/basic/english.hwp --json
```

번들이 없으면 사용자에게 실제 `.hwp`/`.hwpx` 를 받는다.
OLE 또는 ZIP 시그니처가 있는 파일만 자가검증에 넣는다.

## 픽스처로 실패를 재현

```bash
python tools/agent_onboarding/rhwp_doctor.py \
  --sample tools/agent_onboarding/fixtures/samples/empty.hwp --json
python tools/agent_onboarding/rhwp_doctor.py \
  --sample tools/agent_onboarding/fixtures/samples/text_named_hwp.hwp --json
```

바이너리가 있을 때 두 호출 모두 임계 FAIL 이어야 한다.

## 파서가 거절하는 경우

매직은 맞는데 `info` 가 exit 1 이면 잘린 OLE/ZIP 이거나 암호·DRM 이다.
`classify_selftest_failure` 는 이도 `bad_sample` 로 묶는다.
같은 명령을 손으로 실행해 stderr 를 읽는다.

```bash
rhwp info <그파일> --json
```

## 사례 01 — 확장자만 맞음

Windows 가 숨긴 확장자. 매직을 본다.

## 사례 02 — HWP 가 아니라 HWPX 이름

이름은 힌트일 뿐. ZIP 이면 hwpx 로 분류.

## 사례 03 — PDF 를 줌

시그니처 `%PDF`. `not_document`.

## 사례 04 — DOCX 를 줌

ZIP 이라 `hwpx` 로 오분류될 수 있다. `info` 가 거절하면 그 신호가 맞다.

## 사례 05 — 디렉터리를 --sample

파일이 아니므로 missing.

## 사례 06 — 상대 경로 cwd

닥터는 상대 경로를 그대로 연다. cwd 를 맞춘다.

## 사례 07 — LFS 포인터

Git LFS 포인터 텍스트는 `not_document`. lfs pull.

## 사례 08 — 권한 거부

크기 읽기 실패. OS 오류를 detail 에 싣는다.

## 사례 09 — 잠긴 파일

다른 프로세스가 독점. 닫고 재시도.

## 사례 10 — 한글 경로

닥터는 Path 로 연다. 콘솔 깨짐과 실제 실패를 혼동하지 않는다.

## 사례 11 — 심볼릭 링크

대상이 파일이면 따른다.

## 사례 12 — 너무 큰 파일

하한만 있다. 상한은 타임아웃.

## 사례 13 — 암호 문서

비밀번호 없으면 exit 2 계열로 분류될 수 있다.

## 사례 14 — DRM

열기 실패. 온보딩 샘플로 쓰지 않는다.

## 사례 15 — 생성 직후 빈 산출

`output/` 은 avoid.

## 사례 16 — fuzz 코퍼스

자가검증 후보가 아니다.

## 사례 17 — gym 입력

온보딩 경로에서 제외.

## 사례 18 — 잘린 ZIP

PK 만 있고 central directory 없음. info FAIL.

## 사례 19 — 잘린 OLE

매직만 8바이트. too_small.

## 사례 20 — 성공 위조 유혹

테스트를 skip 하거나 fixture 를 교체하지 않는다.

## 성공

정상 샘플로 다시 돌려 `selftest-info` 와 `selftest-export-text` 가 PASS.
