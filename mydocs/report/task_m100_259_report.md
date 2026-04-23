# Task #259 최종 결과 보고서

- 이슈: [#259](https://github.com/edwardkim/rhwp/issues/259)  HY 계열 한글 폰트명 → 메트릭 DB 영문명 매핑 누락 → 글자 폭 오류·겹침
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task259`
- 기간: 2026-04-23 (당일 착수·완료)
- 재현 샘플: `samples/text-align.hwp` 4번 문단 (작업지시자 시각 확인)

## 1. 요약

웹 에디터에서 `text-align.hwp` 4번 문단의 HY중고딕 텍스트가 글자 폭 오류로 겹쳐 보이는 현상을 해결했다. 근본 원인은 폰트명 2단 정규화 경로의 중간이 끊어져 있었던 것 — `style_resolver.rs` 는 한국어 별칭(`한양중고딕`)을 한국어 정규명(`HY중고딕`)으로 정규화하고, `font_metrics_data.rs::resolve_metric_alias` 는 한국어 정규명을 메트릭 DB 영문명(`HYGothic-Medium`)으로 매핑해야 하지만 **HY 계열 매핑이 전무**했다. 그 결과 `find_metric` 이 `None` 을 반환하고 호출부가 기본 폴백 폭을 적용하여 후속 글자가 앞 글자와 겹쳤다.

Stage 2 커밋으로 HY 계열 7종(중고딕·견고딕·견명조·신명조·그래픽·헤드라인M·궁서) alias 를 추가하고 단위 테스트를 붙였다. Stage 3 의 before/after SVG diff 로 글자 폭이 폴백 상수(예: 7.67px) 에서 실측 가변폭(`0`=9.04, `,`=5.58 등) 으로 교체됨을 확인.

## 2. 변경 사항

### 소스 수정

| 파일 | 변경 |
|------|------|
| `src/renderer/font_metrics_data.rs` | `resolve_metric_alias` 에 HY 7 arm 추가 (+ `cfg(test)` 단위 테스트 모듈) |

실제 변경 라인 수: 약 30 라인 (소스 8 + 테스트 22).

### 최종 매핑 (7종)

```rust
"HY중고딕"     => "HYGothic-Medium",
"HY견고딕"     => "HYGothic-Extra",
"HY견명조"     => "HYMyeongJo-Extra",
"HY신명조"     => "HYSinMyeongJo-Medium",
"HY그래픽"     => "HYGraphic-Medium",
"HY헤드라인M"  => "HYHeadLine-Medium",
"HY궁서" | "HY궁서B" => "HYGungSo-Bold",
```

근거: 한컴 폰트 PostScript name 관례 + 공개 폰트 메타데이터(Fontke / Wfonts / Fontsgeek / Koreafont) 교차 확인. 상세 근거는 `mydocs/tech/task_259_hy_mapping.md`.

## 3. 단계별 수행 요약

| # | 단계 | 커밋 | 산출물 |
|---|------|------|--------|
| 1 | HY 매핑 테이블 확정 | `f9791d7` | `tech/task_259_hy_mapping.md`, `working/_stage1.md`, 수행·구현 계획서 |
| 2 | `resolve_metric_alias` 에 7 arm 추가 + 단위 테스트 | `7f158de` | 소스 수정, `working/_stage2.md` |
| 3 | `text-align.hwp` 회귀 검증 (before/after SVG diff) | `c62025f` | `working/_stage3.md` |
| 4 | 타 HY 계열 5종 스모크 검증 (5개 샘플 정상 렌더) | `96fb740` | `working/_stage4.md` |
| 5 | 최종 보고서 + 메모리 등록 + orders 갱신 | 본 커밋 | 본 보고서, memory 등록 |

## 4. 검증 결과

| 검증 | 결과 |
|------|------|
| Stage 2 단위 테스트 (`hy_normalized_names_resolve_to_db_entries`) | ✅ 7종 전수 통과 |
| `cargo test --lib` 전체 | ✅ 948 passed · 0 failed |
| `cargo test --test svg_snapshot` | ✅ 3 passed (golden 갱신 불필요) |
| `cargo clippy --lib --tests` | ✅ 신규 경고 없음 |
| before/after SVG diff (text-align.hwp) | 322 라인 차이 — 글자별 x 좌표 전수 갱신 |
| HY 5종 스모크 (samples/*.hwp 중 대표 5개) | ✅ panic·렌더 결손 없음 |

## 5. 범위 외 / 후속 이슈

- **HY그래픽 / HY궁서 실측 렌더 회귀**: `samples/` 에 해당 폰트 사용 파일이 없어 alias 정합성만 단위 테스트로 검증. 실제 렌더 확인은 샘플 확보 후 별도 이슈에서.
- **웹 에디터 최종 시각 확인**: rhwp-studio 에서 브라우저 시각 확인은 WASM 재빌드가 필요하므로 본 타스크에서는 네이티브 SVG 수치 검증으로 갈음. WASM 배포 사이클에서 자연 반영될 예정.
- **DB 엔트리 약칭(HYbdaL 등)**: style_resolver 가 직접 이 이름으로 정규화하지 않는 한 본 타스크에서 건드리지 않음.

## 6. 메모리 등록

`feedback_font_metrics_alias_sync.md` 등록 — **폰트 메트릭 DB 변경 시 resolve_metric_alias 도 함께 갱신** 해야 한다는 교훈. 위치: `~/.claude/projects/-home-planet-iop-rhwp/memory/`.

## 7. 감사

- 작업지시자 시각 확인 — PDF 비교 없이 식별 불가능한 미묘한 글자 겹침을 웹 에디터 렌더에서 발견.
- 작업지시자 `font.reg` (한컴 공식 폰트명 레지스트리) 참조 지시로 Stage 1 매핑 근거 확보.
- 작업지시자 HY궁서 방어적 추가 지시로 HWP 직접 저장 케이스까지 커버.

## 8. 관련 이슈·파일

- `src/renderer/font_metrics_data.rs::resolve_metric_alias`
- `src/renderer/style_resolver.rs` (상류 정규화)
- [`mydocs/tech/task_259_hy_mapping.md`](../tech/task_259_hy_mapping.md)
- Issue [#253](https://github.com/edwardkim/rhwp/issues/253) Visual Diff 하네스 — 본 버그가 재발하면 즉시 감지할 수 있도록 하는 기반 인프라
