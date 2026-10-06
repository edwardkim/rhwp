# Task M100 #6112 Stage 3 사후 재구성 — 전체·브라우저 검증

- **이슈**: [#6112](https://github.com/edwardkim/rhwp/issues/6112)
- **일자**: 2026-08-26 KST
- **기능 commit**: `d707d4cf2b4efef757420381b6a460f1b325482e`
- **최종 동기화 기준**: `upstream/devel` `6b5c4f871972380c0866e2a8d27ac2bc67d257e6`
- **검증 대상 merge commit**: `93dc5773f`
- **문서 성격**: 작업 뒤 실행 로그를 대사한 감사 증거

## 단위 회귀

```text
cd rhwp-studio && npm test
tests 1138
pass 1137
fail 0
skipped 1
```

최종 소스·테스트 정리 뒤 같은 전체 테스트를 다시 실행해 같은 결과를 확인했다.

## 타입·프로덕션 빌드

기존 작업공간의 생성된 `pkg/rhwp.js`, 타입 선언과 WASM 산출물을 임시 재사용해 다음을 실행했다.

```text
cd rhwp-studio && npm run build
tsc 통과
Vite production build 통과
```

CanvasKit의 브라우저 호환 externalize 및 기존 chunk size 경고는 있었지만 빌드 실패는 없었다. 검증에
사용한 임시 복사본은 소스 diff에서 제거했다.

## 실제 브라우저 E2E

헤드리스 Chrome에서 다음 사용자 여정을 수행했다.

1. `view.toolbar*` 저장 키가 없는 상태로 시작
2. 기본 도구 상자만 접힘, 서식 도구 상자는 보임 확인
3. 메뉴와 우측 버튼 상태·접근 가능한 이름 확인
4. 실제 포인터 클릭으로 우측 버튼을 눌러 기본 도구 상자 펼침
5. 버튼 포커스 상태에서 실제 `Ctrl+F1` 키 입력으로 다시 접음
6. 저장값을 유지한 채 리로드해 숨김 복원과 초기 프레임을 측정

초기 구현 검증은 리로드 첫 35프레임에서 숨긴 도구 상자가 보인 프레임 `0/35`였다. 최신 devel
병합 뒤 같은 E2E를 다시 실행했고 모든 단언 통과, `0/37` visible frames를 확인했다.

## 정적 점검

- `git diff --check`: 통과
- `cargo fmt --all`, `cargo fmt --all -- --check`: 통과
- `python3 scripts/check_markdown_links.py`: 603개 문서, 상대 링크 이상 없음
- 별도 작업트리 상태: 기능·테스트 파일만 변경 후 커밋, 임시 `pkg/` 제거 확인
- 원래 작업트리의 사용자 변경: 미수정

최초 `cargo fmt --all`은 review 전용 `tests/generated/regression_suite_*`가 없는 상태라 포맷 전에
중단됐다. `node scripts/rust-test-suite-manifest.mjs --prepare`로 파생 suite를 준비한 뒤 두 fmt 명령을
재실행해 통과했다. 생성 suite는 검증 산출물이며 PR source에 포함하지 않는다.

## 시각 검증 판정

Studio chrome UI 변경이며 문서 렌더러·레이아웃·paint 경로는 변경하지 않는다. 따라서 PDF/SVG visual
sweep 대신 실제 브라우저의 버튼 클릭 가능성, 펼침·접힘 display, 접근성 상태와 첫 페인트 프레임을
검증 근거로 사용했다.

## 잔여 게이트

최신 `upstream/devel` 동기화와 로컬 필수 검증은 완료했다. 원격 branch push와 PR 생성 뒤 GitHub CI는
PR head에서 별도로 판정한다.
