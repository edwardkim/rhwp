---
문서: 열린 이슈 37건 우선순위 지정 피드백 폼
작성일: 2026-04-22
기준: 2026-04-22 03:30 KST origin/devel
목적: 작업지시자가 우선순위(P0~P3) 와 배치 사이클을 기록할 수 있도록 정리
---

# 읽는 법

각 이슈 블록 마지막 `**우선순위**: [ ]` 에 다음 중 하나를 기록해 주세요.

| 코드 | 의미 |
|---|---|
| **P0** | 즉시 (이번 사이클 안에 반드시) |
| **P1** | 다음 사이클 (1~2주) |
| **P2** | v1.0.0 전까지 |
| **P3** | v1.0.0 이후 |
| **iOS** | 알한글 프로젝트 (맥 환경 전용, 현재 환경 작업 불가) |
| **보류** | 장기 대기 / 재고 |
| **close** | 이슈 정리 차원에서 종료 권장 |

`**배치**: [ ]` 에는 선택적으로 다음 기록:
- **self** : 작업지시자 본인 처리
- **contrib-call** : 외부 기여 공고 (good first issue / help wanted)
- **skia-전후** : PR #165 skia 머지 이후에 하는 게 자연스러운 작업
- 또는 **자유 지시**

한 번에 채워서 보내주시면 이슈 분류 + 마일스톤 정리 + 다음 사이클 작업 준비 진행합니다.

---

# 통계 요약

- **총 37건 (열린 이슈)**
- **v1.0.0 마일스톤**: 13건 (모두 edwardkim 등록)
- **iOS 마일스톤 (M2~M5)**: 7건 (알한글, 맥 환경)
- **마일스톤 미지정**: 17건 (외부 기여자 신규 보고 다수)

**외부 기여자 보고 이슈** (14건):
@SBKIM9704, @koolerjaebee, @seunghan91(×4), @seagua1, @janghi1, @channprj, @InsuJeong496, @haseo-ai, @DanMeon, @planet6897, @studysnack

---

# A. 버그 — 외부 사용자 보고 (우선 검토 가치 높음)

## [#157](https://github.com/edwardkim/rhwp/issues/157) · 2026-04-16 · @SBKIM9704 · bug

**본문 번호 목록과 표가 같은 y 구간에 중첩 배치됨 — 레이아웃 엔진 오류 (SVG/Canvas 공통)**

주주총회 참석장 2페이지에서 번호 목록(위임 사항)과 그 뒤 표(대리인 정보)가 겹쳐 렌더링. 레이아웃 엔진 레벨.

- **우선순위**: [ ]
- **배치**: [ ]

## [#158](https://github.com/edwardkim/rhwp/issues/158) · 2026-04-16 · @koolerjaebee · bug

**[Bug] 되돌리기 다시실행 기능에 표 크기 조절이 포함되지 않는 현상**

표 크기 조절이 Undo/Redo 스택에서 빠짐. #204 (표 편집 Undo 전반) 와 연관, 동시 처리 권장.

- **우선순위**: [ ]
- **배치**: [ ]

## [#162](https://github.com/edwardkim/rhwp/issues/162) · 2026-04-16 · @seunghan91

**도형 리사이즈 시 width=0 전송으로 도형이 사라지는 문제 (Rectangle·Group)**

