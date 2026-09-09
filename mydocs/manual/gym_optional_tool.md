---
kind: guide
status: active
canonical: mydocs/manual/gym_benchmark_operations.md
last_verified: 2026-09-09
---

# Gym — 선택적 평가 도구

제품 실행에는 Gym 설치가 필요하지 않습니다.
Gym은 에이전트가 rhwp CLI/API를 활용하는 능력을 평가하는 별도 도구입니다.
의존 방향은 **Gym → rhwp CLI/API**이며, rhwp 제품은 Gym 없이 빌드·실행·배포할 수 있어야 합니다.

평가를 원할 때만 Gym이 포함된 저장소를 별도로 준비하고, 사용할 rhwp 바이너리를 지정합니다.
Gym 점수는 한컴 조판 동등성이나 제품 릴리스 적합성의 독립 정답지가 아닙니다.

- [Gym 참가자 안내](https://github.com/edwardkim/rhwp/blob/devel/gym/README.md)
- [메인테이너용 Gym 운영 매뉴얼](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/gym_benchmark_operations.md)

이 안내는 제품이 소유하며 MCP `rhwp://docs/gym`에서 오프라인으로 읽을 수 있습니다.
Gym README 전체를 제공하거나 채점기를 실행하지 않으며, 링크를 자동으로 내려받지 않습니다.
링크의 문서는 최신 devel 기준이므로 평가할 때는 사용할 제품과 Gym의 버전을 따로 고정합니다.
