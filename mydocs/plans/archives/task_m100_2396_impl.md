# 구현계획서 — #2396 custom scheme 최상위 legacy 요청 회귀

- 수행계획서: `mydocs/plans/task_m100_2396.md`
- 구현 대상: `rhwp-studio/src/embed/runtime.ts`
- 테스트 대상: `rhwp-studio/tests/embed-protocol.test.ts`

## 현재 코드 경로

`installEmbedRuntime()`의 `message` listener는 다음 순서로 동작한다.

1. transferred ports를 수집한다.
2. `event.source === options.parentWindow`와 `isUsableParentOrigin(event.origin)`을 검사한다.
3. v1 `rhwp-connect`이면 port/session binding을 설치한다.
4. binding이 없고 connect가 아니면 legacy handler로 전달한다.

회귀는 2번에서 top-level same-window와 iframe parent를 구분하지 않는 데서 발생한다.

## Stage 1 상세 — 테스트만 변경

### positive case

- `hostWindow`와 `parentWindow`에 같은 fake window 객체를 전달한다.
- source도 같은 객체, origin은 `alhangeul-studio://app`으로 구성한다.
- legacy `ready` 요청이 `{ type: 'rhwp-response', id, result: true }`를 같은 custom origin으로 반환하는지
  확인한다.
- 현행 구현에서는 timeout되어야 하며 이 실패를 Stage 1 기준선으로 기록한다.

### negative cases

- `hostWindow === parentWindow`인 top-level 환경에서도 custom origin v1 `rhwp-connect`는 거부한다.
- 거부된 v1 connect의 port를 정리하고, 이후 legacy `ready`가 정상 응답하는지 확인한다.
- `hostWindow !== parentWindow`인 iframe 환경에서 exact parent source가 custom origin을 사용하면 거부한다.
- sibling 객체를 source로 사용한 custom origin 요청도 거부한다.
- 두 요청 모두 handler 호출과 response가 없고 transferred port가 닫히는지 확인한다.

Stage 1에서는 runtime 구현을 변경하지 않는다.

## Stage 2 상세 — runtime 최소 변경

runtime 설치 시 다음 identity를 계산한다.

```ts
const isTopLevelSameWindow = options.parentWindow === options.hostWindow;
```

초기 gate는 다음 논리로 변경한다.

```text
source가 parent가 아니면 거부
origin이 HTTP(S)가 아니고 top-level same-window legacy rhwp-request도 아니면 거부
그 밖에는 기존 connect/legacy 분기로 진행
```

`isUsableParentOrigin()` 자체는 변경하지 않는다. custom scheme 문자열 allowlist도 추가하지 않는다.
따라서 iframe의 origin 규칙과 source identity 경계는 기존과 동일하며, v1 MessageChannel connect도
계속 HTTP(S) origin에서만 binding을 설치한다.

## Stage 3 상세 — 검증

```bash
node --test rhwp-studio/tests/embed-protocol.test.ts
npm --prefix rhwp-studio test
wasm-pack build --target web --out-dir pkg --dev
npm --prefix rhwp-studio run build
git diff --check
```

- 새 worktree에 `node_modules`가 없으면 lockfile 기준 `npm ci` 후 재실행한다.
- `pkg/`, `target/`, `node_modules/`, `dist/`는 gitignored 생성물이며 stage하지 않는다.
- UI 레이아웃이나 렌더링 변경이 없으므로 시각 sweep은 적용하지 않는다.
- 실제 WKWebView downstream representative suite는 이 저장소에서 실행할 수 없으므로 최종 보고의 잔여
  검증으로 명시한다.

## 위험과 완화

| 위험 | 완화 |
|---|---|
| custom scheme을 iframe까지 허용 | window identity를 top-level same-window로 한정하고 negative test 고정 |
| forged sibling이 우회 | `event.source === parentWindow` 조건을 유지하고 source mismatch test 고정 |
| custom scheme에서 MessageChannel binding까지 열림 | 예외 type을 legacy `rhwp-request`로 제한하고 v1 connect 거부·port 정리 테스트 고정 |
| 생성물이 커밋에 포함 | 각 단계 커밋 전 `git status --short --ignored`와 staged name 검사 |

## 롤백

Stage 2의 runtime 조건 변경 한 곳과 회귀 테스트 세 건을 단계 커밋 단위로 되돌릴 수 있다.
비준수 이전 구현은 단계 진행 중 별도 backup ref로만 보존하고, 정상 단계 이력과 최종 검증을 확인한 뒤
삭제한다. 정상 작업 이력에는 재사용하지 않는다.
