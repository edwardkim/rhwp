# PR #434 검토 — Task #430 그림 자동 크롭 (FitToSize+crop) 공식 교정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#434](https://github.com/edwardkim/rhwp/pull/434) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 신뢰 컨트리뷰터 |
| 이슈 | [#430](https://github.com/edwardkim/rhwp/issues/430) (closes) |
| base / head | `devel` ← `planet6897:local/task430` |
| 변경 규모 | +783 / -26, 16 files (5 commits) |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-29 |

## 본질

`samples/exam_kor.hwp` 1쪽 상단 헤더 이미지의 자동 크롭 공식이 잘못됨 → src 영역이 항상 이미지 전체와 일치 (crop 무력화) → 정정.

### 잘못된 공식 (수정 전)

```rust
let scale_x = cr as f64 / img_w;          // 스케일을 crop 우경계로부터 추정
let src_w  = (cr - cl) as f64 / scale_x;  // 결과: cr/(cr/img_w) = img_w (항상 전체 폭)
```

### 올바른 공식 (수정 후)

```rust
let scale_x = original_width_hu  / img_w_px;   // 진짜 HU/px 스케일
let src_x = cl as f64 / scale_x;
let src_w = (cr - cl) as f64 / scale_x;
```

## 5 commits 정황

| commit | 내용 |
|--------|------|
| `060bd5f` | Stage 1: 그림 crop 미적용 원인 정밀 조사 |
| `ead4a43` | Stage 2: ImageNode.original_size_hu + svg.rs crop 공식 교정 |
| `09f5d0c` | Stage 3: web_canvas.rs 동기화 + 회귀 검증 |
| `dfb685d` | Stage 4: 최종 보고서 + 오늘할일 갱신 |
| `add226a` | **추가 fix**: 테두리 문단 inner padding (별도 결함, 작성자 명시) |

## 처리 방향

**옵션 A — 5 commits 모두 cherry-pick** (작업지시자 결정).

본 PR 의 5번째 commit 은 별도 결함 (테두리 문단 inner padding) 정정이지만 작성자가 본 브랜치에 함께 포함하여 합리적이라 함께 흡수.

## dry-run cherry-pick 결과

`local/pr434` 브랜치 (`local/devel` 분기) 에서 5 commits cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `bdee30d` (← `060bd5f`) | @planet6897 | Stage 1: 그림 crop 미적용 원인 정밀 조사 |
| `a5541f9` (← `ead4a43`) | @planet6897 | Stage 2: ImageNode.original_size_hu + svg.rs crop 공식 교정 |
| `401f3c5` (← `09f5d0c`) | @planet6897 | Stage 3: web_canvas.rs 동기화 + 회귀 검증 |
| `97d18db` (← `dfb685d`) | @planet6897 | Stage 4: 최종 보고서 + 오늘할일 갱신 |
| `84c8a27` (← `add226a`) | @planet6897 | fix: 테두리 문단 inner padding |

cherry-pick 결과: 충돌 없이 자동 적용.

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1066 passed** (PR #427 1062 → +4 신규 단위 테스트) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 44s, 4,182,395 bytes |

## 광범위 회귀 byte 단위 비교

10 샘플 / 309 페이지 SVG 비교 (devel baseline ↔ PR #434 적용):

| 결과 | 카운트 |
|------|------|
| byte 동일 | 269 / 309 |
| 차이 발생 (의도된 정정 영향) | **40 / 309** |

차이 분포:
- **exam_kor 22 페이지** (이슈 #430 자동 크롭 정정 + 테두리 inner padding)
- **exam_eng 8 페이지** (테두리 inner padding)
- **2025년 기부 6 페이지** (테두리 inner padding)
- **synam-001 3 페이지**
- **k-water-rfp 1 페이지**

## 한컴 PDF 환경 의존성 정황 발견 (검토 중)

본 PR 검토 중 작업지시자가 추가 자료 3종 제공:

| 자료 | 한컴 환경 |
|------|----------|
| `samples/2010-exam_kor.pdf` | 한컴 2010 (4.57 MB) |
| `samples/2020-exam_kor.pdf` | 한컴 2020 (4.57 MB) |
| `samples/hancomdocs-exam_kor.pdf` | 한컴독스 (6.05 MB) |

작업지시자 코멘트:
> "이 예제 파일을 한컴의 2010 버전과 2020 버전 윈도우 프로그램에서 조차 다르게 조판한다는 사실. 심지어 우리가 조판하는 것이 오히려 더 이 시험지를 조판한 사람의 의도에 맞다는 생각이 들 정도입니다."

### 후속 조치 (본 검토 사이클에 포함)

1. **3종 PDF samples/ commit + push** (메인테이너) — 모든 컨트리뷰터 공유
2. **위키 페이지 [한컴 PDF 환경 의존성](https://github.com/edwardkim/rhwp/wiki/한컴-PDF-환경-의존성) 보강** — "발견 정황 II (PR #434 / 이슈 #430)" 섹션 추가
3. **README.md / README_EN.md 에 위키 링크 추가** — Contributing 섹션에 "한컴 PDF 는 정답지가 아닙니다" 항목 + 신규 "위키 자료 (Wiki Resources)" 서브섹션

## 시각 판정 (작업지시자 직접)

**한컴 3종 PDF (한컴 2010 / 2020 / 한컴독스) vs rhwp 출력** 비교 — SVG + Canvas 양 경로 작업지시자 직접 판정.

| 경로 | 시각 판정 결과 |
|------|--------------|
| SVG + Canvas | ✅ **통과** (옵션 3 — 한컴 3종 PDF 와 광범위 회귀 직접 점검 후 머지 결정) |

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1066 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 (push 전 필수) | ✅ 작업지시자 직접 한컴 3종 PDF 비교 후 통과 |
| `feedback_pdf_not_authoritative.md` | ✅ 본 PR 검토 중 한컴 PDF 환경 의존성 II 발견 — 위키 페이지에 정황 정리 + README 안내 추가 |
| `reference_authoritative_hancom.md` | ✅ 한컴 2010 + 2022 정답지 정책 재확인 (단 본 케이스는 한컴 2010/2020/한컴독스 차이 발견) |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr434` 에서 커밋 |
| 이슈 close 시 commit devel 머지 검증 | ✅ 머지 + push 후 close |

## 다음 단계

1. 본 보고서 + 오늘할일 갱신 commit
2. `local/pr434` → `local/devel` → `devel` 머지 + push
3. PR #434 close + 작성자 댓글 (이슈 #430 자동 close)

## 참고

- PR: [#434](https://github.com/edwardkim/rhwp/pull/434)
- 이슈: [#430](https://github.com/edwardkim/rhwp/issues/430)
- 위키 페이지: [한컴 PDF 환경 의존성](https://github.com/edwardkim/rhwp/wiki/한컴-PDF-환경-의존성) (발견 정황 II 보강)
- 같은 작성자 머지 PR: [#401 v2](https://github.com/edwardkim/rhwp/pull/401), [#406](https://github.com/edwardkim/rhwp/pull/406), [#408](https://github.com/edwardkim/rhwp/pull/408), [#410](https://github.com/edwardkim/rhwp/pull/410), [#415](https://github.com/edwardkim/rhwp/pull/415), [#424](https://github.com/edwardkim/rhwp/pull/424)
