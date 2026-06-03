# Task M100 #476 최종 결과 보고서

| 항목 | 내용 |
|------|------|
| 이슈 | [#476](https://github.com/edwardkim/rhwp/issues/476) |
| 마일스톤 | M100 (v1.0.0) |
| 브랜치 | `local/task476` |
| 단계 | Stage 1~4 완료 |

## 1. 증상

`samples/21_언어_기출_편집가능본.hwp` 페이지 12의 22번 문제 위쪽 빈 영역(y≈742px)에 **박스 + "배너지와 뒤플로" 텍스트가 잘못 출현**. 사용자 보고 — "왼쪽 박스 아래 '배너지와 뒤플로'".

## 2. 근본 원인

paragraph 238(21번 본문, 26줄)의 인라인 박스가 char index 0(line 0)에 있고, paragraph가 페이지 11/12에 걸쳐 분할되었다.

핵심 흐름:

1. **TypesetEngine**(메인) / Paginator(fallback)의 `Control::Shape` 분기가 **paragraph 처리 종료 직전(`current_items`)에 무조건 `PageItem::Shape`를 push** — paginate가 이미 페이지 12로 진행한 상태이므로 박스가 페이지 12에 등록됨.
2. paragraph_layout(페이지 12)은 lines 4..26만 처리 — 박스(line 0)는 페이지 11에서 처리되었으므로 페이지 12 tree의 `inline_shape_positions`에는 박스 좌표 미등록.
3. 페이지 12 `layout_column_shapes_pass`가 `get_inline_shape_position()` → `None` → `compute_object_position` fallback (`shape_layout.rs:218`).
4. Shape 위치 "가로/세로=문단 오프셋=0,0"이라서 paragraph 238의 페이지 12 시작 y(742.45)에 박스를 잘못 다시 그림.

또한 부수적으로 페이지 11 단 1의 정상 박스는 `PageItem::Shape`가 페이지 11에 미등록되어 누락 상태였음(같은 원인).

## 3. 수정

### 3-1. 본질 수정 (A) — paginator/typeset 라우팅 정정

공용 함수 `find_inline_control_target_page`를 `src/renderer/pagination.rs`에 추가 — 박스의 char 위치 → line index → 그 line이 라우팅된 (page_idx, col_idx) 반환.

`Paginator::process_controls`(engine.rs)와 `TypesetEngine::typeset_section`(typeset.rs) 양쪽의 Shape 분기에서 호출하여, treat_as_char Shape를 박스가 속한 페이지/단의 `column_contents.items`에 직접 push.

### 3-2. 회귀 안전장치 (D) — fallback 경로 차단

`shape_layout.rs:218`에 `treat_as_char + inline_pos=None`인 경우 박스 렌더 스킵 추가. paginator 라우팅 미커버 케이스에서 잘못된 위치 출현을 막는 시그널 + `RHWP_DEBUG_LAYOUT=1` 로깅.

### 3-3. 빈 paragraph + 인라인 Shape 보강

`paragraph_layout.rs:2087` 빈 paragraph 분기에 Shape의 `set_inline_shape_position` 등록 추가 — Picture에만 있던 처리를 Shape에도 확장하여 D 차단의 false positive 방지.

## 4. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/pagination.rs` | `find_inline_control_target_page` 공용 함수 추가 |
| `src/renderer/pagination/engine.rs` | Paginator Shape 분기 라우팅 적용 |
| `src/renderer/typeset.rs` | TypesetEngine Shape 분기 라우팅 적용 (메인 경로) |
| `src/renderer/layout/shape_layout.rs` | D 안전장치 (fallback 차단 + 디버그 로그) |
| `src/renderer/layout/paragraph_layout.rs` | 빈 paragraph + 인라인 Shape의 inline_pos 등록 |

## 5. 검증

### 5-1. 단위/통합 테스트

```
cargo test --release
test result: ok. 1078 passed; 0 failed; 1 ignored; (lib)
test result: ok. 14, 25, 1, 6, ... (모든 통합 테스트 통과)
```

`test_task78_rectangle_textbox_inline_images` 포함 모든 회귀 통과.

### 5-2. 골든 SVG 스냅샷

```
cargo test --release --test svg_snapshot
test result: ok. 6 passed; 0 failed;
```

기존 시각 회귀 없음.

### 5-3. 핵심 회귀 케이스

`samples/21_언어_기출_편집가능본.hwp` 페이지 11/12:

| 항목 | Before | After |
|------|--------|-------|
| 페이지 11 단 1 끝의 paragraph 238 박스 | 누락 ❌ | 출현 ✓ (x=616.1, y=1313.3) |
| 페이지 12 22번 위 잘못된 박스 (y=742) | 출현 ❌ | 제거 ✓ |
| 페이지 12 23번 박스 | 출현 (위치 별도) | 출현 (위치 별도) |

PDF 정답과의 정성 비교 — paragraph 238 박스 위치 일치. 사용자 보고 핵심 증상 해결.

## 6. 잔여 이슈 (별도 분리)

### 23번 paragraph 위치 어긋남

페이지 12 단 0의 23번 paragraph가 단 끝(y≈1166)에 그려지고 답안 일부가 단 밖으로 밀려나 잘림 — 단 0 사용량 `used=1012px vs hwp_used=1219px (diff=-206.8px)`.

원인: paragraph 238의 페이지 12 영역(lines 4..26)이 차지하는 높이 계산 또는 단 분배가 한컴과 다름. 본 task의 박스 라우팅 수정과는 별개의 layout/페이지네이션 버그.

별도 이슈 등록 권장.

## 7. 향후 개선 후보

- **옵션 C** (수행 계획서 4번 항목): treat_as_char Shape를 paragraph_layout이 직접 렌더하고 별도 PageItem::Shape 등록 자체를 제거. 현재의 inline_pos 우회 패턴을 폐기하는 깔끔한 구조 — z-order 정렬 패스와의 호환 검토 필요. 별도 리팩터 타스크.
- D 안전장치는 회귀 시그널로 유지. 향후 paginator/typeset 통합 시 함께 정리.

## 8. 요약

- 사용자 보고 증상(잘못된 박스+텍스트 출현) 해결 ✓
- 부수적으로 페이지 11 누락 박스도 정상화 ✓
- 회귀 없음 (단위 1078건 + 통합 + 스냅샷 6건 모두 통과) ✓
- 안전장치(D) + 디버그 로그 추가로 향후 회귀 즉시 감지 가능 ✓
