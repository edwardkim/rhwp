# Task M100-5587 완료 보고서 — 부모보다 넓은 중첩표 클리핑

- Issue: #5587 (`[renderer] 부모보다 넓은 중첩표를 클리핑하지 않아 점선 상자가 표 밖으로 삐져나온다 (00387)`)
- 브랜치: `fix/issue-5587-nested-table-clip` (base `stream/devel` 61baa6783)
- 변경: `src/renderer/layout/table_layout.rs`, `tests/cases/issue_5587_overwide_nested_table_clip.rs`
- 작성일: 2026-08-19 KST

## 1. 증상과 재현

`admrul_downloads/기후에너지환경부/3227255_[별지 2] 직무윤리 서약서…hwpx` (코퍼스 docid 00387)
1쪽의 점선 상자(중첩표)가 바깥 표 오른쪽 경계를 넘어 그려졌다.

| 요소 | 저장 폭 | px(96dpi) | 렌더 오른쪽 끝 |
| --- | --- | --- | --- |
| 바깥 표 셀 | 45,359HU | 604.8 | 680.37 |
| 중첩표(점선 상자) | 46,490HU | 619.9 | **704.88** |

즉 원본이 이미 부모보다 15.1px 넓게 저장돼 있고, 렌더는 그 선언 폭을 그대로 그려
부모 오른쪽으로 24.5px 삐져나왔다.

## 2. 원인

패인트 경로가 아니라 **셀 clip 확장**이 원인이다.

