# #3587 Stage 21 — D2 WASM 가져오기 경계

- 승인: Stage 20 산출물 한컴 정상 열림 확인 후 「다음 절차 진행을 승인합니다」.
- 범위: D1 core를 불변 source 핸들 + options JSON으로 WASM에 연결하고 실제 JS 실행으로 검증.
- 계약: `target.importParagraphBlock(source, {request,dryRun}의 JSON)`.
  두 핸들은 서로 다른 문서다. 기존 `applyTemplateOperation`의 단일 문서 계약은 바꾸지 않는다.
- 8 MiB JSON 상한, 엄격한 타입/미지 필드 거부, 동일 core의 full dry-run/실행, 원본 불변,
  대상 앞뒤 문단 보존과 오류 원자성을 확인한다. URL 실행·파일 저장은 하지 않는다.
- CLI source 파일/지문·MCP·schema/레시피의 완결 연결은 D2의 다음 묶음으로 남긴다.
  공개 연결 전체 완료 또는 PR 준비 완료로 보고하지 않는다.

## 구현

- 시작 commit `4c21f93a4`(직전 한컴 확인 기록), 제품 기준 `a99046f12`.
- `paragraph_block/import_json.rs`는 크기 제한 후 엄격한 options를 역직렬화하고 기존
  full native preview/import를 호출한다. 다른 소유 트리 순회·자원 매핑은 추가하지 않았다.
- `wasm_api/template_automation.rs`는 별도의 불변 `HwpDocument`를 받아 위 JSON 경계로 위임한다.
  저장은 별도 export API이며 응답은 경로/자원 집계만 포함한다.
- 응답 봉투 버전은 `schema_registry::ENVELOPE_SCHEMA_VERSION`을 참조한다.
- #2724 저장 무효화 가드에 `import_paragraph_block_native` 위임 근거를 추가했다.
  무효화 자체를 중복 호출하거나 기존 가드를 완화하지 않았다.
- `template_automation.md`에 API/옵션/상한/오류/주소/비범위를 명시했다.

## native 검증

