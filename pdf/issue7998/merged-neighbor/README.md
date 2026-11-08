# #7998 — 병합된 셀 이웃의 연속 실선 경계

가로 병합 3셀(`colspan-retained.pdf`), 세로 병합 3셀(`rowspan-retained.pdf`), 정상 위쪽
정렬의 세로 병합(`rowspan-top-retained.pdf`)을 대조한다. `colspan-varying-side.pdf`는
가로 병합 셀의 긴 변 중간에 다른 굵기를 추가한 거절 반례다.

이들은 원래 한컴 Print 자체가 아니다. 공개 `../cell-regions/`의 한컴 PDF에서 텍스트·
text-show clip·셀 크기를 사용하고, 선 폭 0.359→0.5pt·독립 subpath의 compound 묶음·
join용 끝점의 명시적 셀 축 종료를 적용한 별도 계약이다. 원래 끝점 확장까지 인증했다고
주장하지 않는다. 용지만 399→600pt로 넓혀 열 삽입 후에도 문구가 종이 안에 남는다.
좁은 용지에서 한컴/rhwp 모두 오른쪽 문구가 잘리던 진단은 로컬에 따로 보존한다.

`print-source/rowspan-top.hwpx`는 원래 정상 `../cell-regions/print-source-inputs/rowspan.hwpx`의
실제 첫 셀 `verticalAlign`만 Top으로 바꾼 문서다. 같은 폴더의 `rowspan-top.pdf`는 해당
정확한 HWPX를 한컴 2020 11.0.0.9136·원본 용지·PrintMethod 0·one-up으로 인쇄한 독립
출력이다. 이를 사용한 세로 병합 대조군에서 새 행이 실제 선언 격자를 유지하는지 확인한다.
원래 문단 간격이 큰 입력의 행 편집을 강제로 허용하려고 LineSeg를 수정하지 않는다.

저장소 루트에서 다음으로 네 PDF를 바이트까지 재현한다(pikepdf 버전은 provenance 참조).

```bash
python3 pdf/issue7998/merged-neighbor/generate.py \
  --source-dir pdf/issue7998/cell-regions \
  --top-print pdf/issue7998/merged-neighbor/print-source/rowspan-top.pdf \
  --out /tmp/rhwp-7998-merged-neighbor-reproduced
```

`print/`의 PDF 30개는 실제 성공한 15종 행·열 편집의 HWP/HWPX를 각각 인쇄한 독립
기준이다. 이름 끝의 숫자와 `provenance.json`의 operation 맵은 기존 DocumentCore 행·열
명령을 뜻한다. 입력/저장본/기준 PDF SHA·600×360pt 원본 용지·engine/인쇄 방식·Native와
fresh WASM 점수를 같은 레코드로 연결한다. private 실물 PDF·폰트·서비스 자격 증명·로그는
포함하지 않는다.

가로 병합은 9종 편집, 원래 문단 간격의 세로 병합은 4종 열 편집, Top 세로 병합은 위/끝
행 추가 2종을 검증한다. 지원되지 않는 7종 격자와 변 중간 다른 pen의 6종 편집은 원문
모델·이벤트·실제 SVG·양 저장을 보존하며 거절한다. 세로 병합에 대한 일괄 금지가 아니라
원본 잉크와 실제 측정/배치가 같은 격자를 유지하는지 확인한다.

기존 자동 수집 `tests/cases/issue_7998_pdf_retained_border_structure.rs`에서 source의
3셀/span·실제 네 변 소유, 새 셀과 생존 문구, 셀/종이 내부의 최종 글자, 양 저장 재열기의
topology와 편집 거절의 원자성을 검사한다. 실제 경계 모습과 농도는 독립 Print와 전쪽
Visual Sweep으로 확인하며 임의 px·전체 SVG 해시를 기대값으로 고정하지 않는다.

검증 범위는 Native/fresh WASM 각 66입력·77쪽이며 전체 최저 97.57194%/97.13390%,
새 편집 저장본 30쪽 최저 99.90587%/98.85019%다. 기존 대조군/실물 37쪽 PNG는 각 backend에서
직전 수용본과 같다. 대표 compare/standalone overlay/review·전쪽 TSV·실제 Studio의
SnapshotCommand/undo/redo·현재 시각 판정 상태는
[stage28](../../../mydocs/working/task_m100_7998_stage28.md)에 연결한다. 2026-10-20 메인테이너가
**시각·편집 구조 통과**로 수용했다. 이를 R1/P4 또는 최종 PR 준비 완료로 확대하지 않는다.
