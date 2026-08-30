# samples/issue5652 — #5652 B2 엔진 구조 편집 한컴 판정 입력

**이 폴더는 일반 fixture 가 아니다.** [#5652](https://github.com/edwardkim/rhwp/issues/5652)
B2-엔진 본구현에서 **엔진(`set_chart_data_by_index_native`, `structure:true`)이 만들어 한컴에서
판정한 32개 산출**이다. 현행 기준 PDF 는 [`pdf/issue5652/`](../../pdf/issue5652/)의 `-2020.pdf`이며,
판정 원장은 이 폴더의 [`MANIFEST.json`](MANIFEST.json), 결론은
[`mydocs/report/task_m100_5652_report.md`](../../mydocs/report/task_m100_5652_report.md) 다.
[#5447](https://github.com/edwardkim/rhwp/issues/5447) 스파이크 자산([`samples/issue5447/`](../issue5447/))의
후속이다 — 그쪽은 손으로 만든 변종, 이쪽은 **엔진이 만든 바이트**다.

## 회귀 코퍼스로 승격하지 말 것

- **`c:f` 참조 범위·③ 레거시 `Contents`·④ EMF 프리뷰는 설계대로 갱신하지 않았다**(#5447 확정 전제).
  행을 늘려도 `c:f` 는 옛 범위 그대로다.
- 계열삭제 2종은 **위치 기반**이다 — 뒤 계열의 이름·값이 앞으로 당겨지고 마지막 `c:ser` 가 지워진다.
  한컴 렌더는 #5447 의 "요소 제거 + 재번호" 산출과 픽셀 동일이다(원장 `cross_reference_issue5447`).
- `samples/` 를 훑는 기존 게이트는 전부 비재귀이고 `samples/chart` 만 명시적으로 합류하므로 이 폴더는
  어디에도 잡히지 않는다. 그 상태를 유지한다.

## 구성 — 32 파일

| 역할 | 수 | 무엇 |
|---|---|---|
| 대조군 | 7 `.hwpx` | `samples/chart/**` 원본 무편집 사본. 변종이 쓰는 기준 문서마다 하나 |
| 변종 | 12 × 2포맷 = 24 | 행추가 ×4(묶은세로·묶은가로·표식꺽은선·3D묶은세로)·행삭제·계열추가·계열삭제 ×2(묶은세로·누적세로)·계열명변경·라벨변경·분산형 점추가·특이케이스 점추가 — 전부 `csv-to-chart --structure` 와 같은 코어 경로 |
| 변환본 | 1 `.hwp` | `묶은세로막대형-행추가` 를 HWPX 에서 엔진으로 편집한 뒤 HWP5 로 변환 (①이 ②로 접힌다) |

경계 2종(원형 계열추가·주식형 계열삭제)은 엔진 가드(`pieSeriesCountFixed`·`stockSeriesCountFixed`)가
거부하므로 여기 없다 — 거부는 `tests/issue_4100_chart_data_edit.rs` 가 상시로 고정한다.

`PANJEONG.md` 는 이 꾸러미와 함께 작업지시자에게 보낸 **판정 지시표**다.

## 원장과 재계산

`MANIFEST.json` 이 32행 각각에 대해 원본 경로·SHA-256, 한컴 PDF 경로·SHA-256, 144dpi 래스터 해시
2축(PyMuPDF·poppler), 판정을 적어 둔다. **파일별 전체 SHA-256 은 이 원장이 유일한 정본**이다.

```bash
python tools/hancom_chart_judgment_verify.py --manifest samples/issue5652/MANIFEST.json                      # PyMuPDF
python tools/hancom_chart_judgment_verify.py --manifest samples/issue5652/MANIFEST.json --rasterizer pdftoppm
python tools/hancom_chart_judgment_verify.py --manifest samples/issue5652/MANIFEST.json --rasterizer none    # 해시만
```

CI 트립와이어 `b2_engine_judgment_assets_match_the_manifest`(`tests/issue_4100_chart_data_edit.rs`)가
원장 ↔ B2 원본·현행 `-2020.pdf` 32건의 SHA-256 과 등재 누락을 상시로 본다. 이 B2 판정 자산을
더하거나 빼면 그 테스트와 원장을 한 커밋에서 같이 옮긴다. 같은 `pdf/issue5652/`의 `-2022.pdf`는
이전 판정 PDF로 보관하되 현행 B2 원장 기준에는 속하지 않는다.
