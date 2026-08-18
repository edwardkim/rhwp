> **PR base 브랜치가 `devel` 인지 확인해주세요** (`main` 아님 — GitHub 기본 선택이 main 일 수 있습니다).
> 작업 브랜치는 최신 `upstream/devel` 에서 생성합니다.

## 변경 요약

기준 풀이 조립기(`gym/tools/build_baseline.py`)의 자리표 치환·부재 산출·실패 보고를 고도화한다. 새 CLI 플래그와 새 pack 은 없다.

- 한 문자열의 `{sub:}` 를 전부 바꾼다. 세 개 이상·`{input}` 혼합·닫히지 않은 `{sub:` 도 시험으로 고정한다.
- `{sub:}` 이름은 제출 폴더 안의 상대경로만 허용한다. 부모·절대·드라이브·UNC·홈은 `RuntimeError` 로 그 과제만 실패한다.
- `submit.files` 가 선언한 파일이 없으면 채점 전에 `부재 산출:` 로 보고한다. `score_task` 를 부르지 않는다.
- 채점 실패는 `pack/task: 이유` 한 줄을 유지한다. 검사 이름을 남기고, 비-dict·`pass` 키 없음은 통과로 접지 않는다.
- CLI 는 `--agent` / `--pack` / `--bin` 만. `resolve` · `build_task` · `verify_built_task` 서명은 그대로다.

## 관련 이슈

closes #5273

## 테스트

- [x] `python -m unittest scripts.tests.test_gym_build_baseline -q` 통과 (128 ran, 0 failed)
- [x] `python -m unittest scripts.tests.test_gym_packs -q` 통과 (18 ran, 0 failed; `BaselineResolveTests` 기존 3칸 + 자리표 3개·부재 산출·검사 이름)
- [x] `python gym/tools/audit.py` 통과 (18 pack, 위반 0)
- [x] **`cargo fmt --all -- --check` 통과**
- [ ] `node scripts/rust-test-suite-manifest.mjs --check` 통과 (해당 없음)
- [ ] `node scripts/rust-unit-test-tiers.mjs --check` 통과 (해당 없음)
- [ ] `cargo test` 통과 (해당 없음 — Python/문서만)
- [ ] `cargo clippy -- -D warnings` 통과 (해당 없음)
- [ ] 관련 샘플 파일로 SVG 내보내기 확인 (해당 없음)
- [ ] 웹(WASM) 렌더링 확인 (해당 없음)
- [ ] rhwp-studio 편집·UI 변경 시: e2e 시나리오 또는 편집 커맨드 리뷰 체크리스트 통과 (해당 없음)

## 성능 영향 및 측정 결과 (해당하는 경우)

- 예상 영향: 영향 없음
- 재현·측정: 단위 시험은 바이너리 없이 돈다. 라이브 왕복은 기존과 같이 rhwp 가 필요하다.

## 스크린샷

해당 없음.
