# task_m100_4407 stage1 — 합침의 시대 정세 문서 + 작업 계보 DAG 확장 설계서

- 이슈: #4407 ([#3907 지평] — 3년 축 #4401/PR #4406 의 다음 지평)
- 브랜치: task_m100_4407 (base: devel — 코드 0줄, 문서 1건이라 적층 불필요)

## 무엇을 만들었나

`mydocs/tech/agent_roadmap/trend_merge_dag_2026h2.md` — "LLM 두 개(N개)가
합쳐지는 게 올까?"라는 질문에서 출발해, (전반) 모델 합침의 3층 지형 전수 정리와
(후반) 그 세계가 요구하는 검증 인프라 — 작업 계보의 합류(DAG) 확장 — 설계서.

- §2–4 정세 3층: 가중치 병합(soup→task arithmetic→TIES/DARE→진화적 병합, 성립
  직관과 한계까지) · MoE 업사이클링 · 시스템 수준(라우팅·증류·MoA).
- §5 시간축 전망 — 확신도(높음/중간/낮음) 열로 판단과 실측을 구분.
- §6 검증 공백 논증 — 모델 카드·BOM의 진술, C2PA의 서명·자산 결속, 레시피
  재현성을 서로 다른 보장으로 구분하고, 남은 재현 공백과 문서 실증장의 범위를 명시.
- §7 DAG 설계 — `parents[]` v1.1 스키마(role + edge binding), receipt
  `inputs[]`/`outputs[]`, v1.0 봉투 하위호환, 모든 parent output↔child input slot
  결속, canonical 실파일 방문 키 BFS, 실행 입력·도구 프로필까지 포함한 deep cache key.
- §8 위협 모델 T1–T7 — 역사 전체 재작성(T7)은 해시 체인만으로 못 잡는다고
  명시하고, 계획 밖 비밀 입력과 별도 복사본의 논리 identity도 자동 검출 범위 밖으로
  분리(외부 앵커는 후속 축).
- §9 단계 M1–M4 + DoD, M4(모델 레시피 캡슐 동형 검증)의 논지.

## 서지 규약 (정직 조항)

arXiv 식별자는 원문 초록 대조로 검증한 4건(2203.05482 · 2311.03099 · 2403.13187 ·
2406.04692)만 링크로 적고, 나머지는 저자·연도·성과 수준으로만 인용했다. 전량
원문 링크 검증은 M1 DoD 에 포함.

## 검증

- scripts/check_markdown_links.py · scripts/check_document_metadata.py 통과
- 교차 참조는 전부 이슈/PR URL — 미머지 문서 상대링크 금지 규약 준수
- `track_*.md` 집계 밖 `trend_` 접두어 — roadmap_progress 집계 불변

## 메인터너 검토 보정

- 동일 plan text라도 입력 digest가 다르면 재실행 결과를 공유할 수 없어
  `planSha256` 단독 cache를 실행 키로 교체했다.
- material parent의 capsule 무결성만으로는 그 산출물이 자식 재료였음을 증명하지
  못하므로 모든 edge에 parent output↔child input slot digest 결속을 요구했다.
- 같은 capsule bytes를 다른 폴더에 복사하면 상대 parent 의미가 달라질 수 있어
  file hash 단독 방문 키를 canonical 실파일 기준으로 교체했다.
- 4노드라고 쓰고 5개 작업을 세던 합본 예시를 분할 s + 편집 a·b·c + 합본 d의
  5노드로 바로잡고, 현행 run plan에는 다중 input/재료 step이 없음을 명시했다.
- C2PA는 단순 자기 신고가 아니라 manifest와 자산을 암호학적으로 결속하므로,
  강점을 인정하면서 레시피 재실행과는 다른 보장이라고 범위를 바로잡았다.