MDM 데스크톱 앱 사용자 보고. 반대편 너머로 여러 번 드래그 시 width=0 전송되어 도형 사라짐. 클램프 미적용 케이스 추가 발견 (#153 후속).

- **우선순위**: [ ]
- **배치**: [ ]

## [#218](https://github.com/edwardkim/rhwp/issues/218) · 2026-04-20 · @seagua1 · bug

**시험지 양식 깨짐**

웹앱에 HWP 업로드 시 시험지 양식이 깨짐. 사용자 보고 — 구체 재현 샘플 필요.

- **우선순위**: [ ]
- **배치**: [ ]

## [#229](https://github.com/edwardkim/rhwp/issues/229) · 2026-04-21 · @planet6897 · bug

**표 셀 내 긴 숫자 텍스트가 음수 자간으로 인해 글자 겹침 및 셀 폭 미사용 현상**

예: "65,063,026,600" + letter_spacing=-24% → 글자 겹침 + 셀 폭 부족 처리 미흡. PR #221 기여자의 후속 보고.

- **우선순위**: [ ]
- **배치**: [ ]

## [#231](https://github.com/edwardkim/rhwp/issues/231) · 2026-04-21 · @studysnack · bug

**표 선택 시 가이드라인 점선 침범**

표 선택 시 빨간색 가이드 점선이 문서 영역 밑을 침범. UI 레이어 이슈.

- **우선순위**: [ ]
- **배치**: [ ]

---

# B. 버그 — 내부 등록 (v1.0.0 마일스톤)

## [#77](https://github.com/edwardkim/rhwp/issues/77) · 2026-04-07 · @edwardkim · 마일스톤 미지정

**SVG 렌더러: 특정 페이지에서 탭 리더/페이지 번호 누락**

Canvas 는 정상, SVG 는 누락 케이스. 렌더러 간 정합성.

- **우선순위**: [ ]
- **배치**: [ ]

## [#103](https://github.com/edwardkim/rhwp/issues/103) · 2026-04-11 · @edwardkim · v1.0.0

**비-TAC wrap=위아래 표의 out-of-flow 레이아웃 처리**

hwpspec 93페이지 · 앵커 문단에 비-TAC 표 + TAC 표 공존. 한컴 동작 재현 필요.

- **우선순위**: [ ]
- **배치**: [ ]

## [#146](https://github.com/edwardkim/rhwp/issues/146) · 2026-04-14 → 2026-04-20 · @edwardkim · v1.0.0 · comments=3

**원문자(③) 위에 ⓪이 겹쳐 렌더링되는 버그**

원문자 ③ 정상, 문항 답번호 ⓪ 동일 좌표 겹침. 최근 댓글 활동 있음.

- **우선순위**: [ ]
- **배치**: [ ]

## [#147](https://github.com/edwardkim/rhwp/issues/147) · 2026-04-15 · @edwardkim · v1.0.0

**메모 컨트롤이 바탕쪽으로 잘못 파싱되어 렌더링되는 버그**

aift.hwp 4페이지. 한컴에서는 없는 바탕쪽이 rhwp 에서 나타남. 메모 컨트롤 파싱 분기 오류.

- **우선순위**: [ ]
- **배치**: [ ]

## [#174](https://github.com/edwardkim/rhwp/issues/174) · 2026-04-17 · @edwardkim · v1.0.0

**수식 TAC 높이가 줄 높이에 미반영 — 큰 수식과 텍스트 겹침**

exam_math 16/20페이지. ∑/분수/lim 등 큰 수식 줄과 인접 줄 겹침. 줄 높이 계산에 TAC 수식 높이 반영 필요.

- **우선순위**: [ ]
- **배치**: [ ]

## [#175](https://github.com/edwardkim/rhwp/issues/175) · 2026-04-17 · @edwardkim · v1.0.0

**CASES+EQALIGN 한글 혼합 수식 오버래핑**

exam_math 8페이지. `a_{n+1} = cases{...}` 한글+수식 혼합 긴 조건 겹침. 지난 v0.2.1 사이클 이월분.

- **우선순위**: [ ]
- **배치**: [ ]

## [#150](https://github.com/edwardkim/rhwp/issues/150) · 2026-04-15 · @edwardkim · v1.0.0

**그림 효과: 밝기/대비/워터마크 속성 SVG 반영**

PR #149 후속. 그레이스케일/흑백은 됐으나 밝기/대비/워터마크 미구현.

- **우선순위**: [ ]
- **배치**: [ ]

## [#204](https://github.com/edwardkim/rhwp/issues/204) · 2026-04-19 · @edwardkim · v1.0.0

**표 편집(삭제/생성/행열 삽입·삭제/셀 분할·병합) Undo/Redo 미동작**

지난 v0.2.1 사이클 분석 후 등록. 8건 누락 Command. #158 과 연관, 함께 처리 권장.

- **우선순위**: [ ]
- **배치**: [ ]

---

# C. 기능 제안 · 논의 (외부 기여자)

## [#220](https://github.com/edwardkim/rhwp/issues/220) · 2026-04-20 · @janghi1

**F5 블록 선택 모드 + F3 영역 확장 선택 기능 추가**

한컴 F5/F3 단축키 구현 요청. 에디터 상호작용 기능.

- **우선순위**: [ ]
- **배치**: [ ]

## [#223](https://github.com/edwardkim/rhwp/issues/223) · 2026-04-20 · @channprj · enhancement

**macOS 단축키 지원**

Cmd+←/→, Opt+←/→ 등 macOS 관용 단축키 미지원. 맥 사용자 UX 개선.

- **우선순위**: [ ]
- **배치**: [ ]

## [#226](https://github.com/edwardkim/rhwp/issues/226) · 2026-04-21 · @haseo-ai

**[Proposal] @rhwp/editor read-only viewer mode (readOnly option)**

`@rhwp/editor` 임베드 시 뷰어 전용 모드 요청 (툴바·메뉴 숨김). 파일 탐색기 통합 사용 사례.

- **우선순위**: [ ]
- **배치**: [ ]

## [#227](https://github.com/edwardkim/rhwp/issues/227) · 2026-04-21 · @DanMeon · discussions

**PyO3 Python 바인딩 + PyPI 배포 제안**

`dev_roadmap_v1_backup.md` Phase 2 의 PyO3 항목 제안. Python 생태계 HWP 도구 공백을 메우는 의의.

- **우선순위**: [ ]
- **배치**: [ ]

---

# D. HWPX Serializer 계보 (seunghan91)

## [#164](https://github.com/edwardkim/rhwp/issues/164) · 2026-04-16 · @seunghan91

**HWPX Serializer 구현 — Document IR → HWPX(ZIP+XML) 저장**

PR #170 머지로 텍스트/문단 직렬화 완료. 표/이미지/그림은 후속 (#172), header.xml 은 후속 (#171). **사실상 상위 이슈** — #170 후속이 분리되어 있어 완전 close 인지 검토 필요.

- **우선순위**: [ ]
- **배치**: [ ]

## [#171](https://github.com/edwardkim/rhwp/issues/171) · 2026-04-16 · @seunghan91

**HWPX Serializer: header.xml IR 기반 직렬화 (글꼴/스타일/문단 모양)**

현재 레퍼런스 템플릿 임베딩 상태. IR 기반 동적 생성 필요. #186 과 연관.

- **우선순위**: [ ]
- **배치**: [ ]

## [#172](https://github.com/edwardkim/rhwp/issues/172) · 2026-04-16 · @seunghan91

**HWPX Serializer: Section 컨트롤(표/이미지/그림) 직렬화**

Paragraph.controls IR 미직렬화 상태. 표/이미지/도형 HWPX 저장 시 손실됨.

- **우선순위**: [ ]
- **배치**: [ ]

## [#186](https://github.com/edwardkim/rhwp/issues/186) · 2026-04-17 → 2026-04-19 · @edwardkim · v1.0.0 · comments=1

**HWPX section.xml 완전 동적화 — secPr + 다중 run + Control dispatcher**

#182 Stage 2/3 이월분. #171 / #172 와 영역 겹침, 함께 처리 가능성.

- **우선순위**: [ ]
- **배치**: [ ]

## [#197](https://github.com/edwardkim/rhwp/issues/197) · 2026-04-19 · @edwardkim · 마일스톤 미지정

**HWPX→HWP 완전 변환기 (한컴 호환)**

#178 결론: 단순 어댑터로 한컴 호환 불가. 완전 변환기가 필요. 대형 작업.

- **우선순위**: [ ]
- **배치**: [ ]

---

# E. 인프라 · 문서 · 기타

## [#84](https://github.com/edwardkim/rhwp/issues/84) · 2026-04-09 → 2026-04-10 · @edwardkim · v1.0.0 · comments=1

**브라우저 확장 보안 취약점 수정 (Chrome/Edge/Safari)**

보안 감사 + 컨설턴트 검토 결과: Critical 2 / High 5 / Medium 5 / Low 1. Chrome/Edge 는 이미 v0.2.1 심사 통과, 취약점 별도 검증 필요.

- **우선순위**: [ ]
- **배치**: [ ]

## [#143](https://github.com/edwardkim/rhwp/issues/143) · 2026-04-14 · @edwardkim · v1.0.0

**LaTeX 파서 구현: 듀얼 토크나이저 방식 LaTeX 입력 지원**

한컴 파서 무수정 + LaTeX 전용 토크나이저/파서 추가. 수식 편집 UX 대폭 개선.

- **우선순위**: [ ]
- **배치**: [ ]

## [#144](https://github.com/edwardkim/rhwp/issues/144) · 2026-04-14 · @edwardkim · v1.0.0

**수식 편집 UI 개선: 듀얼 모드, 자동완성, 템플릿 확장**

#143 의 UI 짝. 명령어 자동완성/템플릿 확장/기호 검색. LaTeX 파서 완성 후 진행 자연스러움.

- **우선순위**: [ ]
- **배치**: [ ]

## [#185](https://github.com/edwardkim/rhwp/issues/185) · 2026-04-17 · @edwardkim · v1.0.0

**rhwp validate CLI — 한컴 DVC Rust 포팅 + HWP/HWPX 통합 서식 검증**

한컴 DVC (Apache 2.0) Rust 포팅. rhwp IR 활용으로 OWPML 중간층 제거. 한컴 호환성 검증 인프라.

- **우선순위**: [ ]
- **배치**: [ ]

## [#199](https://github.com/edwardkim/rhwp/issues/199) · 2026-04-19 · @edwardkim · 마일스톤 미지정

**rhwp-studio 인쇄 미리보기 창 스타일 깨짐 (about:blank 줌 메모리)**

Chrome about:blank 줌 메모리 문제로 인쇄 미리보기 창 거대 표시. 사용자는 Ctrl+0 으로 수동 해결 가능하나 일반 사용자 인지 어려움.

- **우선순위**: [ ]
- **배치**: [ ]

## [#225](https://github.com/edwardkim/rhwp/issues/225) · 2026-04-20 · @InsuJeong496

**mydocs/manual/MEMORY.md 링크 누락 및 중복 파일**

문서 정리 이슈. MEMORY.md 와 memory/MEMORY.md 중복, 링크 누락. 간단 수정.

- **우선순위**: [ ]
- **배치**: [ ]

---

# F. 알한글 iOS — M2~M5 (맥 환경 전용)

현재 Linux/WSL 환경 작업 불가. 작업지시자 본인 맥북에서만 진행 가능. **기본값 [iOS]** 권장.

## [#94](https://github.com/edwardkim/rhwp/issues/94) · M2: Core Graphics 렌더러 + Apple Pencil

**Apple Pencil 어노테이션 레이어**

PencilKit 연동. 투명 어노테이션 레이어 (필기/형광펜/메모).

- **우선순위**: [ iOS ]
- **배치**: [ ]

## [#100](https://github.com/edwardkim/rhwp/issues/100) · M2

**iPad 뷰어 UX 개선 (네비게이션, 툴바, 줌, 다크 모드)**

#93 완료 후 UX. 페이지 네비게이션 · 썸네일 사이드바 · 줌 · 다크모드.

- **우선순위**: [ iOS ]
- **배치**: [ ]

## [#95](https://github.com/edwardkim/rhwp/issues/95) · M3: .rhwp 포맷

**.rhwp 파일 포맷 설계 + 구현**

HWP 원본 + 어노테이션 레이어 → 단일 파일. 내보내기 .hwp/.pdf/.rhwp.

- **우선순위**: [ iOS ]
- **배치**: [ ]

## [#96](https://github.com/edwardkim/rhwp/issues/96) · M3

**레이어 관리 UI + 내보내기**

레이어 목록 · 가시성 · 잠금. PDF 내보내기 시 레이어 합성. 학생/교사 역할 구분.

- **우선순위**: [ iOS ]
- **배치**: [ ]

## [#97](https://github.com/edwardkim/rhwp/issues/97) · M4: 학습 워크플로우

**학습 워크플로우 UI (시험문제 풀기 + 채점)**

문제별 탭 이동 · 답안 · 풀이시간. 교사: 자동/수동 채점 · 코멘트.

- **우선순위**: [ iOS ]
- **배치**: [ ]

## [#98](https://github.com/edwardkim/rhwp/issues/98) · M4

**AI 자동 채점 + 학습 분석**

.rhwp 구조화 데이터 → AI. 객관식 자동 · 서술형 보조 · 오답 패턴 → 유사 문제 추천.

- **우선순위**: [ iOS ]
- **배치**: [ ]

## [#99](https://github.com/edwardkim/rhwp/issues/99) · M5: App Store 출시

**App Store 심사 + 베타 테스트 + 출시**

앱 아이콘 · 스크린샷 · 메타 · TestFlight · 심사.

- **우선순위**: [ iOS ]
- **배치**: [ ]

## [#87](https://github.com/edwardkim/rhwp/issues/87) · v1.0.0

**iOS 네이티브 HWP 뷰어 앱 개발 (rhwp-ios)**

iOS Safari Web Extension 구조적 제약 → 네이티브 앱 필요. M2~M5 의 상위 이슈.

- **우선순위**: [ iOS ]
- **배치**: [ ]

---

# 결정 후 절차

작업지시자 우선순위 기록 완료 후:

1. 본 문서의 P0/P1 목록으로 다음 사이클 작업 큐 구성
2. 각 이슈의 마일스톤·라벨 재설정 (GitHub)
3. **contrib-call** 표시 이슈 → `good first issue` / `help wanted` 라벨 부여
4. **close 권장** 표시 이슈 → 사유 코멘트 + close
5. iOS 이슈는 맥 환경 전용 별도 트랙 관리

작성 완료 후 문서 전체 한번에 공유해 주시면 됩니다.
