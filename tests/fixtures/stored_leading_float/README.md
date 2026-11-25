# 음수 앵커를 공유하는 자리차지 표

두 파일은 공개 [float-pagination.hwp](https://github.com/edwardkim/rhwp/blob/2c0c1d205a54b8d5bd262241bac1391cffe0d801/mydocs/pr/assets/coanchored-table-pagination/float-pagination.hwp)의 파일 헤더와 글자·문단 모양을 바탕으로 새로 작성한 합성 입력이다. 본문은 제목과 새 영문 레이블을 담은 1×1 표 두 개로 구성했다. 실제 재현 사례의 쪽 크기와 여백, 글줄 메트릭, 앵커의 상대 기준과 두 표의 크기를 보존했다.

두 자리차지 표는 같은 공백 문단 안에 있고 세로 오프셋은 각각 −67,298 HWPUNIT과 +2,974 HWPUNIT이다. 첫 표의 외곽이 저장 호스트 줄의 시작에 닿는다. 두 파일의 표와 앵커는 동일하다. 제목 크기와 그에 따른 저장 높이만 13pt와 10pt로 다르며, 두 제목의 줄 간격은 120%다.

Windows 한컴 2020 Print는 13pt 제목, 첫 표, 둘째 표를 각각 별도 쪽에 둔다. 10pt 제목과 첫 표는 1쪽을 공유한다. [3쪽 한컴 인쇄본](../../../pdf/stored-leading-float-page-break/stored-negative-title-anchor-2020.pdf) · [2쪽 한컴 인쇄본](../../../pdf/stored-leading-float-page-break/stored-negative-title-fits-2020.pdf) · [출력 영수증과 시각 증적](../../../mydocs/pr/assets/stored-leading-float-page-break/README.md)
