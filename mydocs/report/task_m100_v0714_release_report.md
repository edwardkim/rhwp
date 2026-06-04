# v0.7.14 PATCH 릴리즈 결과 보고서

- 일자: 2026-06-05
- 방식: 1 방식 (devel→main PR/머지 + 태그)
- 기준: v0.7.13 (`ba87d20e`, 태그 시점) → v0.7.14

## 1. 결정

PATCH 릴리즈. 미주(해설) 흐름·간격 정합, 수식 렌더링/배치 정밀화, 표 셀 안 그림 편집(삽입·복사·
hit-test) 한컴 정합, HWPX 저장 계약 확장, 외부 기여자 PR 다수 흡수를 포함한다. 공개 API 하위
호환을 깨지 않는 버그 수정·정합 범위이므로 PATCH 로 진행한다.

## 2. 버전 동기화 (0.7.13 → 0.7.14)

| 파일 | 결과 |
|------|------|
| `Cargo.toml` | ✓ |
| `Cargo.lock` (rhwp entry) | ✓ |
| `rhwp-studio/package.json` | ✓ |
| `rhwp-studio/package-lock.json` (2곳) | ✓ |
| `rhwp-vscode/package.json` | ✓ |
| `rhwp-vscode/package-lock.json` (2곳) | ✓ |
| `npm/editor/package.json` | ✓ |

브라우저 확장(rhwp-chrome/firefox)은 이번 사이클에 변경 없음 → `0.2.3` 유지(배포 제외).
0.7.13 잔존 문자열 0 확인.

## 3. CHANGELOG

- `CHANGELOG.md` — `## [0.7.14] — 2026-06-05`
- `CHANGELOG_EN.md` — `## [0.7.14] — 2026-06-05`
- `rhwp-vscode/CHANGELOG.md` — `## [0.7.14] - 2026-06-05`

## 4. 주요 포함 범위 (v0.7.13 이후, 머지 PR 41건 + 메인테이너 task/통합)

- **미주(Endnote) 흐름·간격**: compact 미주 제목 사이 간격(7mm), 다줄 줄간격, 연속 인라인 수식
  다행 병합, 구분선 아래 여백, 다단 흐름 단 끝/오버플로우 (#1240/#1241/#1247/#1255/#1259/#1262,
  task #1245/#1248/#1256/#1257/#1258)
- **수식**: root/sqrt·관계연산자·prime·cdots glued-split (#1208), LEFT-RIGHT 그룹 첨자 결합 (#1226),
  큰 연산자 간격 (#1235), 수식 줄 한글 압축 해소 (#1223), prefixChar 마커 '문' (#1202)
- **표 셀 그림**: 삽입/토글/시각/클릭 (#1177), 셀 내부 도형 속성 (#1150), 중첩 셀 복사+cascade (#1228),
  글상자 picture hit-test (#1254), 중첩 셀 붙여넣기 경로 (#1207), wrap=Square 커서/z-표 (#1220/#1225)
- **레이아웃·렌더링**: curve `<hp:seg>` 외곽선 (#1203), textFlow roundtrip (#1213), z-order 합성
  (#1163/#1252), 회전 이미지 bbox (#1102), RawSvg 백지 (#1182), 폰트 폴백 (#1234), 격자/잔상 (#1137/#1164)
- **HWPX 저장**: Bookmark/Field/OLE chart/회전 그림/맞쪽 여백/masterpage idRef (#1289/#1242),
  문단 id 전역 유니크 (#1222), external image contract (#1142/#1143)
- **rhwp-studio**: 입력 재렌더 비용 축소 (#1212), 모달 드래그 공통화, mac 리사이즈/대화상자 Enter/
  hit-test caret (#1193/#1281/#1291)
- **인프라**: Dependabot 그룹화 + dev-dep bump (#1214/#1216), macOS Skia hang (task #823),
  테스트 경고 정리 (#1180), ClickHere 필드 손상 (#1076)

## 5. 자기 검증

| 항목 | 결과 |
|------|------|
| `cargo fmt --all -- --check` | 통과 |
| `cargo build` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 (hit_test_leading_gap doc 주석 lint 1건 해소) |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |
| `cargo nextest run --no-fail-fast` | **2036 passed, 0 failed** (21 skipped, 147.9s) |
| `docker compose ... wasm` | (검증 진행) |
| `rhwp-studio npm run build` | (검증 진행) |

## 6. 기여자 (누락 방지 — .mailmap 정규화 선행)

`.mailmap` 보강으로 커밋 author ↔ GitHub PR 계정 불일치를 비파괴 매핑(history 재작성 없음):
- 미연결 정정: @planet6897(Jaeook Ryu), @twoLoop-40(joonho), @lidge-jun(bitkyc08-arch),
  @humdrum00001010(phi hu)
- 동일인 통합: @jangster77, @postmelee(Taegyu Lee), @johndoekim, @Martinel2(Kim J.H), @Mireutale,
  @seo-rii, @oksure, @chkwon, @wonbbnote, @edwardkim(메인테이너)

**v0.7.14 외부 기여자 15명**:
@planet6897, @postmelee, @jangster77, @johndoekim, @Martinel2, @Mireutale, @chkwon, @oksure,
@seo-rii, @xogh3198, @twoLoop-40, @lidge-jun, @humdrum00001010, @HaimLee-4869, @wonbbnote
(+ Dependabot)

## 7. 릴리즈 절차

1. `local/devel`에서 .mailmap 정규화(`0dec863a`) → 버전 bump + CHANGELOG 3종 + 본 보고서
2. 로컬 검증(fmt/build/clippy/wasm check/nextest/WASM Docker/studio) 후 릴리즈 커밋
3. `devel`에 `local/devel` 반영 + `origin/devel` push
4. `devel` → `main` 머지 (main 동기화 점검 완료 — 분기 없음, ff 가능)
5. `main` push + `v0.7.14` 태그 생성/push

## 8. 비고

- main 동기화 점검: 로컬 main = origin/main(`ce45231c`), devel→main ff 가능(main 8커밋 전부 devel 반영).
- 가장 느린 테스트: `serializer::cfb_writer::tests::test_serialize_real_hwp_files` (~140s, 단독 병목) — CI 분할 후속 검토.
