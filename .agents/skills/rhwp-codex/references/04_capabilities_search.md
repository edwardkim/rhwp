# capabilities --search 폴백

판단 트리의 일곱 갈래에 안 들어가면 자기서술을 검색한다.

```bash
rhwp capabilities --search 누름틀
rhwp capabilities --search 영수증
rhwp capabilities --search 은닉
```

## 규칙

1. `--search` 는 `capabilities` 의 검색 모드다. 새 명령이 아니다.
2. 히트의 `name` 으로 장 번호를 얻는다 (이 스킬의 `fixtures/search_fallback.json`).
3. 0건이면 **표면 밖**(X03). 없는 하위명령을 지어내지 않는다.
4. 85장으로만 떨어지면 개발자 전용임을 고지하고 통상 작업에서는 거절한다.
5. `capabilities` 단독 호출은 전체 JSON 자기서술이다. `--json` 은 `--search` 전용 맥락.

## 왜 폴백인가

스킬과 판단 트리는 흔한 요청만 닫는다. CLI 는 진화한다.
골든 파일로 명령 집합을 박제하지 말고, 바이너리 자기서술에 물어라.
스킬 표류 가드가 같은 원칙이다.

## 검색 키워드 카탈로그

`fixtures/search_fallback.json` 의 각 항목은 실제 인자 배열
`["capabilities", "--search", "<키워드>"]` 이다.
