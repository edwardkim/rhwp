# PR #8421 메인터너 보정 계획

사용자 지시: 메인터너 쪽에서 처리해 머지한다. 원 기여 커밋 9개는 보존한다.

- 기준 devel: `0b697f90c3637558196b9e3b81c4eb59049432af`
- 원 head: `d8c2fc3e1a04a006e971219dd3fb0b175ee96987`
- 경로: maintainer_general + intake_and_review + local_validation + post_merge.
- 기존 CHANGES_REQUESTED의 P2를 해결한다. Picture writer 구현과 ingest 빌더의 placeholder 처리를 구분한다.
- 그림 필수 입력은 F18로 중단하고 경로·배치 변경만으로 재시도하지 않는다. 수식·표 이미지도 같은 제한이다.
- SKILL, 관련 안내·예제·fixture와 생성기 문구를 맞춘다. Rust·helper 실행 로직·스키마 형식은 변경하지 않는다.
- YAML/router/catalog, 기존 스킬 계약, 생성기 문자열 외 AST 및 격리 재생성 일치, JSON·링크·공백을 검증한다.
- 최종 source head를 최신 devel과 병합한 tree가 로컬 검증 tree와 같음을 확인한다. 새 head CI·self-review 뒤 원 PR을 normal merge한다.
- 조판 소비 경로에 변화가 없으므로 Rust 전체 빌드·Native/fresh WASM·Visual Sweep은 비해당이다.
- merge 뒤 확정 결과는 maintainer 운영 기록으로 archive·오늘할일에 남기고 검토 worktree·임시 output을 정리한다.
