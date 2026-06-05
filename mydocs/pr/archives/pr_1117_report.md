# PR #1117 처리 결과 보고서

- PR: <https://github.com/edwardkim/rhwp/pull/1117>
- 제목: `render: gate advanced glyph outline payloads`
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/536>
- 처리일: 2026-05-26
- 작성자: Codex

## 1. 처리 요약

PR #1117은 권장안대로 체리픽 수용했다.

이 PR은 `GlyphOutline`의 advanced payload vocabulary를 추가하고, CanvasKit에서 COLRv1의 제한된
solid/gradient graph subset만 명시적으로 replay하도록 gate를 추가한다. 기본 renderer 선택은 바꾸지
않고, unsupported payload는 `TextRun` fallback 및 diagnostic으로 유지한다.

## 2. 반영 커밋

체리픽한 커밋:

```text
c54e80d0 feat: gate advanced glyph outline payloads
3945a937 test: expose glyph outline payload diagnostics
225e8229 docs: document P19 glyph payload gates
4d1938e7 feat: replay gated colrv1 glyph gradients
90f6fa84 test: align layer schema tuple expectation
606505bd fix: address p19 glyph gate review
```

원본 PR head:

```text
b2134a312d88a461173cfbe45b67fa675ae8810a
```

## 3. 반영 내용

주요 변경:

```text
1. GlyphOutline payload family 추가
   - colorLayers
   - bitmapGlyph
   - svgGlyph

2. payload family exclusivity 및 contract validation 추가

3. COLRv1 bounded graph contract 추가
   - node count/depth bound
   - duplicate/missing/cycle 검증
   - supported node kind whitelist
   - non-empty glyph range 검증

4. CanvasKit에서 COLRv1 제한 subset replay
   - solid path
   - linear gradient path
   - radial gradient path
   - full-circle sweep gradient path
   - transform chain

5. Studio TypeScript type/gate/diagnostic helper 추가

6. README 및 docs/text-ir-v2.md에 P19 gate 문서화
```

Schema 변경:

```text
schemaVersion = 1 유지
schemaMinorVersion = 14
```

## 4. 검토 결과

검토 결론:

```text
수용 가능
```

이유:

```text
1. 기본 renderer 동작은 그대로 유지된다.
2. advanced payload는 default selection에서 직접 replay되지 않는다.
3. CanvasKit은 명시적으로 열린 COLRv1 subset만 replay한다.
4. unsupported graph/payload는 fallback과 reject reason으로 남긴다.
5. Rust gate와 TypeScript gate가 주요 contract를 맞춘다.
6. Copilot 지적 10개가 후속 커밋에서 반영되었다.
```

추가 maintainer-side 코드 수정은 하지 않았다.

## 5. 검증

실행한 검증:

```text
cargo fmt --check
cargo check
cargo test --lib colrv1
cargo test --lib serializes_advanced_glyph_outline_payload_gate_metadata
npm --prefix rhwp-studio test -- render-backend
git diff --check
docker compose --env-file .env.docker run --rm wasm
npm --prefix rhwp-studio run build
```

결과:

```text
success
```

확인된 warning:

```text
- 기존 Rust test warning만 재출력됨
- Vite chunk size warning 및 canvaskit-wasm의 fs/path browser externalization 안내
```

이번 PR 처리에서 새로 해결해야 할 실패는 없었다.

## 6. 남은 절차

보고서 승인 후 다음 절차를 진행한다.

```text
1. 검토/완료 보고서 커밋
2. PR #1117에 처리 결과 코멘트 작성
3. PR #1117 close
4. local/devel -> devel 병합
5. devel에서 필요한 검증 확인
6. origin/devel push
```
