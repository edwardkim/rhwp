# task_m100_3357 처리결과 보고서 — capabilities 의 feature 가용성 자기서술

- **이슈**: [#3357](https://github.com/edwardkim/rhwp/issues/3357) (3안 조각)
- **브랜치**: `pr/fix-issue-3357-capabilities-feature-truth` (upstream/devel `4a39f7cc0` 직분기)
- **범위**: `src/main.rs`(capabilities 의 export-png 항목 + cmd_gated 헬퍼),
  `tests/issue_3357_capabilities_feature_gate.rs`(신규), `mydocs/manual/cli_commands.md`(1항목)
- **분류**: 버그 수정 (자기서술의 계약 — 광고와 실체의 불일치)

## 1. 배경

공식 릴리스 바이너리는 native-skia feature 없이 빌드되어 `export-png` 가 실행되지
않는데(#3357 실측: exit 2), `capabilities` 는 이를 조건 없이 등재한다. 매니페스트만
보고 호출을 생성하는 에이전트는 **호출해 보고서야** 기능 부재를 알게 된다 — #3347 이
잡은 "광고만 하고 구동할 수 없는 도구" 결함과 같은 부류다.

## 2. 설계 결정

- **필드는 항상, 값만 빌드별** — `requiresFeature:"native-skia"`·`available:bool` 을
  빌드와 무관하게 항상 방출한다(스키마 안정성). 값은 `cfg!(feature)` 실측이다.
- **일치성까지 테스트로 고정** — `available` 이 실제 호출 결과와 어긋나면(기능 부재
  오류의 발생 여부로 판정) 테스트가 실패한다. native-skia 유무 **어느 빌드에서 돌려도
  통과**하도록 실측 대조로 작성했다 (CI 의 native-skia-tests 잡에서도 유효).
- **게이트 없는 명령은 무변경** — 두 필드는 export-png 에만 붙는다(의미 오염 방지 테스트
  포함). MCP 매니페스트(`--mcp`)는 export-png 를 애초에 광고하지 않아 무변경.
- 릴리스에 feature 를 포함하는 1안(release-binary.yml)은 러너 요건·크기 판단이 필요해
  이슈에 남긴 그대로 메인테이너 결정 사항으로 분리.

## 3. 검증

- **계약 테스트 3종** (red→green): 두 필드 상시 방출 / **available ↔ 실제 호출 일치**
  (빌드 무관 판정식) / 비게이트 명령(info·export-text·export-svg)에 필드 부재
- 무회귀: `cli_json_contract`(capabilities 계약 포함) 전부 green
- `cargo fmt` clean, clippy `-D warnings` 0건 (release-test 프로필)
- **전/후 스크린샷**: `assets/task_m100_3357/before.png·after.png` — 에이전트 관점
  (전: 매니페스트 믿고 호출 → exit 2 지뢰 / 후: 매니페스트에서 사전 판별 → export-svg 로 우회)

## 4. 남긴 것

- `export-pdf --backend direct` 도 같은 feature 게이트다 — 명령 전체가 아니라 플래그
  단위 게이트라 표현 방식(예: `gatedFlags`)이 별도 논의 대상. 후속 조각 후보.
- 릴리스 바이너리에 feature 를 포함할지(1안)는 #3357 에서 계속.
