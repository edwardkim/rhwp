# #5737 Stage 1: nextest archive A/B partition

- 단일 17분 27초 archive와 전용 slow shard를 A/B 병렬 archive와 4 worker로 교체한다.
- A는 짝수 숫자 suffix binary, B는 보집합이다. 두 filterset은 겹치지 않고 전체 target을 덮는다.
- 각 archive는 `hash:1/2` 두 worker가 소비하며 aggregate가 archive별 count 합을 검증한다.
- 후보 CI에서 critical path, builder 시간, artifact 크기, 총 runner minute을 기존 단일 archive와 비교한다.
  공통 dependency 중복 compile으로 이득이 없으면 채택하지 않는다.
