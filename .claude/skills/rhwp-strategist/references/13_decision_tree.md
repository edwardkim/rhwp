# 13 판단 트리 — 요청에서 단계로

```
요청
├─ 증상(깨짐/잘림/안 열림) → rhwp-fde 로 인계. 여기서 멈추지 말고 보내지 말 것: 전략 골격
├─ 큐 운영(오늘 할 일, 우선순위) → rhwp-chief
├─ 전망만 ("시장이 어떻게 될까", 코퍼스 없음) → ST-FORECAST 거부
└─ 목표 + 문서 더미
   ├─ engagement.json 없음 → 질문 설계(20장) 후 작성
   ├─ 엔진 미실행 → engagement.py
   │    ├─ exit 2 → 입력 수정
   │    ├─ exit 1 → capabilities/bin/search 전패 점검
   │    └─ exit 0 → 지도·대장 수치 확인
   │         ├─ failed 문서 있음 → 회신에 남김. 삭제 금지
   │         ├─ truncated 있음 → 한도/키워드 재검토
   │         └─ 대장 읽기
   │              ├─ CLAIM 작성 (EV 동거)
   │              ├─ --validate
   │              │    ├─ exit 3 → 위반 kind 별 수정 → 재검증
   │              │    └─ exit 0 → 납품 (scaffold 광고 시에만 hwpx)
   │              └─ 0건 절 → 공란 유지
```

## 30초 질문

1. 코퍼스 경로가 있는가.
2. 목표가 검증 가능한 질문으로 쪼개지는가.
3. 엔진을 이미 돌렸는가.
4. 실패 문서·절단·0건을 정직하게 봤는가.
5. validate 가 pass 인가.

하나라도 아니면 납품하지 않는다.

다음: [14_recipe_index.md](14_recipe_index.md).
