# PR #1240 검토 — 미주(해설) 다줄 문단 줄간격 간헐적 좁음 수정

- **작성일**: 2026-06-02
- **PR**: #1240 (OPEN)
- **제목**: `미주(해설) 다줄 문단 줄간격 간헐적 좁음 수정 (closes #1236)`
- **컨트리뷰터**: @planet6897
- **연결 이슈**: #1236
- **base/head**: `devel` ← `feature/issue-1236-endnote-line-spacing`
- **Head SHA**: `b6c9a4df7ec043be59d144b5909a993cabb2734f`
- **PR 기준 base SHA**: `f83c43b57ee4e9bf3e5ecb8be73f53a81f290430`
- **현재 local/devel**: `704f62d2` (#1235 반영 후)
- **규모**: 9 files, +422 / -1, 1 commit
- **mergeable**: true
- **PR 댓글**: 없음

## 1. PR 요약

PR #1240은 #1236에서 보고된 `samples/3-11월_실전_통합_2022.hwpx` 풀이/미주 페이지의 간헐적 줄간격 압축 문제를 수정한다.

핵심 증상은 미주 다줄 문단의 마지막 줄과 다음 문단 사이 간격이 `line_height`만큼만 내려가 정상적인 `line_height + line_spacing`보다 좁아지는 것이다.

컨트리뷰터의 수정 방향은 다음과 같다.

```text
다줄 미주 문단 마지막 줄의 trailing line_spacing을
"다음 렌더 문단이 같은 Endnote control 내부의 연속 문단"일 때만 복원한다.
```

## 2. 주요 변경 범위

| 영역 | 변경 |
|---|---|
| `src/renderer/layout.rs` | `endnote_para_has_same_endnote_successor()` 추가 |
| `src/renderer/layout/paragraph_layout.rs` | `endnote_line_vpos_base` 경로의 마지막 줄 trailing line spacing 조건 게이트 추가 |
| `mydocs/plans/task_m100_1236*.md` | 수행/구현 계획 문서 추가 |
| `mydocs/tech/endnote_line_spacing_1236.md` | 원인 조사 문서 추가 |
| `mydocs/working/task_m100_1236_stage*.md` | 단계별 작업 기록 추가 |
| `mydocs/report/task_m100_1236_report.md` | 최종 보고서 추가 |

## 3. 타당한 부분

### 3.1 증상 범위와 수정 범위가 좁다

이슈 #1236은 일반 본문 페이지가 아니라 미주/풀이 페이지에서 발생한다.
PR은 일반 문단 전체의 줄간격 계산을 바꾸지 않고, `endnote_line_vpos_base`가 활성화된 다줄 미주 문단 경로에만 영향을 준다.

이는 기존의 미주 페이지네이션 정밀 튜닝을 크게 흔들지 않으려는 방향이라 적절하다.

### 3.2 문제 경계와 같은 미주 내부 연속을 구분한다

무조건 마지막 줄 trailing line spacing을 더하면 문제-사이 margin이 중복 가산될 수 있다.
PR은 `endnote_para_sources`의 `(section_index, para_index, control_index)`를 비교해 다음 가상 미주 문단이 같은 원본 Endnote control 내부인지 확인한다.

따라서 같은 문제 풀이 내부 문단 사이에는 줄간격을 복원하고, 다른 문제 풀이로 넘어가는 경계는 기존처럼 0을 유지한다.

### 3.3 기존 고위험 회귀 지점을 인식한다

PR 설명은 issue_1139/1189 계열 회귀를 명시적으로 경계한다.
미주 영역은 페이지 수, 다단, 문제-사이 여백, 표/그림 배치와 강하게 결합되어 있으므로 이 관점은 중요하다.

## 4. 확인 필요 사항

### 4.1 PR base가 현재 devel보다 뒤처짐

PR 기준 base는 `f83c43b5`이고, 현재 `local/devel`은 `704f62d2`이다.

그 사이 #1232, #1234, #1235가 반영되었으므로 현재 devel 기준 검증 브랜치에서 병합해야 한다.
현재 패치 범위상 충돌 가능성은 높지 않지만, 미주 페이지네이션 테스트는 반드시 현재 devel 위에서 다시 실행해야 한다.

### 4.2 새 helper의 판정 축

`endnote_para_has_same_endnote_successor()`는 다음 가상 미주 문단이 같은 `(section, para, control)`에서 왔는지만 본다.
이 판정은 "같은 Endnote control 내부 연속 문단"을 잘 표현하지만, 다음 경우는 실측 검증이 필요하다.

```text
1. 같은 Endnote control 안의 마지막 빈 문단
2. inline object 이후 line vpos 보정이 걸리는 미주 문단
3. 다단 미주에서 column/page transition 직전 문단
```

PR 문서상 관련 회귀 테스트를 통과했다고 되어 있으나, 로컬에서 `issue_1139`, `issue_1189`, `issue_1082` 계열을 포함해 재검증해야 한다.

### 4.3 시각 판정 대상

자동 테스트가 통과해도 사용자가 보는 문제는 줄 간격 시각 정합이다.
최소한 다음 페이지는 SVG 및 가능하면 웹 캔버스에서 직접 판정해야 한다.

```text
samples/3-11월_실전_통합_2022.hwpx
- 10쪽 문8 주변
- 11쪽 문11 주변
- 12쪽 문15/문19 주변
- 14쪽 문22/문24 주변
```

## 5. 권장 검증

현재 devel 기준 검증 브랜치에서 다음을 실행한다.

```text
git diff --check HEAD
cargo fmt --all --check
cargo test --test issue_1139_inline_picture_duplicate
cargo test --test issue_1082_endnote_multicolumn_drift
cargo test --lib
cargo test --tests
docker compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
```

메인테이너 시각 판정용 SVG:

```text
target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 9 -o output/poc/pr1240-endnote-line-spacing
target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 10 -o output/poc/pr1240-endnote-line-spacing
target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 11 -o output/poc/pr1240-endnote-line-spacing
target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 13 -o output/poc/pr1240-endnote-line-spacing
```

필요하면 같은 페이지를 `--debug-overlay --show-grid=3mm`로 별도 산출한다.

## 6. 권장 처리

권장안: **수용 후보로 진행한다. 단, 현재 `local/devel` 기준 검증 브랜치에서 병합하고, 미주/페이지네이션 회귀 테스트와 메인테이너 시각 판정을 게이트로 둔다.**

코드 변경은 좁고 문제 정의와 잘 맞는다.
다만 미주 줄간격은 과거에도 여러 번 회귀가 발생한 고위험 영역이므로, 단순 `cargo test --lib`만으로 종료하지 않고 `issue_1139`, `issue_1189`, `issue_1082` 계열 및 실제 지적 페이지 시각 판정을 함께 통과시켜야 한다.

## 7. 다음 승인 요청

다음 단계로 진행하려면 작업지시자 승인이 필요하다.

권장 절차:

```text
1. `local/pr1240-verify` 브랜치를 현재 `local/devel`에서 생성
2. PR #1240을 병합 시뮬레이션
3. 미주/페이지네이션 중심 테스트와 전체 테스트 실행
4. WASM/Studio 빌드
5. 지적 페이지 SVG 및 웹 canvas 시각 판정 후 local/devel 반영
```
