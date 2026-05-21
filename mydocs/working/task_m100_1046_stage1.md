# Stage 1 완료보고서 — #1046: overflow 16건 분류 + 키 확정

- 타스크: #1046 (M100), 브랜치 `local/task1046`
- 단계: Stage 1 (코드 변경 전 진단)
- 작성일: 2026-05-21
- 대상: `samples/2. 인공지능(AI) ... 제안요청서.hwpx` (비공개, 185p, **2섹션**)

## 1. 핵심 발견

### (a) overflow 로그 인덱스는 "섹션-로컬"
overflow 레코드의 `page_index`/`para_index`는 **섹션-로컬**이다. 문서는 2섹션:
- 섹션 0: 글로벌 page 0~111, body bottom **1046.9** (y=105.8 h=941.1).
- 섹션 1: 글로벌 page 112~184, body bottom **1026.7** (y=113.4 h=913.3, 여백 상이).

→ 섹션 0 overflow는 글로벌 `dump-pages -p N`과 일치(12건). 섹션 1 overflow(4건)는
`page=24/39/40/44`가 **섹션1-로컬**이라 글로벌 page 136/151/152/156에 해당해 dump-pages
글로벌 표기와 어긋나 보였다(체계 차이일 뿐, 페이지네이션은 동일 엔진·동일 결과).

### (b) "page-larger" 판정의 견고한 신호 = is_first_in_column
overflow 항목이 **단의 첫 항목**이면 다음 페이지로 이월해도 또 넘침(본문보다 큰 단일
항목) → 이월 무의미. 첫 항목이 아니면(위에 다른 항목 존재) 이월로 해소 가능. 이 신호를
`page_index` 매칭으로 외부 재구성하면 섹션-로컬/글로벌 혼선이 생기므로, **overflow 기록
시점**(`layout.rs:2257` 항목 루프 + `2539` 기록부)에서 `section_index`(=
`page_content.section_index`)와 `is_first_in_column`(루프 `enumerate` idx==0)을 **직접
기록**한다. → Stage 2 설계 반영.

## 2. 16건 분류 (예비 — Stage 3 런타임 is_first로 확정)

### 섹션 0 (12건, bottom 1046.9)
| 글로벌 page | pi | type | overflow | 위 항목 | 분류 |
|---|---|---|---|---|---|
| 20 | 218 | PartialTable | 2.2 | 216,217 | 이월 대상 |
| 27 | 242 | PartialTable | **19.2** | 238~241 | 이월 대상 (**SIR-002**) |
| 34 | 256 | PartialTable | 6.9 | 253~255 | 이월 대상 |
| 39 | 266 | Table | 7.2 | 263~265 | 이월 대상 |
| 48 | 290 | PartialTable | 8.7 | 286~289 | 이월 대상 |
| 53 | 308 | Table | 11.6 | 305~307 | 이월 대상 |
| 61 | 323 | PartialTable | 29.1 | (단독, items=1) | **page-larger** (범위 외) |
| 71 | 361 | PartialParagraph | 4.3 | 346~360 | 이월 대상 |
| 78 | 429 | FullParagraph | 10.7 | 427,428 | 이월 대상 |
| 92 | 567 | PartialParagraph | **856.7** | 566 + 자기표 1797px | **page-larger**(nested, 범위 외) |
| 95 | 600 | PartialTable | 2.7 | 592~599 | 이월 대상 |
| 111 | 781 | FullParagraph | 15.8 | 757~780 | 이월 대상 |

### 섹션 1 (4건, bottom 1026.7) — 섹션1-로컬 page
| 로컬 page (글로벌) | pi | type | overflow | 분류 |
|---|---|---|---|---|
| 24 (136) | 268 | PartialParagraph | 12.3 | 런타임 is_first로 확정 (이월 대상 추정) |
| 39 (151) | 354 | Table | 8.3 | 런타임 확정 (이월 대상 추정) |
| 40 (152) | 357 | Table | 10.0 | 런타임 확정 (이월 대상 추정) |
| 44 (156) | 406 | FullParagraph | 3.1 | 런타임 확정 (이월 대상 추정) |

## 3. 예상 결과
- **이월 대상 ~10건(섹션0) + 섹션1 4건(추정)** = 해소 가능 후보 ~14건.
- **page-larger 2건(pi=323, pi=567)** = 범위 외 잔존(별도 이슈: 내부 분할).
- 사용자 보고 28·35·49쪽(pi=242·256·290)은 모두 이월 대상 ✓.

## 4. Stage 2 설계 확정 (반영 사항)
1. `LayoutOverflow`에 `section_index: usize`, `is_first_in_column: bool` 필드 추가.
   기록부(`layout.rs:2539`)에서 `page_content.section_index`와 루프 enumerate idx로 채움.
2. 항목 루프 `for item in col_content.items.iter()` → `.enumerate()` 로 순번 확보.
3. force-break hint 키 = `(section_index, para_index)`.

## 5. 코드 변경
없음(진단 단계). 본 보고서만 커밋.
