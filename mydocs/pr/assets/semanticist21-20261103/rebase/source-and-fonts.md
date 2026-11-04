# 렌더링 source와 글꼴 공급의 동일성

검증 후보 `696240b004707a6686864f94130befe3dd7f213e`, Native 전쪽/편람 producer `ef6d3029371262204f7b5c3ecf44820c66e54345`다. 두 commit을 `git diff --name-only`로 대조해 다음 차이만 확인했다.

- `.claude/skills/rhwp-provenance/SKILL.md`
- `.claude/skills/rhwp-provenance/fixtures/command-untrusted-fields.json`
- `.claude/skills/rhwp-provenance/fixtures/injection-boundaries.json`
- `.claude/skills/rhwp-provenance/references/anti-patterns.md`
- `.claude/skills/rhwp-provenance/references/command-field-catalog.md`
- `.claude/skills/rhwp-provenance/references/export-provenance-map.md`
- `.claude/skills/rhwp-provenance/references/untrusted-content-fields.md`
- `crates/rhwp-contracts/src/provenance.rs`
- `mydocs/pr/assets/semanticist21-20261103/ci-provenance-followup.md`
- `mydocs/tech/envelope_provenance.md`

실행 코드 차이는 `crates/rhwp-contracts/src/provenance.rs`의 JSON 문서 출처 선언뿐이다. renderer·model·DocumentCore·paint·parser·Native/Canvas backend·글꼴 선택/메트릭·Cargo·asset·WASM 바인딩·검사 assertion은 같다. 소비 fixture와 문서는 표지 지도를 동기화했다. 따라서 ef6 Native 전쪽/편람 출력은 같은 렌더링 source의 재사용으로 표시하고, 새 후보가 재실행한 것으로 표시하지 않는다. 새 Native 실제 API 캡처와 최적화 fresh WASM은 696 후보에서 별도로 실행했다. 제출 증적과 상대 링크만 정리한 후속 로컬 commit도 제품 source 동일성을 확인한다.

## 원본 휴먼명조

- 원본: rowbreak charPr 26 / hangul font 8 / TTF 휴먼명조.
- 실제 명시 공급: `/home/edward/mygithub/ttfs/hwp/HMKMM.TTF`.
- SHA-256: `d033df71fa41de48905407f7d39aca74ebd9ae87298c5420513acaa187fbefc6`.
- 주 측정: 이 파일을 포함한 `/home/edward/mygithub/ttfs/hwp`와 공통 글꼴 경로를 Native/fresh WASM 양쪽에 공급했다.
- 이 파일과 HFT 변환본은 로컬 검증용이며 저장소나 배포물에 넣지 않는다. 소스 이름이 다른 TTC로 대신하지 않았다.

기존 [글꼴 공급 기록](../fonts.md)의 Form01·공통 글꼴 조건과 함께 읽는다. 최소 글꼴 회귀의 허용된 저장소 글꼴 5개와 독립 Print 시각 측정의 명시 공급을 혼동하지 않는다.
