# 05 §5 게이트 — 근거 대장 밖 주장은 산출물에 못 들어간다

정본: playbook §5, engagement.py `validate_spec`.

## 계약 한 줄

**근거 대장에 실존하는 EV id 에 연결되지 않은 CLAIM 은 납품할 수 없다.**

이것이 이 capability 의 핵심이다. 문장이 그럴듯한지, 전략이 옳은지는
재지 않는다. 연결만 잰다.

## 골격이 만드는 자리

Phase C 는 질문마다 매치가 있으면:

```
[CLAIM-1: 에이전트가 근거 EV-1, EV-2, EV-3 로 작성]
```

에이전트는 이 플레이스홀더를 실제 주장으로 바꾸고 같은 문단(또는 같은
표 행)에 EV id 를 남긴다.

권장 형식:

```
발주 공고는 표준 API 연계를 필수기능으로 못 박는다. [근거: EV-1, EV-7]
```

끝의 근거 연결표(주장 | 근거 ID | 파일·좌표)도 실제 인용에 맞게 고친다.
게이트는 문단·제목·표 행을 한 단위로 보고 CLAIM 과 EV 의 동거를 검사한다.

## 위반 세 종류

| kind | 뜻 | 고치는 법 |
| --- | --- | --- |
| `placeholder` | 플레이스홀더가 그대로 | 실제 문장으로 교체. EV 유지 |
| `unknown-evidence` | 대장에 없는 EV-99 등 | 지어낸 id 삭제. 필요하면 질문 보강 후 엔진 재실행 |
| `unlinked` | CLAIM 옆에 실존 EV 가 없음 | 같은 단위에 대장 EV 를 명시 |

`verdict` 는 위반이 하나라도 있으면 `fail`. exit 3.

## 0건 질문

매치 0건 절에는 CLAIM 자체가 없다. 그곳에 "시장이 성장할 것"을 쓰면
그 문장에 CLAIM-n 이 없는 한 게이트는 CLAIM 단위로만 본다. **그래서**
스킬 정지 규칙 ST-FORECAST 가 게이트 앞을 지킨다. 게이트는 CLAIM 형식의
누수를, 스킬은 형식 없는 전망 삽입을 막는다.

## 호출

```bash
python3 tools/strategist/engagement.py --validate spec.json --evidence evidence.json
# 기본 evidence 경로는 spec 옆 evidence.json
# --no-sws-audit 로 SWS 만 생략 (연결 게이트는 그대로)
```

stdout 이 판정 봉투다. 실패 경로에서 침묵하지 않는다.

## 픽스처

- `fixtures/validate/pass.json`
- `fixtures/validate/placeholder.json`
- `fixtures/validate/unknown_evidence.json`
- `fixtures/validate/unlinked.json`
- `fixtures/validate/mixed_violations.json`

예제: 05·06·07·08·14.

다음: [06_coordinate_rules.md](06_coordinate_rules.md).
