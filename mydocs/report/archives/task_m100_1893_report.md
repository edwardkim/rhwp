# 최종 결과 보고서 — Task #1893

**이슈**: #1893 해양경찰청 별지 서식 hwpx — IR 보존인데 라운드트립 렌더 위치 452~752px 분기
**브랜치**: `local/task1893` (origin/devel 12f7c03a 기반)
**마일스톤**: M100
**작성일**: 2026-07-04

---

## 1. 요약

20k 라운드트립 위치 검사에서 표면화된 해양경찰청 범죄수사규칙 별지 서식 계열의 라운드트립
렌더 분기(최대 752px)를 **빈 누름틀(CLICK_HERE) 필드의 3중 결함 체인**으로 규명하고 교정했다.
대표 3066571: **752px → PASS 0.00px**. 계열 9/9 전부 PASS 전환 + 타 기관 서식·규제영향분석서
포함 **광역 16건 개선, 악화 0**.

## 2. 근본원인 — 3중 결함 체인 (초기 진단 2회 정정)

진단 여정: ①빈 `<t/>` 직렬화 의심(오귀속 — 무해한 plain 직렬화 산출물과 비교했었음) →
②ZIP 컨테이너 의심(오귀속 — 2-파일 모드와 `--via` 모드 혼동) → ③**진범 = DocumentCore
경유 직렬화**(`export_hwpx_native` = 실제 편집기 저장 경로)에서만 결정적(5/5) 재현.

1. **`clear_initial_field_texts` 수술 불완전** (document_core/commands/document.rs):
   초기상태 누름틀 안내문("소속관서" 등)을 `para.text`/`field_ranges` 에서만 삭제하고
   **char_offsets/char_count/char_shapes 를 stale 방치**. 이 불일치 IR 을 직렬화하면
   재파스 정준형과 조판이 갈라짐. (안내문 삭제 자체는 정합 — pyhwpx 한글 2022 PDF 로
   한컴도 안내문을 렌더하지 않음을 확인.)
2. **직렬화기: 문단 끝 0-length 필드의 fieldEnd 선방출** (serializer/hwpx/section.rs):
   후처리 순서가 [미방출 fieldEnd 전부]→[잔여 슬롯(fieldBegin)]이라 문단 끝 빈 필드는
   end 가 begin 앞에 놓임 → 재파스가 고아 end + 미닫힘 begin 으로 해석, field_range 소실
   → 빈 누름틀 placeholder 미렌더.
3. **직렬화기: 슬롯 루프의 갭 침범**: 같은 갭에서 fieldEnd 몫 8유닛을 다음 fieldBegin
   슬롯이 탐욕 소진 → begin 연속 배치 → 재파스 LIFO 페어링 교차
   (fr(0,0)+(50,50) → fr(0,50)+(0,0)) → placeholder 소실 + 줄바꿈 분기.

## 3. 수정

- **document.rs**: 삭제 수술 완성 — 원본 char_offsets 스냅샷으로 utf16 범위를 구해
  ① 삭제 구간 오프셋 엔트리 drain + 후속 엔트리 감산(**삭제 문자 폭만** — `orig_offsets[end]`
  사용 시 end 마커 8유닛 갭까지 폭에 포함되어 갭 소실, 실측 727px 역행으로 확인)
  ② char_count 감산 ③ char_shapes 경계 시프트(범위 내부 경계는 zero-width run 으로
  시작점 고정 — 한컴 자신의 필드값-삭제 표현과 동형).
- **serializer/hwpx/section.rs**: ① 문단 끝 0-length 필드 fieldEnd 를 자기 begin 슬롯
  직후로 지연 ② 문자 루프 내 슬롯 방출 직후 자기 0-length fieldEnd 인터리브(begin→end
  순서 보존) ③ 말미 방어 방출.
- 시도-폐기: `rebuild_char_offsets` 재사용 — 선행-컨트롤 휴리스틱(첫 gap/8)이 문단 서두
  0-length 필드의 end 마커를 컨트롤로 오산해 begin 갭 유실(126px 잔존) → 직접 수술로 교체.

## 4. 검증

| 게이트 | 결과 |
|---|---|
| 대표 3066571 `render-diff --via hwpx` | 752px STRUCT → **PASS 0.00px** |
| 해경 범죄수사규칙 별지 계열(20k 표본 내 비-PASS 9건) | **9/9 PASS 전환** (잔여 2건은 별표/도형류 별개 축) |
| 20k 표본 hwpx 비-PASS 127건 전수 재검 | **16건 →PASS, 악화 0** (관세청·금융위·행안부·국방부 서식, 규제영향분석서 #1891 계열 3건 포함) |
| big_hwpx 2,500 render-diff | PASS 2480→**2483** (STRUCT→PASS 2, OVER→PASS 1), **회귀 0** |
| admrul_0013 쪽수 4→2 변화 | **한컴 2쪽 정합 확인**(pyhwpx) — devel 4쪽이 오답, 부수 개선 |
| `cargo test --release` 전 스위트 | 통과 (doc-test 까지 완주, FAILED 0) |
| 신규 핀 `tests/issue_1893.rs` | 통과 (roundtrip_geom ≤1px + 구조 일치 + 1쪽) |
| 한컴 PDF 대조 | fixture 렌더 median +0.41pt (p1, compare_line_baselines) |

## 5. 동봉

- `samples/issue1893_clickhere_field_roundtrip.hwpx` — 재현 fixture (해경 별지 92, 16KB)
- `pdf/issue1893_clickhere_field_roundtrip-2022.pdf` — 한글 2022 권위 PDF (pyhwpx,
  리뷰 규칙: 검증 정답지는 pdf/ 에 커밋)
- `tests/issue_1893.rs` — 라운드트립 렌더 자기정합 핀

## 6. 잔여/후속

- 해경 잔여 2건(별표 순서도 452px/기록부 288px)은 CLICK_HERE 서식이 아닌 도형·표류 별개
  축 — #1892(대법원 서식)와 함께 별도 추적.
- `clear_initial_field_texts` 가 로드 시 영속 IR 을 변형하므로 **저장 시 안내문이 파일에서
  소실**되는 데이터-보존 축은 남음(열고 저장만 해도 안내문 삭제). 본 수정으로 렌더
  자기정합은 확보; 안내문 원문 보존(render-side clear 이관)은 별도 설계 사안.
- 진단 부산물: `hwpx-roundtrip` CLI 와 render-diff 의 ZIP 리더 관용도 불일치(#1891 단서)
  는 본 건과 무관함을 확인(플래그 그대로 유효).
