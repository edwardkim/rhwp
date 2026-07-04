# PR #1454 리뷰 기록 — rhwp-studio 로컬 글꼴 감지 동의 흐름

- PR: https://github.com/edwardkim/rhwp/pull/1454
- 작성일: 2026-06-21
- 작성자: collaborator self-merge 후보 경로
- 작성 시점 head: `00f462dbf3dbe8da5dfb225267519616315b42b9`
- base: `devel`
- head: `postmelee:1328-local-font-consent`

## 1. PR 메타

| 항목 | 확인 내용 |
|------|-----------|
| 작성자 | `postmelee` |
| PR 상태 | open, draft 아님 |
| mergeable | 작성 시점 `MERGEABLE` / `CLEAN` |
| 관련 이슈 | `Refs #1328` |
| 이슈 자동 close | 없음. 작업지시자 승인 전까지 `Closes`를 사용하지 않음 |
| 커밋 수 | 7개 + 본 self-merge review 문서 커밋 예정 |

`mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 이 문서는 작성 시점 값을 참고로만 기록한다.
최종 merge 판단은 merge 직전 최신 PR head 기준으로 다시 확인한다.

## 2. 변경 범위

### 2.1 기능 변경

- rhwp-studio 문서 로드 후 기본 지원/웹 대체 밖 글꼴이 있으면 로컬 글꼴 감지 동의 모달을 표시한다.
- 사용자 승인 전에는 로컬 글꼴 목록을 조회하지 않는다.
- Chrome/Edge에서는 사용자 승인 후 Local Font Access API(`queryLocalFonts`)를 사용한다.
- Firefox에서는 전체 목록 API가 없으므로 현재 문서 후보 글꼴만 canvas presence probe로 확인한다.
- 확인된 로컬 글꼴만 CSS/Canvas 렌더링 font chain에 원본 글꼴명으로 반영한다.
- WASM Canvas2D 렌더링 경로에도 동일한 font substitution 규칙을 적용한다.
- 환경설정에서 감지 상태 확인, 재감지, 감지 결과 초기화를 제공한다.

### 2.2 테스트와 샘플

- `rhwp-studio/tests/local-fonts.test.ts`
- `rhwp-studio/tests/document-font-status.test.ts`
- `rhwp-studio/tests/font-substitution.test.ts`
- `samples/hwpx/local-font-nanumsquare-bold.hwpx`

나눔스퀘어 Bold 샘플은 승인 전/후 렌더링 차이를 확인하기 위한 작은 HWPX fixture다.

### 2.3 문서

- Hyper-Waterfall 계획/구현 계획서
- Stage 1~5 완료 보고서
- 샘플 fixture 제작 기록
- AI 샘플 문서 작성 가이드
- 최종 결과 보고서

## 3. 커밋 구조

Hyper-Waterfall 단계별 추적을 위해 단일 커밋을 다음 구조로 재구성했다.

1. `Task #1328: Add local font consent plans`
2. `Task #1328: Stage 1 local font state model`
3. `Task #1328: Stage 2 local font consent modal`
4. `Task #1328: Stage 3 gate display font chains`
5. `Task #1328: Stage 4 refresh local font UI`
6. `Task #1328: Stage 5 finalize local font settings`
7. `Task #1328: Add sample fixture and final report`

본 review 문서 커밋은 collaborator self-merge 후보 운영 기록이다.

## 4. 보안/프라이버시 검토

- 문서 로드만으로 로컬 글꼴 목록을 자동 조회하지 않는다.
- 감지 버튼을 누른 뒤에만 로컬 글꼴 감지를 수행한다.
- 감지 결과는 브라우저/확장 로컬 저장소에만 저장한다고 UX에 명시한다.
- 서버 전송 경로는 없다.
- Firefox fallback은 전체 로컬 글꼴 목록을 열거하지 않고 문서 후보 글꼴명만 확인한다.
- GitHub Advanced Security가 지적한 테스트 정규식 비효율 문제는 선형 파서로 교체해 반영했다.

## 5. 리스크

| 리스크 | 판단 |
|--------|------|
| Local Font Access API 권한 UX | 사용자 동의 모달 선행으로 제어한다. |
| Firefox 동작 차이 | 전체 목록 감지가 아니라 문서 후보 확인으로 제한하고 문구로 안내한다. |
| 렌더링 font chain 변경 | 승인 전에는 미확인 로컬 글꼴을 사용하지 않도록 테스트로 고정했다. |
| Canvas2D font setter wrapping | WASM 렌더링 실제 적용을 위해 필요하며, `wasm-bridge.ts` 경로에 한정된다. |
| 문서/샘플 diff 증가 | #1328 작업 검증 기록과 AI 샘플 작성 가이드로 PR 의도와 재현성을 높인다. |

## 6. 검증

PR head `00f462db` 기준 로컬 검증:

```bash
git diff --check upstream/devel...HEAD
cd rhwp-studio && npm test
cd rhwp-studio && npm run build
cargo fmt --all -- --check
cargo test
cargo clippy -- -D warnings
```

GitHub Actions 작성 시점 확인:

- Build & Test: success
- Canvas visual diff: success
- CodeQL: success
- Analyze (javascript-typescript): success
- Analyze (python): success
- Analyze (rust): success
- WASM Build: skipped

본 review 문서 커밋 push 후 GitHub Actions가 다시 실행되므로, merge 전 최신 head 기준으로 위 상태를 재확인한다.

## 7. 판단

작성 시점 기준으로 기능 변경, 보안 의도, Firefox fallback, CodeQL 지적 반영, Hyper-Waterfall 커밋 구조가 모두 확인되었다.

최종 조건:

1. 본 review 문서 2건이 PR head에 포함된다.
2. push 후 최신 PR head 기준 GitHub Actions가 통과한다.
3. 작업지시자 승인 상태가 유지된다.

위 조건 충족 시 collaborator self-merge 후보로 merge 수용한다.
