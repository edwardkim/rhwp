# PR #1650 처리 보고서 — README 브라우저 확장 스토어 배지 추가 (#1646)

- PR: https://github.com/edwardkim/rhwp/pull/1650
- 제목: `README에 브라우저 확장 스토어 배지 추가`
- 작성자: postmelee (collaborator, 30건/20 merged)
- 연결: Refs #1646
- base ← head: `devel` ← `postmelee:local/task1646`
- 처리일: 2026-06-29

## 1. 처리 결정

**admin merge.** 루트 `README.md` 상단에 Chrome/Edge/Firefox 스토어 배지를 추가하는 문서 전용
PR. 충돌 0 + CI 전부 pass + 배지/스토어 링크 실측 정상 + 기존 배지 무훼손 + 자기검열 통과.

## 2. 변경 범위 (6 files +248/-0, 문서 전용)

| 파일 | 내용 |
|---|---|
| `README.md` | 기존 배지줄 아래 별도 가운데 정렬 줄에 스토어 배지 3개 추가(순수 +6줄, 삭제 0) |
| `mydocs/{orders,plans,working,report}/...1646*` | 계획/단계/보고서 5건 |

## 3. 배지/링크 실측 검증

| 배지 | 스토어 링크 | 동적 배지 |
|---|---|---|
| Chrome | 301→200 (실제 확장 "rhwp HWP 문서 뷰어/에디터", id `pgakpjfl…`) | `chrome-web-store/v` → 0.2 정상 |
| Edge | 200 (게시됨, id `nfkdfobh…`) | 정적 배지(`Edge Add--ons \| Store`) — 안정성 목적 |
| Firefox | 301→200 (AMO 게시, `rhwp-free-hwp-editor`) | `amo/v` → 0.2 정상 |

3개 스토어 모두 실제 게시된 확장을 가리키고 동적 배지(Chrome/Firefox)가 버전을 반환한다.
깨진 배지·잘못된 링크 없음.

## 4. 검증

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas) | 전부 pass |
| 충돌 | 0건 |
| src/code 변경 | 0건 (문서 전용) |
| 기존 배지줄 보존 | 삭제 0줄 — 순수 추가 |
| 배지/스토어 링크 실측 | 3개 전부 정상 |
| 자기검열 (과장/최상급/공공기관 오인) | 없음 |

## 5. 메모

- postmelee 직전 #1640(확장 hover card) 연속. 확장 스토어 배포(0.2.8) 가시화용 문서 보강.
- Edge를 정적 배지로 둔 것은 합리적(Shields.io Edge 동적 배지가 불안정) — 본문에 사유 명시.

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1650_review.md`
