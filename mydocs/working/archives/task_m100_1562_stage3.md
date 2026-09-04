# Stage 3 완료보고서 — Task #1562

> HWPX 폼 컨트롤 caption `&&` 표시 정합 — SVG golden 갱신 및 회귀 검증

- **이슈**: [#1562](https://github.com/edwardkim/rhwp/issues/1562)
- **브랜치**: `local/task1562`
- **작성일**: 2026-06-26

---

## 1. 수행 내용

Stage 2에서 적용한 form caption 표시 helper 결과를 SVG snapshot golden에 반영했다.

변경 파일:

- `tests/golden_svg/form-002/page-0.svg`

의도된 diff:

- `IP R&amp;&amp;D연계` → `IP R&amp;D연계`
- `R&amp;&amp;D 자율성트랙(일반)` → `R&amp;D 자율성트랙(일반)`
- `R&amp;&amp;D 자율성트랙(지정)` → `R&amp;D 자율성트랙(지정)`

좌표, 폰트, 체크박스 도형, 기타 본문 텍스트 변경은 없었다.

## 2. 검증 결과

실행:

```text
env UPDATE_GOLDEN=1 cargo test --test svg_snapshot form_002
cargo test --test svg_snapshot form_002
cargo test --test issue_1534_hwpx_form_caption_escape
cargo test --test issue_1562_hwpx_form_caption_display
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

결과:

- `svg_snapshot form_002`: 통과
- #1534 저장/roundtrip 회귀: 4개 통과
- #1562 SVG 표시 회귀: 1개 통과
- `cargo fmt --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과

## 3. 저장값 보존 확인

#1534 테스트가 통과했으므로 이번 변경은 parser/serializer 계층의 저장값을 변경하지 않는다.

- 저장 모델: `R&&D` 유지
- HWPX XML: `R&amp;&amp;D` 유지
- 표시 계층: `R&D`로 출력

## 4. 다음 단계

Stage 4에서 최종 보고서를 작성한다.

포함 예정:

- 원인: 저장 caption을 표시 caption으로 그대로 사용
- 수정: form caption display helper와 SVG/Web Canvas/Skia 적용
- 검증: Stage 2/3 테스트 결과
- 잔여 범위: 단일 `&` mnemonic prefix 제거/밑줄 표시는 미적용
- #1534 parent issue close 판단

## 5. 승인 요청

Stage 3 회귀 검증이 완료되었다. Stage 4 최종 보고서 작성으로 진행한다.