`src/renderer/layout/table_layout.rs`의
`extend_clipped_cell_horizontal_clip_to_nested_table_borders` 는 42065(#2007) 계약을 위해
존재한다 — 저장 폭이 부모 셀 **이하**인 중첩표가 셀 왼쪽 패딩만큼 밀려 오른쪽 외곽
테두리를 host clip 밖에 두면, 그 테두리가 통째로 사라지므로 clip 을 테두리까지 넓힌다.

이 확장에 "부모보다 넓게 저장된 표" 예외가 없었다. 00387에서는 host 셀 clip 이
604.79px → 629.54px 로 넓어져(SVG `cell-clip-26`) 점선 4변이 부모 밖까지 그대로 칠해졌다.
셀 clip 은 SVG·Canvas·paint 공통 경로라서 렌더러 전체가 같은 증상을 보였다.

두 형상은 저장 폭 하나로 갈린다 (repo 픽스처 실측):

| 문서 | 부모 셀 | 중첩표 | 관계 |
| --- | --- | --- | --- |
| `issue2007_nested_cell_pagination_42065.hwp` | — | — | 부모보다 넓은 중첩표 **0건** (패딩 이동만) |
| `issue1994_behindtext_table_20200830.hwp` | 34,161HU | 35,144HU | 넓게 저장 (+13.1px) |
| 00387 | 45,359HU | 46,490HU | 넓게 저장 (+15.1px) |

## 3. 수정

직접 자식 표의 bbox 폭이 host clip 폭보다 크면(`NESTED_OVER_WIDE_EPSILON_PX = 0.5`)
clip 확장 대상에서 제외한다. host viewport 를 그대로 두면 셀 clip 이 중첩표의 테두리와
내용을 부모 경계에서 자른다. #2007 노출 경로와 중첩 셀 content 클램프는 그대로 둔다.

레이아웃·줄바꿈·쪽수는 건드리지 않는다 — 페인트 범위만 좁힌다.

## 4. 한컴 정답지

00387 자체의 한컴 PDF 는 없다. 대신 **부모보다 넓게 저장된 중첩표**를 가진 문서 중 한컴
PDF 가 함께 있는 두 건으로 방향을 확인했다.

### 4.1 코퍼스 오라클 쌍 (문서 id 36301151) — 수정 후 clip 이 한컴 clip 과 일치

부모 셀 48,194HU, 중첩표 48,440HU(+246HU). 같은 문서의 한컴 PDF(`hwpdocs_10k_share/_oracle_pdf_2022/…`, Hancom PDF 1.3.0.550) 1쪽에서 이 중첩표가 놓인 구간의 clip
사각형은 `mutool draw -F trace` 기준 **x = 59.49 ~ 541.15pt** 다.

| | rhwp host 셀 clip (px) | 오른쪽 끝(pt) | 한컴 clip 오른쪽(pt) | 차 |
| --- | --- | --- | --- | --- |
| 수정 전 | 79.36 + 650.61 | 547.48 | 541.15 | **+6.33pt** |
| 수정 후 | 79.36 + 642.59 | 541.46 | 541.15 | +0.31pt |

수정 전 페인트 viewport 는 한컴보다 6.3pt 넓었고, 수정 후 0.3pt(0.4px) 안으로 맞는다.
이 문서는 그 구간에 칠할 것이 없어 잉크는 변하지 않는다(SVG clip 값만 변경).

### 4.2 `issue1994_behindtext_table_20200830.hwp` — 부모 밖에는 아무것도 없다

repo 픽스처(부모 셀 34,161HU, 중첩표 35,144HU)와 그 한컴 출력
`samples/issue1994/issue_1994.pdf`(Creator: Hwp 2020 11.0.0.9083). host 셀 오른쪽은
512.2px = 384.15pt, 중첩표 선언 폭 오른쪽은 529.1px = 396.8pt 다. 1쪽 전체 경로를 뽑으면
x ∈ [391.5pt, 402pt] 구간에 stroke·clip·glyph 가 **하나도 없다** — 한컴은 중첩표 선언 폭의
오른쪽 끝에 아무것도 그리지 않는다. 다만 이 문서는 그 구간에 그릴 테두리 자체가 없어서
"한컴이 잘라낸다"까지 증명하지는 못한다. 새 회귀 테스트가 고정하는 값이 이 실측이다.

## 5. 검증

| 게이트 | 결과 |
| --- | --- |
| `issue_5587_overwide_nested_table_clip` (신규) | 통과 — 수정 원복 시 실패(가드 유효) |
| `issue_2007_nested_cell_pagination` | 15건 통과 (테두리 노출 계약 유지) |
| `cargo test --profile release-test --lib` | 3,893건 통과 |
| `cargo test --profile release-test --tests --no-fail-fast` | 49 suite ok / 2 실패 — `issue_4179_…`·`issue_4128_…` 두 perf 카운터 테스트. **둘 다 수정 없는 baseline 에서 동일하게 실패**하고 단독 실행은 통과(아래 §7) |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| Native Skia 3종 | **미실행** — 이 호스트에 `freetype`/`fontconfig` 개발 라이브러리가 없어 `--features native-skia` 링크 실패 (`rust-lld: unable to find library -lfreetype`) |
| Docker WASM | **미실행** — 표준 환경 없음 |

### 10k 코퍼스 전후 스윕

`/home/planet/hwpdocs_10k_share` 10,000건 중 9,995건 검사(경로 인용 실패 4건, 파싱 패닉 1건 제외).

| 항목 | 값 |
| --- | --- |
| 부모 셀보다 넓게 저장된 중첩표를 가진 문서 | **71건 (0.71%)** |
| 수정 전후 SVG 가 달라진 문서 | 35건 |
| 그중 픽셀(잉크)이 달라진 문서 | **20건 / 21쪽** |
| 제거된 잉크 | **6,709px** |
| 추가된 잉크 | **0px** |

전부 부모 표 오른쪽 경계 **밖**에서만 사라졌다. 잉크가 늘어난 픽셀은 한 점도 없다.
표본 육안 확인(overlay, 빨강=제거): 부모 세로 테두리 오른쪽으로 튀어나온 가로 테두리
꼬리와 중첩표 오른쪽 세로선만 사라지고 본문·표 내용은 그대로다.

## 6. 하지 않은 것

- 중첩표 축소(폭 스케일)·재조판. 정답지에 근거가 없고 줄바꿈·쪽수를 흔든다.
- 세로 방향 클리핑. RowBreak continuation 이 다음 쪽 내용을 clip 아래에 의도적으로
  남기는 기존 계약과 충돌한다.
- 이슈가 함께 지적한 **폭 583px 빈 TextRun**(x=377.3 → 오른쪽 끝 960.3, 쪽 폭 밖).
  칠하는 내용이 없고, 이제 host 셀 clip(604.79px) 안에서 잘리므로 이 결함의 경로가
  아니다. 렌더 트리 bbox 위생 문제로 별도 추적이 필요하다.

## 7. 기존 실패 2건 (이 변경과 무관)

- `issue_4179_cursor_rect_text_host_para_pages::text_host_para_cursor_rect_builds_few_page_trees`
- `issue_4128_cell_cursor_page_narrowing::deep_cell_cursor_queries_build_few_page_trees`

둘 다 `perf_counters` 의 page tree build 횟수 상한을 보는 테스트다. 이 호스트에
`cargo-nextest` 가 없어 묶음 suite 를 `cargo test` 로 **한 프로세스에서 병렬** 실행하면
다른 테스트의 빌드 횟수가 전역 카운터에 섞인다(측정값이 실행마다 13·18·27 로 흔들린다).
테스트 주석의 "파일당 테스트 1개" 전제가 generated suite 묶음에서 깨진 것으로, CI 의
nextest(프로세스 분리) 실행에서는 발생하지 않는다.

- 단독 실행: 둘 다 통과
- 수정을 원복한 baseline 에서 같은 명령: 둘 다 동일하게 실패 — 이 변경과 무관함을 확인

## 8. 재현 명령

```bash
# 증상/수정 확인 (부모 표 오른쪽 = 680.37px)
rhwp export-svg "<00387>.hwpx" -o out/
grep -o '<clipPath id="cell-clip-26[^/]*/>' out/00387.svg   # width 604.79 (수정 후)

# 회귀
node scripts/run-rust-test.mjs --cargo-test issue_5587_overwide_nested_table_clip -- \
  --profile release-test --target-dir target/pr-review
node scripts/run-rust-test.mjs --cargo-test issue_2007_nested_cell_pagination -- \
  --profile release-test --target-dir target/pr-review

# 한컴 정답지 재계산
mutool draw -F trace -o - samples/issue1994/issue_1994.pdf 1 | grep -E 'x="39[1-9]|x="40[0-2]'
```
