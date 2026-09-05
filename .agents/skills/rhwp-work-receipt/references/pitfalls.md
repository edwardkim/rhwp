# 함정 — `toolVersion` 과 귀속 비주장

이 장은 에이전트가 영수증을 **과대해석**하는 자리만 모은다.

## 1. `toolVersion` 불일치

영수증 필드 `toolVersion` 은 재현 조건이다. 같은 계획이어도 rhwp 마이너가
바뀌면 직렬화·치환 엔진이 다른 바이트를 낼 수 있다.

판정 순서:

1. 상대 영수증의 `toolVersion` 을 읽는다.
2. 지금 돌릴 바이너리 버전과 대조한다 (`rhwp replay … --json` 의 같은 필드,
   또는 `rhwp --version` 계열).
3. **다를 때** `reproduced: false` 를 "상대가 거짓말을 했다"로 단정하지 않는다.
   보고서에 두 버전을 적고, 같은 버전으로 다시 검증하거나 멈춘다.

이 스킬은 버전을 고정하는 새 플래그를 만들지 않는다. 기존 필드다.

픽스처: `fixtures/capsules/toolversion_mismatch.capsule.json`,
예제 [19_toolversion_pitfall.md](../examples/19_toolversion_pitfall.md).

## 2. 귀속·서명 주장 금지

3해시와 캡슐은 다음만 증명한다.

- **무엇**을 입력으로
- **어떤 계획 원문**으로
- **어떤 산출 바이트**가 나왔는가

증명하지 **않는** 것:

- 누가 실행했는가 (작성자, 에이전트 이름, PR 저자)
- 언제 실행했는가 (타임스탬프는 영수증 필수 키가 아니다)
- 서명이 유효한가 (4년 축, `--sign-key` / `verify-signature`)
- 입력이 합법적으로 입수됐는가

사용자가 "누가 했는지 증명해" 라고 하면 이 스킬은 거절하고, 3해시는
신원이 아니라고 말한다. `--sign-key` 를 기본 경로에 끼워 넣어
"서명했으니 작성자 확인"이라고 쓰지 마라.

`fixtures/catalog.json` 의 `attributionClaim` / `signatureClaim` 은
둘 다 `false` 다. 시험이 이 값을 고정한다.

## 3. 그 밖의 실록

1. **`replay` 가 사용자 `output` 을 만든다고 가정.** 임시 재실행만 한다. 실파일은 `run`.
2. **계획 pretty-print 후 검증.** `planSha256` 이 바뀐다. 원문 바이트를 보존하라.
3. **캡슐을 열어 저장.** 부모 해시가 깨진다. 재발급.
4. **`--parent` 를 cwd 기준으로 해석.** 캡슐 파일 기준이다.
5. **`audit` 가 하위 폴더를 본다고 가정.** 비재귀 `*.capsule.json`.
6. **빈 폴더 audit 를 루프.** exit 2, 봉투 없음.
7. **`reproduced` 타입 혼동.** replay 는 bool\|null, audit 는 number.
8. **exit 3 을 예외로 raise.** 근거 봉투를 잃는다.
9. **머리 없는 `lineage` 를 사용법으로 읽음.** 머리 없음은 exit 1 (IO).
10. **`parent` 필드 부재를 뿌리로 읽음.** 뿌리는 `null`. 필드 없음은 깨짐.
11. **gym pack 으로 채점.** 이 경로는 실작업 증명이다. gym 금지.
12. **새 CLI 이름.** `receipt` / `prove` / `work-receipt` 는 없다.

## 4. 워크스루

- [19_toolversion_pitfall.md](../examples/19_toolversion_pitfall.md)
- [20_no_attribution.md](../examples/20_no_attribution.md)
