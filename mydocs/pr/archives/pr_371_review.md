# PR #371 검토 — Task #370: 한글 세리프 폴백 체인 개선 (macOS/Linux 볼드 표시)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#371](https://github.com/edwardkim/rhwp/pull/371) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| base / head | `devel` ← `local/task370` |
| state | OPEN |
| mergeable | **CONFLICTING** (DIRTY) — devel 과 충돌 |
| 이슈 | [#370](https://github.com/edwardkim/rhwp/issues/370) |
| 변경 통계 | +816 / -606, 11 files |

## 결함 요약

`form-002.hwpx` 10쪽 등의 볼드 한글 문구가 SVG 출력에서 볼드로 표시 안 됨.

원인: SVG 의 `font-weight="bold"` 속성은 정상 적용되지만, 한글 세리프 폴백 체인이 환경별 한계로 Bold variant 매칭 실패:
- **Linux**: `'Batang','바탕','AppleMyungjo','Noto Serif KR',serif` — 모두 표준 환경에 미설치
- **macOS Chrome**: `AppleMyungjo` 매칭 후 Regular variant 만 보유 → Bold 합성 실패

## 변경 내용

`src/renderer/mod.rs::generic_fallback()` 의 한글/영문 세리프 분기 두 곳:

```diff
- 'Batang','바탕','AppleMyungjo','Noto Serif KR',serif
+ 'Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR',serif
```

| 추가 폰트 | 위치 | 효과 |
|---|---|---|
| `'Nanum Myeongjo'` | `'AppleMyungjo'` 앞 | macOS 10.9+ 기본 설치 + **실제 Bold variant 보유** |
| `'Noto Serif CJK KR'` | `'Noto Serif KR'` 뒤 | Linux 표준 환경 (`noto-cjk` 패키지) |

## 메모리 점검 — feedback_font_alias_sync 와의 관계

**참고 메모리**: 한글 폰트 추가 시 `style_resolver` (Layer 1) + `font_metrics_data::resolve_metric_alias` (Layer 2) 모두 동기화 필수.

본 PR 의 변경은 **CSS `font-family` 폴백 체인 (= SVG 출력의 brower 가 OS 폰트 매칭에 사용)** 이지, rhwp 내부의 metric lookup 경로가 아님.

- **`generic_fallback`**: SVG/HTML 의 `font-family` 속성 문자열 — 브라우저 폰트 매칭용
- **`resolve_metric_alias`**: rhwp 내부 텍스트 폭 계산용 (FONT_METRICS lookup)

→ 두 경로 분리되어 있으며 본 PR 은 Layer 1/2 와 무관. 메모리의 동기화 의무는 본 PR 에 적용되지 않음.

다만 **확인 필요 항목**:
- "Nanum Myeongjo" 가 입력 폰트 패밀리로 사용될 때 (= HWP 파일이 이 이름을 명시적으로 가짐) metric lookup 이 정상인지
- 현재 `font_metrics_data.rs:91` 에 `"나눔명조" => "NanumMyeongjo"` 매핑은 있음 (한글)
- 영문 `"Nanum Myeongjo"` (공백) 의 별칭은 없으나, **본 PR 의 변경은 SVG 출력 시 fallback 체인** 이라 rhwp 가 이 이름을 입력으로 받는 일이 없음 → 메트릭 매핑 추가 불필요

→ **메모리 동기화 의무 위반 없음**.

## 변경 평가

### 강점
1. **결함 진단 정확**: SVG `font-weight=bold` 속성 정상 + 환경별 폰트 매칭 한계 식별
2. **Trade-off 합리적**: macOS 의 default face 가 AppleMyungjo → Nanum Myeongjo 변경되지만 둘 다 명조 계열, 사용자 지정 폰트 미설치 fallback 한정
3. **Linux 표준 환경 지원**: `Noto Serif CJK KR` 추가 (noto-cjk 패키지)
4. **테스트**: cargo test 1055 passed + 골든 SVG 3건 갱신 (정확한 영향만)
5. **PDF 영향 없음**: pdf.rs 의 별도 경로 사용 명시
6. **HTML 영향 동일**: `generic_fallback` 공유

### 약점 / 점검 필요
1. **mergeStateStatus = DIRTY** (CONFLICTING) — devel 과 충돌 (rebase 필요)
2. **macOS face 변경**: AppleMyungjo → Nanum Myeongjo 가 default 로 매칭되는 사용자가 있음. 시각 차이 가능성 (둘 다 명조 계열이라 큰 차이 없으나 작업지시자 시각 검토 권장)
3. **체인 길이 증가**: 5개 → 7개. 기존 환경에서는 후순위에 추가됨 (Windows Batang 우선 매칭) → 영향 없음

### 충돌 가능성

devel 의 현재 상태:
- v0.7.7 릴리즈 후 PR #366 흡수 (page_number)
- mod.rs 는 `pub mod page_number;` 추가됨 (PR #366 흡수 commit)

PR #371 이 mod.rs 를 변경 — 두 변경이 다른 위치이지만 같은 파일 → 자동 머지 가능할 수도. CONFLICTING 표시는 다른 commit history 차이일 수 있음.

## 처리 방향

### 옵션 A: 정상 PR 머지 (rebase + 충돌 해결 + 머지)

근거:
- 결함 진단 정확
- 변경 범위 좁고 명확
- 테스트 + 시각 검증 통과
- 영향 범위 작음 (CSS 폴백만, rhwp 내부 무관)

처리:
1. PR rebase devel
2. 충돌 해결 (mod.rs 의 다른 변경 부분 통합)
3. cargo test 재검증
4. 머지

### 옵션 B: 메인테이너 흡수 (PR #366 처럼)

근거: 충돌이 있고 PR 의 stage 보고서 등 문서가 메인테이너 task #361 / #362 와 다른 prefix (`task_m100_370`).

처리:
- 코드 변경 + 골든 SVG 만 체리픽
- 문서는 그대로 두거나 메인테이너가 정리

## 권장

**옵션 A** — 본 PR 은 정상 PR 머지. 이유:
- 변경 범위 좁고 명확 (mod.rs 한 함수의 두 줄 + 골든 SVG 갱신)
- 작성자 stage 보고서 + 구현계획서가 정상 절차로 작성됨 (Task #370)
- 결함 정정이 명확하고 메인테이너 동시 정정 없음
- 충돌 자동 해결 가능성 높음 (다른 위치)

PR #366 처럼 메인테이너가 동시 정정한 경우와 달리, 본 PR 은 단독 결함이라 정상 머지가 자연스러움.

## 다음 단계 — 작업지시자 결정

옵션 A (정상 머지) 또는 옵션 B (메인테이너 흡수) 중 선택 부탁드립니다.

옵션 A 선택 시:
1. PR rebase devel + 충돌 해결
2. cargo test + svg_snapshot 검증
3. devel merge + push (작성자 attribution 보존)
4. PR close (자동) + 이슈 #370 close
5. `pr_371_report.md` 작성

## 검토 항목 (Claude 점검 완료)

- [x] 결함 진단 정확성: SVG font-weight 속성 정상 + 환경별 매칭 한계 ✅
- [x] PR 의 코드 변경 범위 (`generic_fallback` 두 분기): 작고 정확 ✅
- [x] 영향 범위 (HTML 공유, PDF 무관): 정확 ✅
- [x] 메모리 (font_alias_sync) 점검: 본 PR 의 CSS 폴백은 metric lookup 경로와 분리, 동기화 의무 위반 없음 ✅
- [x] 메인테이너 동시 정정 여부: 없음 ✅
- [ ] devel rebase + 자동 충돌 해결 가능성: rebase 시도 시 확인
- [ ] cargo test + svg_snapshot 통과 여부: rebase 후 재검증

## 참고

- 이슈: [#370](https://github.com/edwardkim/rhwp/issues/370) (OPEN)
- PR: [#371](https://github.com/edwardkim/rhwp/pull/371) (OPEN, DIRTY)
- 메모리: `feedback_font_alias_sync.md` (본 PR 에 적용 안 됨)
