# task_m100_3287 처리결과 보고서 — `export-svg --json` 산출물 매니페스트

- **이슈**: [#3287](https://github.com/edwardkim/rhwp/issues/3287)
- **브랜치**: `pr/task-render-json` (**upstream/devel 직분기 — 열린 PR 과 공유 커밋 없음**)
- **범위**: `src/main.rs`(export_svg), `tests/render_manifest_json_contract.rs`(신규),
  `mydocs/manual/cli_commands.md`
- **분류**: 기능 추가 (출력 계약)

## 1. 문제

렌더 명령은 **파일을 만드는** 명령인데 무엇이 만들어졌는지가 사람용 텍스트로만 나왔다.
에이전트가 다음 단계(VLM 조판 확인)로 가려면 "어느 페이지 → 어느 파일"을 알아야 하는데,
진행 메시지를 정규식으로 긁는 수밖에 없었고 파일명 규칙이 바뀌면 조용히 깨졌다.

#3140 의 `render_page` 도구가 목표한 "에이전트가 스스로 조판 확인" 레시피의 마지막 고리다.

## 2. 분석 — 설계 결정

- **렌더는 건드리지 않는다.** stdout 을 데이터로 바꾸는 것뿐이다.
- **stdout 순수성**: `--json` 에서는 "문서 로드 완료"·"→ 경로" 진행 메시지를 내지 않는다.
  (초기 구현에서 이 줄이 새어 JSON 파싱이 깨졌고, 계약 테스트가 잡았다.)
- **경로 실재성을 계약에 넣었다.** 매니페스트의 `path` 는 에이전트가 바로 읽는 값이므로,
  테스트가 실제 파일 존재를 단언한다 — 문자열만 맞고 파일이 없으면 무의미하다.
- **기본 출력 무변경 가드**를 걸어 기존 소비자를 보호했다.
- `export-png` 는 `native-skia` feature 게이트라 테스트 환경이 갈리므로 이번 범위에서 제외하고,
  같은 스키마로 후속 확장하는 것이 안전하다.

## 3. 변경

- `export_svg` 에 `--json` 파싱·매니페스트 수집·봉투 방출, 진행 메시지 게이팅
- `cli_commands.md` 갱신 (search 조합 레시피 명시)

## 4. 검증

- **계약 테스트 3종 red→green**: 봉투 스키마·페이지 정합·**매니페스트 경로 실재** /
  기본 출력 무변경 가드 / 없는 파일 exit 1 + stdout 0바이트
- `cargo clippy --release --bin rhwp -- -D warnings` 0, `rustfmt` clean, 문서 검사 2종 clean
- 실측: `-p 3 --json` → `{"page":3,"path":"…hwp3-sample_004.svg","bytes":727767}`,
  파일 실재 확인

## 5. 남긴 것

- `export-png --json`(같은 스키마)은 feature 게이트 환경 확인 후 후속.
- `export-pdf --json` 도 같은 패턴으로 확장 가능하나, 단일 파일 산출이라 값어치가 작다.
