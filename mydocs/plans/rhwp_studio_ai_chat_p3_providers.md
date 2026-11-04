# 구현 계획 — rhwp-studio AI 도우미 P3 (OpenAI·Gemini 제공자)

- 작성일: 2026-10-26 KST
- 상태: 구현·단위·E2E 검증 완료. 실제 키 확인은 작업지시자 키로 별도 수행
- upstream 이슈: #8264
- base: `devel` `4e690c930`
- 선행: [P0 구현 계획](rhwp_studio_ai_chat_p0_impl.md) — #8170(#8228로 통합)의 제공자 중립 경계

## 1. 목표와 범위

P0가 확정한 제공자 중립 인터페이스(`AiProvider.streamTurn`, `AiMessage`, `AiFailure`)에 OpenAI와
Gemini adapter를 붙인다. 대화 엔진·도구 41개·변경 카드·키 보관은 바꾸지 않는다.

| 항목 | 내용 |
|---|---|
| 제공자 | Claude(기존), OpenAI(Responses API), Gemini(Gemini API `streamGenerateContent`) |
| 모델 | OpenAI `gpt-6-astra`·`gpt-6.1-sol`·`gpt-6-luna`, Gemini `gemini-4-argon`·`gemini-3.8-flash`·`gemini-3.5-flash-lite` |
| 응답 깊이 | low·medium·high → OpenAI `reasoning.effort`, Gemini `thinkingConfig.thinkingLevel` |
| 연결 테스트 | 모델 정보 조회(GET) 한 번. 토큰을 쓰지 않는다 |
| 키 | 제공자마다 따로 보관(기존 `KeyStore` 의 provider 키). 헤더로만 보낸다 |

비범위: 서버 프록시, 모델 목록의 동적 조회, OpenAI strict 도구, Gemini 명시적 캐시(`cachedContents`).

## 2. 확정한 결정

### 2.1 SDK 대신 `fetch` + SSE

두 API 모두 브라우저 CORS 요청을 받는다. 벤더 SDK를 들이지 않고 `provider/http.ts`의 SSE 읽기와
오류 갈래 부품을 공유한다. 번들이 커지지 않고, 요청 본문을 바이트 단위로 고정해 단위 시험할 수 있다.
adapter는 `provider/registry.ts`가 고른 제공자의 것만 처음 질문할 때 동적 import 한다.

### 2.2 이력 되돌리기 — 제공자 원형을 고치지 않는다

P0 계약(`assistant`/`raw` 는 고치지 않고 다음 요청에 되돌린다)을 그대로 따른다.

- OpenAI: `store: false`(서버에 대화를 남기지 않음). 응답 `output` 항목(reasoning·message·function_call)을
  다음 `input`에 그대로 넣는다. 무상태 모드의 reasoning 항목은 `encrypted_content`를 싣고 오므로 도구
  사이에서도 추론이 이어진다. 도구 결과는 `function_call_output`(`call_id`)이다.
- Gemini: 응답 part의 `thoughtSignature`를 고치지 않고 되돌린다(빠뜨리면 도구 호출 뒤 요청이 거절된다).
  스트림 조각은 같은 종류의 글 part끼리, 앞 part에 아직 서명이 없을 때만 잇는다. 도구 결과는
  `functionResponse`이고 이름으로 짝을 짓는다. 응답에 호출 id가 없으면 순서로 `gemini-call-N`을 만들고,
  결과를 보낼 때 직전 응답 part에서 같은 규칙으로 다시 계산한다.

### 2.3 도구 schema

- OpenAI: `strict: false`. strict는 모든 속성을 required로 요구해 선택 매개변수가 많은 쓰기 도구와
  맞지 않는다. 입력 제약은 기존 실행 전 클라이언트 검증(`schema-validate.ts`)이 지킨다.
- Gemini: `parameters`(OpenAPI 부분집합)로 옮긴다(`toGeminiSchema`). `additionalProperties` 등 받지 않는
  키는 빼고, 타입 배열은 `nullable`, `const`는 `enum`으로 바꾼다. 속성이 빈 OBJECT는 거절되므로
  매개변수 없는 도구는 `parameters`를 뺀다. 41개 도구 전부를 단위 시험이 허용 키 목록으로 확인한다.

### 2.4 끝 사유와 오류 갈래

| 중립 값 | OpenAI | Gemini |
|---|---|---|
| `tool_use` | `function_call` 항목이 있음 | `functionCall` part가 있음 |
| `max_tokens` | `incomplete` + `max_output_tokens` | `MAX_TOKENS` |
| `refusal` | `incomplete` + `content_filter`, `refusal` 내용 | `SAFETY`·`RECITATION`·`PROHIBITED_CONTENT` 등, `promptFeedback.blockReason` |
| `billing` | `insufficient_quota`·`credit_balance_exhausted`·`*_spend_limit_exceeded` 등 | 402 |
| `auth` | 401 | 401, 400 `API_KEY_INVALID` |
| `rate` | 429 + `retry-after` | 429 + `RetryInfo.retryDelay` |

크레딧 부족 안내는 제공자별 문구(`ai.error.billing.openai`·`.gemini`)를 쓴다.

### 2.5 제공자를 바꾸면 새 대화

이력의 assistant 칸은 제공자 원형이라 다른 제공자에 보낼 수 없다. `ChatController`가 이력을 만든
제공자를 기억하고, 설정에서 제공자가 바뀌면 대화를 새로 시작한다(`reset('provider')`, 안내
「AI 제공자가 바뀌어 새 대화를 시작했습니다」). 전송 동의는 기존대로 제공자마다 다시 묻는다.

### 2.6 사용량

세 제공자의 `inputTokens`가 같은 뜻(캐시에서 읽지 않은 입력)이 되도록 OpenAI `input_tokens`,
Gemini `promptTokenCount`에서 캐시 토큰을 뺀다. Gemini 출력은 `candidatesTokenCount + thoughtsTokenCount`이다.

## 3. 근거 문서 (2026-10 확인)

- OpenAI: developers.openai.com/api/docs/models, guides/function-calling, guides/reasoning,
  guides/streaming-responses, guides/error-codes
- Gemini: ai.google.dev/gemini-api/docs/models, docs/thinking, docs/function-calling, api/generate-content,
  api/models

## 4. 검증

- 단위: `tests/ai-openai-provider.test.ts`, `tests/ai-gemini-provider.test.ts`, `tests/ai-provider-switch.test.ts`
  — 요청 본문 바이트 고정, 원형 되돌리기, schema 부분집합(41개 도구), 끝 사유·오류 갈래, 스트림 조각 잇기,
  키가 URL에 없음, 제공자 전환 시 새 대화.
- E2E: `npm run e2e:ai-providers` — 요청 가로채기로 두 API를 흉내 내 설정·연결 테스트·동의·도구 한 번·
  답 표시·키 오류 문구와 제공자 전환을 실제 studio에서 확인한다.
- 실제 키 확인: 미실행. 작업지시자 키로 실제 브라우저에서 질문·편집 한 번씩 확인한다.
