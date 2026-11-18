# 화학 문서 23쪽의 원본 휴먼모음T 공급

기존 원본·PDF·코드를 그대로 둔 fresh WASM 전 252쪽에서는 화학 23쪽 한 장만
97.57430%로 98%에 미달했다. 같은 쪽 Native는 98.53555%였다.
독립 PDF의 회색 장 제목은 `MoeumTR-HM`(휴먼모음T)이지만 Mac에 원본이 없어
일반 공개 fallback `NotoSansKR-Regular`가 선택됐다. 이 차이를 그림 차이로 분류하지 않는다.

검증용 Windows의 `HMFMMUEX.TTC` face 0과 독립 PDF xref 244의 실제 사용 글리프
46개는 윤곽·hmtx가 모두 일치했다. 이 확인은 해당 글리프 프로그램의 일치이며 서브셋 PDF
글꼴과 전체 TTC 파일의 해시 동일성을 주장하지 않는다. TTC에는 휴먼엑스포 face도 있다.

Mac 사용자 글꼴로 원본을 등록했다. CoreText의 두 PostScript 이름·실제 사용자 경로와
SHA-256을 확인했고, 새 WASM font catalog도 동일 TTC의 face 0을 선택했다. 시스템 글꼴을
바꾸거나 private TTC를 Git에 넣지 않았다. 제품 코드·WASM package·문서·기준 PDF도 그대로다.

같은 23쪽을 다시 측정해 Native/fresh WASM 각각 정밀 raw 100.00000%를 확인했다.
review·overlay는 같은 페이지의 실제 raw/PDF에서 만들었다. 폰트 추가 전 전쪽 결과와
폰트 추가 후 한 쪽 집중 결과를 구분하며, 새로운 글꼴 조건의 전 252쪽 완료로 세지 않는다.

리베이스는 충돌 없이 끝났고 원래 21개 패치와 제품·검사·글꼴 자산 tree가 모두 동일했다.
새로 반영된 CI 정책의 Python 검사 35건도 통과했다. 해당 자료 재사용 판단과 새 글꼴 조건의
재검증은 별개다. 최신 전체 회귀·보호 문서·실제 소비 경로는 아직 완료하지 않았다.