기존 review worktree `/home/edward/mygithub/rhwp-review-3587`에 이번 제품·테스트 파일을 같은
바이트로 반영하고, 공유 target `/home/edward/mygithub/rhwp/target/pr-review`를 사용했다.
기존 review WIP는 삭제하지 않았다. 아래는 overlay 검증이며 새 최종 커밋 전체의 PR 검증이 아니다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --check
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  -E 'test(issue_3587) | test(foreign_paste) | test(issue_4275) | test(issue_5819) | test(passthrough_invalidation) | test(issue_1058)'
```

- 새 JSON 경계 포함 template_operation 먼저 **8 PASS**:
  run `0294f8c6-83dd-4af0-a398-8f852e6e5bb9`, compile 5m41s, tests 0.046s.
- 최종 집중 **186 PASS, 실패 0, 9565 skipped**:
  run `ec5154d3-fcef-43ca-8f42-072040330a37`, compile 1m38s, tests 0.668s.
- native typed/full preview/JSON 실행 결과 동치, 원본·대상 경계 보존, 잘못된 주소/타입/미지 필드/
  8 MiB 초과 입력 거부, `count=0` 무변경을 검사했다. 기존 #3587 A/B/C/D와 관련 가드도 통과했다.
- 가드 source 추가 뒤 manifest `--check`가 파생 suite drift를 알렸다. source 크기 변경으로
  배정이 바뀐 것이므로 진행 중 빌드가 끝난 뒤 `--prepare`하고 최종 집중 검사를 실행했다.
  generated suite·manifest는 review에서만 준비하고 stage하지 않는다.
- fmt·최종 manifest·`git diff --check`, JS probe `node --check` PASS.
  manifest: 1293 sources, 5568 static attrs, 48/48 targets.
- `cargo clippy --locked -p rhwp --lib --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings`
  PASS(58.23초). native lib 검사이며 PR 필수 세 Clippy 전체를 대신하지 않는다.

제품/테스트 SHA-256:

| 파일 | SHA-256 |
| --- | --- |
| `paragraph_block.rs` | `435da96dd5c9e94ff3e713bca7b555e6b3e4517a6406e83e6311d8051073a3fd` |
| `import_json.rs` | `51faadc92cf5d98192a2f4373b2c7eebab09ce31a9a1e1b15de6976f2fd37826` |
| `wasm_api/template_automation.rs` | `d71372400803dd8c9de5db63f5852a975e44d7344434d384b0685771baea944e` |
| `issue_3587_template_operation.rs` | `4c03a53230a0fa09031b80c8b9c8f122dbdf8f1498fcd004f24ca414ae456f31` |
| `issue_2724_passthrough_invalidation_guard.rs` | `56662c01f4308c3f0a94dbe89604b525f923e748cc5432c2af974fb6d02d8124` |

## Docker WASM

표준 `docker compose --env-file .env.docker run --rm wasm`를 주 작업 checkout에서 실행한다.
실제 JS probe는 `mydocs/tech/investigations/issue-3587/probes/import-wasm-contract.mjs`이며,
원본 HWP와 rhwp 파생 HWPX의 pi4를 다른 문서의 BEFORE/AFTER 사이에 두 번 가져온다.
이 입력은 transport 계약 검증이며 한컴 시각 정답지나 실제 배치 승인 샘플이 아니다.

Docker WASM 빌드 **PASS**, wasm-pack 총 11m10s(Rust release compile 8m12s 포함).
`pkg/rhwp.d.ts`에 `importParagraphBlock(source: HwpDocument, options_json: string): string`이 생성되었다.

```bash
node --check mydocs/tech/investigations/issue-3587/probes/import-wasm-contract.mjs
node mydocs/tech/investigations/issue-3587/probes/import-wasm-contract.mjs
```

실제 Node 24.15.0 JS/WASM 실행: **입력 2형식 PASS**. 두 형식 각각 dry-run/실행 결과 동치,
원본 불변, 대상 앞뒤 보존, 잘못된 JSON/초과 길이/옵션/주소 오류 후 무변경, count=0 무변경,
HWP/HWPX 저장 후 재열기와 표·글상자 텍스트 보존을 확인했다.
각 입력에서 서식 84개·바이너리 2개가 추가되었다. 바이너리 2개는 8개 그림 인스턴스가 공유한다.

증적: `output/3587/d2-wasm/result.json`.

| 산출물 | SHA-256 |
| --- | --- |
| `pkg/rhwp_bg.wasm` | `0e38cea689d678e9b08d6a5eaba538d9f004ace01ee4ce8309d3b1179d4b4d65` |
| `pkg/rhwp.js` | `a6b6e1564d302881e7db02fd1a1b4864e65acc4b0ea737517240ac60e2fa708a` |
| `pkg/rhwp.d.ts` | `13eeeb30796fc2e8e9c68009b4aae38e7d438a449550693c4d1bea95067c2f2e` |

### 동일 핸들 오사용의 명시적인 한계

최초 JS 검사에서 의도적으로 source와 target에 같은 객체를 넘겼다. wasm-bindgen이
mutable/shared borrow를 동시에 잡으려다 예외를 내며, 이후 free에서도
`attempted to take ownership of Rust value while it was borrowed`가 발생했다.
native 가져오기 요청 검사에 도달하기 전의 실패이며, 핸들 상태가 복구됐다고 주장하지 않는다.

계획의 전제는 **서로 다른 두 문서 핸들**이다. 사용 예제와 probe는 JS에서 `target === source`를
호출 전에 거부하도록 보완했다. 이 경로의 통과는 **호출자 사전검사**이지 직접 ABI 오사용의 복구
테스트 통과가 아니다. 제품 코드를 바꾸거나 실패 기대값을 정상 가져오기 성공으로 바꾸지 않았다.
이 제약은 소비자 문서에 명시했다. 동일 문서 복제는 기존 API를 사용한다.

## 완료 범위와 다음 순서

이번 묶음은 D2의 WASM 연결과 실제 호출 검증이다. 새 renderer 수정이나 브라우저 UI 시각 판정은
없으며 Stage 20의 한컴 정상 열림 판정을 새 합성 대상의 시각 동등성으로 확대하지 않는다.
WASM 산출물은 갱신했지만 실행 중 Studio가 이 버전을 로드했다고 HTTP/브라우저 검증한 것은 아니다.

다음은 CLI 단독 `import_paragraph_block` action에 source 파일/지문을 연결하고,
스키마·metadata·기존 MCP run·오류/출력 무변경 계약 및 가져오기→채우기 레시피를 함께 검증하는 것이다.
이후 D3의 전체 회귀·세 Clippy·Native Skia·비용 및 선택적 Gym 순서를 따른다.
이번 집중 검사는 PR 제출 전체 게이트가 아니며 remote push·PR·댓글·issue close는 하지 않았다.
