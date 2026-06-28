# PR #1619/#1621/#1625/#1628 통합 처리 계획

## 대상

- 원 PR: #1619, #1621, #1625, #1628
- 작성자: planet6897
- 통합 브랜치: `integrate/planet6897-1619-1628`
- base: `devel`
- 관련 이슈: #1618, #1620, #1624, #1627

## Stage 1. 통합 브랜치 준비

- `upstream/devel` 최신 상태에서 `integrate/planet6897-1619-1628` 브랜치를 생성했다.
- 원 PR head commit 4개를 PR 번호 순서대로 cherry-pick 했다.
- cherry-pick 충돌은 없었다.
- 원 작성자 `Jaeook Ryu <jaeook.ryu@gmail.com>` author 정보는 보존됐다.

## Stage 2. maintainer 보정

`cargo clippy --all-targets -- -D warnings` 에서 #1628 단위테스트의 `Box::new(Table::default())` 가 `clippy::box_default` 경고를 냈다.

보정 commit:

- `a9575dad1` — `fix: task 1627 테스트 clippy 경고 정리`

변경 범위는 테스트 생성 코드 한 줄과 불필요 import 제거뿐이다.

## Stage 3. 로컬 검증

완료한 검증:

- `git diff --check upstream/devel...HEAD`
- `cargo fmt --check`
- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --doc`
- `wasm-pack build --target web --out-dir pkg`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `cargo test --test svg_snapshot`

비고:

- 첫 번째 clippy 는 `box_default` 경고로 실패했고 maintainer 보정 후 통과했다.
- 한 차례 clippy 재실행이 cargo idle 상태로 멈춰 중단 후 같은 명령을 다시 실행했고, 최종 재실행은 통과했다.
- `wasm-pack build` 는 플랫폼용 prebuilt `wasm-bindgen` 미제공으로 cargo install fallback 경고가 있었으나 빌드는 완료됐다.

## Stage 4. PR 준비

통합 PR:

```text
#1631 planet6897 PR #1619/#1621/#1625/#1628 통합 반영
```

PR 본문 포함 항목:

- 원 PR: #1619, #1621, #1625, #1628
- 변경 요약: #1618 vpos 분석, #1620 field removal panic 방지, #1624 footer over-push 정밀화, #1627 bookmark in-order serializer 보존
- maintainer 보정: #1628 테스트 clippy 경고 정리
- 검증 결과 요약
- `Closes #1618`
- `Closes #1620`
- `Closes #1624`
- `Closes #1627`

원격 CI 확인:

- `Build & Test`: 통과, 18m17s
- `CodeQL`: 통과
- `Render Diff`: 통과
- `Canvas visual diff`: 통과
- merge state: `MERGEABLE` / `CLEAN`
- merge commit: `c0c12d5020c9ead8f1cc7309f58522cd82e8d5a7`

## Stage 5. merge 후 후속 처리

- 통합 PR #1631 merge 후 #1618/#1620/#1624/#1627 자동 close 여부를 확인했다.
- 자동 close 가 걸리지 않아 네 이슈 모두 수동 close/comment 처리했다.
- 원 PR #1619/#1621/#1625/#1628에는 통합 PR로 반영했다는 코멘트를 남기고 close 했다.
- `devel` 동기화 후 로컬 통합 브랜치 `integrate/planet6897-1619-1628` 를 제거했다.
- 오늘할일과 처리 보고서를 문서 전용 후속 PR 로 정리한다.

## 보류/주의

- 원 PR 4개는 각각 `BEHIND` 상태였으므로 개별 PR 자체를 merge 하지 않고 #1631로 통합 반영했다.
- #1628 본문에 언급된 char_shape 오프셋 +8 문제는 본 통합 범위 밖이다.
- #1619는 `-1쪽` 시리즈 분석 종결 성격이고, #1625는 같은 페이지네이션 계열 보정이다. #1621/#1628은 독립 결함 수정이다.
