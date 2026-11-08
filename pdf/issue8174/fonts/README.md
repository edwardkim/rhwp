# PDF 폰트 예산 검증의 정상 CFF 대조군

`cff-cid.pdf`는 PyMuPDF로 직접 생성한 1쪽 PDF다. Nimbus Roman Regular의 CFF/OTF를
CID Type 0C (OT), Identity-H로 넣고 Latin·숫자·Greek·악센트·문장부호를 배치했다.
RHWP 출력에서 만든 기준 PDF가 아니며, PDF 가져오기 검증에서는 이 입력 PDF 자체가
독립 기준이다. 글꼴은 검증 호스트의 URW Base35 `NimbusRoman-Regular.otf`를 사용했다.

생성 코드·원본과 글꼴 해시·Native/fresh WASM 실행 증거는
[S07 기록](../../../mydocs/working/task_m100_8174_v2_stage7.md)을 따른다. 새 조판 golden이나
절대 px 회귀를 등록한 자료는 아니다. 예산 거절용 합성 CFF와 구분한다.

폭·간격의 현재 결함도 이 원본을 그대로 사용해 기록했다. 정답을 맞추려고 악센트·Greek·dash를
제거하거나 입력을 다시 생성하지 않는다. 내부 CFF는 ROS 없는 GID 대응이며 실제 CID-keyed
확장의 작은 한도 계약은 별도 합성 검사로 확인한다.

글꼴 저작권은 URW Software/URW++에 있다. 배포본의 AGPL-3와 PDF·PostScript 문서 포함
예외 고지를 [원문](URW-FONT-NOTICE.txt)에 보존했다. 원본 font 프로그램 자체를 별도 파일로
배포하는 자료는 아니다.
