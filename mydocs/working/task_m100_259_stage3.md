# Task #259 Stage 3 완료 보고서 — text-align.hwp 회귀 검증

- 일자: 2026-04-23
- 브랜치: `local/task259`
- 작업: HY 계열 alias 적용 후 `samples/text-align.hwp` 의 실제 렌더링 회귀 검증

## 샘플 특성

`samples/text-align.hwp` 는 1페이지 문서이며, 본문에 **HY중고딕 · HY헤드라인M** (SVG font-family 확인) 을 사용한다. 문단 0.4 는 지점 나열 텍스트(66자, cc=67) 로 숫자·한글·특수문자 혼용 — Stage 2 alias 수정 효과가 글자 폭에 직접 반영되는 대표 문단.

`휴먼명조` (line 483 에서 `HY신명조` 로 정규화 → HYSinMyeongJo-Medium) 도 문서 내 사용됨 — HY 경로 3종(중고딕/헤드라인M/휴먼명조→신명조) 이 동일 문서에서 검증됨.

## 검증 방법

1. **before 빌드** (`HEAD~1`, Stage 2 커밋 직전 소스) 로 release 컴파일 후 SVG 생성
2. **after 빌드** (현재 `HEAD`) 로 release 컴파일 후 SVG 생성
3. 두 SVG 를 `diff` 하여 글자 x 좌표 변화 확인

### 수행 절차

```bash
git show HEAD~1:src/renderer/font_metrics_data.rs > /tmp/before_metrics.rs
cp src/renderer/font_metrics_data.rs /tmp/after_metrics.rs
cp /tmp/before_metrics.rs src/renderer/font_metrics_data.rs
cargo build --release
./target/release/rhwp export-svg samples/text-align.hwp -p 0 -o /tmp/before/
cp /tmp/after_metrics.rs src/renderer/font_metrics_data.rs
cargo build --release
./target/release/rhwp export-svg samples/text-align.hwp -p 0 -o output/svg/text-align-task259/
diff /tmp/before/text-align.svg output/svg/text-align-task259/text-align.svg
```

## 정량적 결과

### SVG diff 라인 수
- 차이: **322 라인** (글자별 `<text x=...>` 요소 전수 갱신)

### 글자 폭 샘플 (문단 0.4, font-size=16.67px)

| 글자 | before x | after x | 폭 (after) | 비고 |
|------|----------|---------|-----------|------|
| `1` (첫) | 301.41 | 300.41 | - | 기준점 |
| `,` | 309.08 | 309.45 | 9.04 | after 가 더 정확 |
| `0` | 316.75 | 315.03 | 5.58 | before=7.67(고정폭) → after=가변 |
| `0` | 324.41 | 324.07 | 9.04 | |
| `0` | 332.08 | 333.11 | 9.04 | |
| `항` | 339.41 | 342.41 | 9.31 | 한글 폭 복원 |
| `목` | 355.41 | 358.41 | 16.0 | |

**핵심 관찰**: before 에서는 모든 라틴 문자가 ~7.67px 균일 폭 (폴백 상수) 으로 찍혔으나, after 는 각 글자가 HYGothic-Medium 실측 폭 (`,`=5.58, `0`=9.04, `3`=9.04 등) 으로 반영되어 제각각 달라짐. 숫자 폭이 실제 폰트 메트릭에 가까워지며 후속 글자와의 간격 계산이 정상화.

## 테스트 결과

| 테스트 | 결과 |
|--------|------|
| `cargo test --lib` (948개) | ✅ all passed |
| `cargo test --test svg_snapshot` (3개) | ✅ all passed — golden 재생성 불필요 (table_text / form_002 는 HY 계열 미사용) |
| `cargo clippy --lib --tests` | ✅ 신규 경고 없음 |

## 산출물

- `output/svg/text-align-task259/text-align.svg` — after 렌더
- `output/debug/text-align-task259/text-align.svg` — after + debug overlay
- `/tmp/before/text-align.svg` — before 렌더 (로컬, 커밋 제외)
- 본 보고서

## 웹 에디터 시각 확인

현 Stage 3 는 **네이티브 SVG 레벨에서의 수치적 검증**이다. 웹 에디터(rhwp-studio)에서는 WASM 빌드(Docker)를 통해 동일 코드가 실행되므로, 본 수정이 적용된 WASM 패키지가 배포되면 브라우저 상에서도 동일 효과가 나타난다. 웹 브라우저 시각 최종 확인은 다음 절차로 가능하다:

```bash
docker compose --env-file .env.docker run --rm wasm    # WASM 재빌드
cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700
# 브라우저에서 text-align.hwp 열어 s0:pi=4 문단 겹침 해소 확인
```

(WASM 재빌드는 본 타스크 범위 외 — rhwp-studio 배포 사이클에서 자연 반영)

## 승인 요청

Stage 3 완료 승인 및 Stage 4 (타 HY 계열 스모크 검증) 착수 승인을 요청드립니다.
