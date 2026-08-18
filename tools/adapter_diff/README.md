# adapter_diff — 전어댑터 상호 diff 골든 하네스 (M06-4)

판정 도구. 있는 어댑터끼리 구조·capability·산출 해시/bbox 를 맞댄다.

- `src/renderer/**` 는 건드리지 않는다.
- gym / `scripts/visual_sweep.py` 는 건드리지 않는다.
- devel 에는 `SvgBackend` 와 계측 백엔드만 있다. Png/Skia 는 PR 에만 있을 수 있다.
- 없는 어댑터는 침묵하지 않고 `skipped_missing` / `skipped_unexported` 로 남긴다.

## 1커맨드 (CI)

저장소 루트에서:

```text
python tools/adapter_diff/harness.py --ci
python tools/adapter_diff/harness.py --ci --json
node scripts/run-adapter-diff.mjs --cargo-test
```

`--ci` 는 `tools/adapter_diff/fixtures/ci-scene.json` 을 읽는다. samples/ 전수가 아니다.

단위 시험 (가짜 트리, 실문서 불필요):

```text
python -m unittest tools.adapter_diff.test_harness
node --test scripts/tests/run-adapter-diff.test.mjs
```

## 발견

| 이름 | 원본 | devel |
| --- | --- | --- |
| svg | `src/render_backend/svg_adapter.rs` | 있음 (필수) |
| null / trace | `src/render_backend/backends.rs` | 있음 (필수) |
| png | `src/render_backend/png_adapter.rs` | 없을 수 있음 — skip |
| skia | `src/render_backend/skia_adapter.rs` | 없을 수 있음 — skip |

파일이 있어도 `mod.rs` 가 타입을 내보내지 않으면 `skipped_unexported` 다.

## 판정

| 판정 | 뜻 |
| --- | --- |
| PRESENT | 파일+export. 상호 diff 대상 |
| SKIPPED_MISSING | 원본 파일 없음 — 정직한 skip |
| SKIPPED_UNEXPORTED | 파일은 있으나 미등록 |
| FAMILY_OK | 광고 family 가 픽스처와 같음 |
| FAMILY_MISMATCH | 광고 family 가 픽스처와 다름 |

없는 어댑터를 MATCH 로 꾸미지 않는다.

## 종료 코드

| 코드 | 조건 |
| --- | --- |
| 0 | 기본. skip 도 데이터 |
| 1 | `--strict` 이고 FAMILY_MISMATCH · ERROR · 필수 어댑터 부재 |
| 2 | 인자/픽스처 사용법 오류 |
