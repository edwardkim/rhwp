# 19 실패 문서 대장

전수성의 실무 장. Phase A 와 SWS L2 가 이 기록을 본다.

## 기록 필드

| 필드 | 출처 |
| --- | --- |
| file | corpus 상대 경로 |
| sizeBytes | stat |
| status | `failed` |
| infoExit | info 종료 코드 또는 null(타임아웃) |
| reason | 엔진이 추정하지 않음. 에이전트가 회신에 해석을 붙일 수는 있다 |

엔진은 reason 문장을 지어내지 않는다. `infoExit` 만 남긴다. 에이전트가
"암호 보호로 보임"이라고 쓰는 것은 회신 해석이지 지도 필드가 아니다.
지도 JSON 을 고쳐 이유를 삽입하지 않는다.

## 이후 단계

failed 문서도 search 루프에 들어간다. search 가 다시 실패하면
`evidence.json.failures` 에 phase=search 행이 추가된다. 지도에서 빼지
않았으므로 실패가 두 층에 남는다. 이것이 정상이다.

search 가 우연히 성공해도(일부 손상 파일) 지도의 status=failed 는
그대로다. info 실패와 search 성공을 합쳐 status 를 ok 로 승격하지 않는다.

## 전부 실패

모든 문서가 info 실패여도 지도는 남는다. 이어지는 search 가 전부
실패하면 엔진은 대장을 만들지 않고 exit 1. 이 경우 산출은
corpus_map.json 까지다. 빈 evidence.json 을 손으로 만들지 않는다.

## 고객 요청 "읽을 수 있는 것만"

읽을 수 있는 파일만의 **별도** engagement 를 새로 만들 수는 있다.
그때는 objective 에 "가독 부분집합"임을 명시하고, 원 코퍼스의 failed
목록을 1부에 인용한다. 원 지도를 고쳐서 성공처럼 보이게 만들지 않는다.

예제: E03, E15, E24.

다음: [20_question_design.md](20_question_design.md).
