# Stage 1 보고서 — Task #1082: 조사 + go/no-go (C군 누적 드리프트)

- 브랜치: `local/task1082` (소스 무변경)

## 확인 사실
- 단일 컬럼(단 0), 섹션 1개, 문단 0~487(typeset_paragraph DRIFT 최대 pi=487).
  dump-pages 의 `pi=607~666` 은 **다른(누적/표시용) 인덱스** — 실제 문단 0-487.
- overflow 페이지: 빈 FullParagraph 다수, typeset `used` 921~989px (body 1001.6px 내, **회계상
  fit**)인데 렌더가 누적 전진 드리프트로 하단 초과 — 최악 `pi=666 y=1989 col_bottom=1092
  overflow=897px`(+~1000px, 단순 +8/문단(240px)으로 설명 불가).
- 일부 페이지 dump-pages 가 이미 **`hwp_used≈ / diff=`** 계기 출력
  (`compute_hwp_used_height`, rendering.rs:2953 — 파일 vpos 기반 바닥). overflow 인근 페이지
  `diff=-90~-187px`(rhwp typeset used < 파일 vpos hwp_used) → **typeset 이 content 높이 과소
  계상**.

## 드리프트 구조 (불완전 — 추가 조사 필요)
- DRIFT: 문단당 fmt_total=21.3(lh13.3+ls8) vs 파일 vpos_h=13.3 → +8/문단. 그러나 used<hwp_used
  (과소)와 부호가 상충 → VPOS_CORR(vpos_snap)·HeightCursor 의 보정과 빈 문단 vpos 증분의
  상호작용이 복합. 단일 산식점이 아직 미특정.
- 빈 문단의 정체(시험지 답안 공간 등)·파일 vpos 증분 패턴이 페이지마다 달라 드리프트 출처가
  다중일 가능성.

## go/no-go 평가
- **범위/위험 최상**: 누적 vpos/line-spacing·VPOS_CORR·HeightCursor 는 **전 문서 공유 core**.
  기존 #1046/#1049/#1054 가 이미 손댄 영역(hwp_used 계기 존재 = 장기 추적 중). 변경 시 광범위
  회귀 위험.
- A/B/D군과 달리 **단일 localized 신호 부재** — 드리프트가 systemic·다중 출처 의심.
- 정확한 단일 root 특정에 Stage 1 추가 심화가 필요(현재 미특정).

## 권고 (작업지시자 결정 필요)
- **(가) 심화 Stage 1 계속**: 빈 문단 vpos 증분·VPOS_CORR·렌더 HeightCursor 정밀 분해로 단일
  root 특정 후 설계. 시간 투입 큼, 성공 시 4파일 해소.
- **(나) C군 보류 + 완료 4건 우선 통합**: #1068/#1070/#1073/#1079 PR 처리(미머지)를 먼저
  마무리하고, C군은 별도 심화 과제로 분리(가장 깊은 core 드리프트, known limitation 기록).

→ 진행 방향 승인 요청. (현 소스 clean.)

(작업지시자 1차 결정: 나 — 완료 4건 PR 우선. 이후 C군 재개 지시.)

---

## Stage 1 재개 — 돌파구: 다단(multi-column) 결함으로 특정 (stream/devel fbfcf682 rebase)

재개 후 새 devel base 에서 최악 overflow 가 **page 16/17, col=1**(2단 페이지 2번째 단)로 이동.
SVG 좌표 분석: col-1 content 가 **x=670~738(우측 단, 정상)인데 y=1711(본문 하단 1092 한참 아래)**
— x 는 단 1 로 리셋되나 **y 가 단 상단으로 리셋 안 됨** → 단 1 내용 수직 드리프트.

### 두 엔진 비교 (결정적)
| 파일 | typeset.rs(기본) max | engine.rs(RHWP_USE_PAGINATOR=1) max |
|------|------|------|
| 3-09'23 hwp | 626.9 | **0** |
| 3-09'22 hwp | 277.0 | **0** |
| 3-10'22 hwp | 158.5 | **0** |
| 3-11'22 hwp | 15.5 | **0** |
| 3-09'23 hwpx | 626.9 | **0** |

→ **레거시 Paginator(engine.rs)는 전 파일 0 overflow(정상)**, 기본 TypesetEngine(typeset.rs)만
드리프트. C군 = **typeset.rs 다단 처리 결함**, engine.rs 가 정상 reference.

### 재평가 (go 권고)
초기 "systemic·단일 root 미특정·보류" 평가를 **갱신**: 결함이 typeset.rs 다단 컬럼 advance/
vpos 리셋에 localize, **작동하는 참조 구현(engine.rs)** 존재 → 훨씬 tractable. 위험(타 다단
문서 영향)은 engine.rs oracle + 전수 sweep 으로 de-risk.

→ **Stage 2 진행 권고**: engine.rs vs typeset.rs 다단 컬럼 처리 비교 → 차이(컬럼 y 리셋/advance)
특정 → typeset.rs 정합. 승인 요청.
