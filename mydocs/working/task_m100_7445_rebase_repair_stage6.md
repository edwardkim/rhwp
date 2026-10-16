---
kind: working
status: active
canonical: mydocs/working/task_m100_7445_rebase_repair_stage6.md
last_verified: 2026-10-14
---

# #7445 최신 devel 리베이스와 완료 페이지 픽셀 회귀

Issue: #7445

## 최신 지시와 리베이스

- 사용자 지시대로 upstream/devel `c0d330737fabae77ca2bf1706cba8514d0e8f1df`를 fetch하고
  `fix/7445-completed-20261009`의 175개 커밋을 리베이스했다. 리베이스 직후 HEAD는
  `9ed3ac16fa32dae4b15137f99482d6a506b04af5`다.
- 표 조판 2곳, 후속 표 원점, 표 원점 종료 계약, 기술 문서 충돌은 기존 조판 의도와 새 행 조회
  인덱스를 함께 보존했다. 임시 stash `692a2990e401433ed5aa4ff485355aecbe814c24`를 apply한 뒤
  사용자 오늘할일·PR 초안, 독립 PDF 5개의 SHA가 같음을 확인했다. stash와 이전 backup ref는 보존한다.
- 새 canonical 지침은 **문서 전체가 아닌 완료 페이지별** Native/fresh WASM 각각98% 이상·
  직접 판독 후 독립 PDF의 좌표를 ±2 CSS px로 검사하도록 허용한다. 관계 검사는 함께 유지한다.
  최신 devel 대비0.01%p 하락 재검토 규칙도 확인했으며 전체 base/head 비교는 아직 완료되지 않았다.

## 완료 페이지의 중요한 좌표 검사

전체 1쪽씩 같은 원본·독립 한컴 Print PDF·compat2022·192dpi에서 직접 PNG를 비교했다.
Native는 native-skia, fresh WASM은 Chrome Canvas2D다. 기존 SVG raster 점수는 재사용하지 않았다.

| 원본 | 페이지 | Native | fresh WASM | 보호한 항목 |
| --- | --- | --- | --- | --- |
| `36494836_field_marker_tac_lines.hwpx` | 1 | 100.00000% | 99.99953% | 결재란 왼쪽·상단, 보고 내용 표 상단 |
| `36312980_broken_cell_ladder_center.hwpx` | 1 | 99.60028% | 99.62015% | 내용 칸 상단·하단 및 기존 가운데 정렬 관계 |

두 경로의 review/overlay PNG를 직접 열어 표 괘선, 탭 뒤 결재란, 셀 안 글줄의 정렬과 내용 보존을
확인했다. 기대 좌표는 PDF 괘선의 pt를96/72로 환산했으며 구현 bbox를 복사하지 않았다.
대상은 문단·컨트롤·셀 소유와 결재 내용으로 찾는다. px 영역으로 대상을 선택하지 않는다.
기존 탭 폭·순서·셀 소유·여백 관계 검사는 유지했고 절대 좌표 assertion5개를 더했다.

- `issue_7344_field_marker_stored_tac_lines`: **4 PASS / 0 FAIL**,0.029초. 새 검사5개 포함.
- Native/fresh 증거 및 독립 좌표（로컬 중간 자료 `evidence.json`）
- [누름틀 Native review](../pr/assets/issue7445/completed-20261009/pixel-regression-20261014/field-marker1-native-review.png)
- [누름틀 fresh WASM review](../pr/assets/issue7445/completed-20261009/pixel-regression-20261014/field-marker1-wasm-review.png)
- [가운데 정렬 Native review](../pr/assets/issue7445/completed-20261009/pixel-regression-20261014/broken-ladder1-native-review.png)
- [가운데 정렬 fresh WASM review](../pr/assets/issue7445/completed-20261009/pixel-regression-20261014/broken-ladder1-wasm-review.png)

## 실행 신원과 도구 보정

- Native CLI 빌드와 Mac locked wrapper `--target web --out-dir pkg --no-opt`가 성공했다.
  Docker 최적화 빌드의 증거는 아니다. root pkg와 Studio/public의 JS/WASM hash는 일치한다.
- 실제 새 Chrome 탭의 초기화 응답 JS/WASM hash를 관측했다. 두 응답200이며 frozen package와
  일치했고 observer 캡처의 PNG도 정식 fresh WASM PNG와 같다. Studio 실제 소비36건을 완료한
  결과는 아니며 export의 실제 초기화 확인이다.
- 새 PNG 공급의 font catalogue가 오래된 euc_kr name record를 해독하다 실패했다.
  유효한 동일 face 이름을 보존하고 해독 실패 레코드를 provenance에 남겼다. 유효한 이름이
  전혀 없는 face는 실패로 처리한다. 원본 font bytes·glyph·폭·실루엣 계산·허용치를 변경하지 않았다.
- 실제 실패와 동일한 해독 오류를 포함한 검사: 수정 전1 ERROR → 수정 후 전체14 PASS.

## 계속 진행할 범위

이번 Native/fresh 생산자는 리베이스 HEAD 위 미커밋 조판 후보를 포함한다. source 파일별 SHA와
patch hash를 evidence에 기록했다. 이를 bare HEAD 전체 통과로 인용하지 않는다. 이 단계의 commit은
완료 페이지 좌표 검사·도구 보정·해당 증거만 포함한다. 사용자 초안과 미완료 #4138 후보는 섞지 않는다.

#4138 집중 검사4건 중3PASS,1FAIL: 원본115·분할 저장197 및 자동 내어쓰기 탭 검사는 통과했고,
독립 Print90쪽의 네 번째 줄이 다음 쪽으로 밀리는 진단은 실패한다. 기존 기대값은 유지했다.
같은 고정 Celsius 입력의 최신 직접 Native PNG7쪽에서는23/25/60/90/91이90% 미만이다.
이 7쪽은 진단 범위이고 전197쪽 완료가 아니다. #4138 및43조건 전쪽 검증, 최신 base/head0.01%p
비교, 전체 nextest·lint/Skia/기타 게이트와 실제 소비 경로 검증을 계속한다. **PR 준비 완료가 아니다.**

실행 로그·TSV·실패 이력은 ignored `output/pr-review/7445-completed-20261009/rebase-latest-20261014/`
아래에 보존한다. 지침 변경·remote push·PR·issue/comment·merge는 수행하지 않았다.
