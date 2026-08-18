#!/usr/bin/env python3
"""M-fill fields / fill-fields / batch fill fixture generator.

Reads live contract functions in form_fill.py and the form inventories in
catalogs.py, then writes survey / targeting / dry-run / verify / batch /
#4781 홍길동 픽스처. DocumentCore is not called. gym is not touched.

    python tools/form_fill/fatten_form_fill.py
    python tools/form_fill/test_form_fill.py
    python tools/form_fill/test_fatten_form_fill.py
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import catalogs as catalog_mod
import form_fill as ff

CLAIM_ID = "M-fill"
GENERATOR = "tools/form_fill/fatten_form_fill.py"
SCHEMA_VERSION = "1.0"


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def write_json(path: Path, data: Any) -> None:
    write_text(path, json.dumps(data, ensure_ascii=False, indent=2))


def write_form_catalog(path: Path, data: dict[str, Any]) -> None:
    """Pretty metadata, one JSON object per field — reviewable, not padded."""
    fields = data.get("fields") or []
    head = {key: value for key, value in data.items() if key != "fields"}
    lines = ["{"]
    items = list(head.items())
    for key, value in items:
        lines.append(f"  {json.dumps(key, ensure_ascii=False)}: {json.dumps(value, ensure_ascii=False)},")
    lines.append('  "fields": [')
    for index, rec in enumerate(fields):
        comma = "," if index + 1 < len(fields) else ""
        lines.append(f"    {json.dumps(rec, ensure_ascii=False)}{comma}")
    lines.append("  ]")
    lines.append("}")
    write_text(path, "\n".join(lines))


CORE_FORM_IDS = {
    "field-01",
    "field-01-memo",
    "form-01",
    "gian-1",
    "gian-2",
    "reg-80168",
    "hongbo",
    "trip-apply",
    "leave-apply",
    "bokhak",
    "minwon",
    "minutes",
    "labor-contract",
    "hr-card",
    "eval-sheet",
    "bid",
    "spend",
    "foi",
    "overtime",
    "trip-settle",
    "admin-appeal",
    "contract",
    "parental-leave",
    "payroll-account",
    "family-cert",
    "empty-none",
}

BATCH_FORM_IDS = {
    "field-01",
    "gian-1",
    "reg-80168",
    "minwon",
    "eval-sheet",
    "spend",
}

HONG_CLONE_IDS = {
    "field-01",
    "gian-1",
    "hongbo",
    "minwon",
    "labor-contract",
    "bid",
    "contract",
    "family-cert",
    "build-permit",
    "foi",
    "admin-appeal",
    "pension",
    "hr-card",
    "trip-apply",
    "security-oath",
}


def md_cell(value: Any) -> str:
    return ff.nfc(str(value)).replace("|", "\\|").replace("\n", " ")


def short_hash(blob: str) -> str:
    return hashlib.sha256(blob.encode("utf-8")).hexdigest()[:12]


@dataclass
class Bundle:
    generated_at: str
    out_root: Path
    forms: list[dict[str, Any]] = field(default_factory=list)
    fills: list[dict[str, Any]] = field(default_factory=list)
    occurrences: list[dict[str, Any]] = field(default_factory=list)
    dry_runs: list[dict[str, Any]] = field(default_factory=list)
    verifies: list[dict[str, Any]] = field(default_factory=list)
    batches: list[dict[str, Any]] = field(default_factory=list)
    hongs: list[dict[str, Any]] = field(default_factory=list)
    paths: list[dict[str, Any]] = field(default_factory=list)
    written: list[str] = field(default_factory=list)


def record(bundle: Bundle, path: Path) -> Path:
    rel = path.resolve().relative_to(bundle.out_root.resolve()).as_posix()
    bundle.written.append(rel)
    return path


SAMPLE_VALUES = {
    "회사명": "주식회사 검증",
    "작성자": "김가온",
    "부서명": "기획조정실",
    "전화번호": "044-200-7000",
    "이메일": "planner@example.go.kr",
    "제목": "2026년 행정업무 자동화 시범사업 협조 요청",
    "목차1": "추진 경과",
    "myMsg01": "홍길동 귀하",
    "행정기관명": "행정안전부",
    "수신자": "각 중앙행정기관의 장",
    "경유": "",
    "본문": "1. 관련: 행정안전부 혁신정책담당관-1234(2026. 7. 20.)",
    "붙임": "시범사업 계획서 1부",
    "발신명의": "행정안전부장관",
    "수신자명단": "각 중앙행정기관의 장",
    "기안자": "행정사무관 이도윤",
    "검토자": "혁신정책담당관 박서준",
    "결재권자": "혁신정책관 최민서",
    "협조자": "디지털정부국",
    "시행번호": "혁신정책담당관-5678",
    "시행일": "2026. 7. 26.",
    "접수번호": "",
    "접수일": "",
    "우편번호": "30112",
    "주소": "세종특별자치시 도움6로 42",
    "홈페이지": "www.mois.go.kr",
    "팩스번호": "044-200-7001",
    "전자우편": "planner@mois.go.kr",
    "공개구분": "공개",
    "생산등록번호": "혁신정책담당관-9012",
    "등록일": "2026. 7. 26.",
    "결재일": "2026. 7. 26.",
    "직위1": "주무관",
    "직위2": "사무관",
    "직위3": "과장",
    "직위4": "국장",
    "요약설명": "시범사업 협조 내부 보고",
    "작성일": "2026. 7. 26.",
    "작성기관": "혁신정책담당관",
    "서식명": "규제영향분석서",
    "법령명": "행정 효율과 협업 촉진에 관한 규정",
    "소관부처": "행정안전부",
    "작성일자": "2026. 7. 1.",
    "연락처": "044-200-7000",
    "규제목적": "반복 서식 작성 부담 완화",
    "규제내용": "표준 누름틀 사용 권고",
    "피규제집단명": "가상협회 회원사",
    "대안비교": "현행 유지 대비 표준 서식",
    "비용편익": "수기 재작성 시간 절감",
    "의견수렴": "관계 부처 서면 의견",
    "일몰설정": "2028. 12. 31.",
    "기관명": "한국수자원공사",
    "배포일시": "2026. 1. 30. 10:00",
    "담당부서": "홍보부",
    "담당자": "정하은",
    "부제": "디지털 전환 성과",
    "문의처": "홍보부 044-000-0000",
    "첨부": "사진 3매",
    "사진설명": "현장 전경",
    "소속": "기획조정실",
    "직급": "행정사무관",
    "성명": "홍길동",
    "출장목적": "현장 점검",
    "출장지": "대전광역시",
    "출발일": "2026. 8. 3.",
    "귀임일": "2026. 8. 4.",
    "교통수단": "철도",
    "여비구분": "국내",
    "여비예산": "국내여비",
    "동행자": "김가온",
    "긴급연락처": "010-1234-5678",
    "신청일": "2026. 7. 28.",
    "신청자": "홍길동",
    "휴가종류": "연가",
    "시작일": "2026. 8. 10.",
    "종료일": "2026. 8. 12.",
    "일수": "3",
    "잔여연가": "11",
    "사유": "개인 용무",
    "업무대리자": "이도윤",
    "대학": "공과대학",
    "학과": "컴퓨터공학과",
    "학번": "20181234",
    "학년": "3",
    "휴학기간": "2025-2 ~ 2026-1",
    "복학예정학기": "2026-2",
    "휴학사유": "군복무",
    "보호자": "홍판서",
    "보호자연락처": "010-2222-3333",
    "수험번호": "26001234",
    "전형": "수시 학생부종합",
    "모집단위": "컴퓨터공학부",
    "주민등록번호": "000101-3******",
    "생년월일": "2000. 1. 1.",
    "출신학교": "세종고등학교",
    "졸업연월": "2018. 2.",
    "보호자성명": "홍판서",
    "보호자관계": "부",
    "지원동기": "공공 문서 자동화 연구",
    "단과대학": "사회과학대학",
    "평균평점": "3.85",
    "이수학점": "98",
    "장학과": "성적우수",
    "신청금액": "2,000,000",
    "계좌은행": "농협",
    "계좌번호": "123-456-789",
    "예금주": "홍길동",
    "가족성명": "홍판서",
    "가족관계": "부",
    "신청사유": "생활비 지원",
    "민원제목": "도로 파손 보수 요청",
    "민원내용": "세종대로 인도 침하",
    "관계": "본인",
    "처리기한": "2026. 8. 15.",
    "공개여부": "공개",
    "회의명": "디지털정부 실무협의회",
    "개최일시": "2026. 7. 30. 14:00",
    "장소": "본관 3층 중회의실",
    "주관부서": "디지털정부국",
    "참석자": "김가온",
    "안건1": "서식 표준화",
    "안건2": "누름틀 안내문",
    "안건3": "배치 채움",
    "결정사항": "표준 서식 우선 적용",
    "확인자": "과장 박서준",
    "공고기관": "행정안전부",
    "공고번호": "제2026-88호",
    "공고제목": "표준 서식 개정 고시",
    "공고내용": "별지 제1호서식 개정",
    "제출처": "혁신정책담당관",
    "게시일": "2026. 7. 20.",
    "사업장명": "한국문서공사",
    "계약시작일": "2026. 3. 1.",
    "계약종료일": "2027. 2. 28.",
    "근무장소": "세종청사",
    "업무내용": "서식 검토",
    "소정근로시간": "09:00-18:00",
    "임금": "월 3,200,000원",
    "임금지급일": "매월 25일",
    "사번": "A10234",
    "한자성명": "洪吉童",
    "직위": "주무관",
    "입사일": "2019. 3. 4.",
    "학력학교": "한국대학교",
    "학력전공": "행정학",
    "경력기관": "지방자치인재개발원",
    "신장": "172",
    "체중": "68",
    "혈압": "120/80",
    "흡연여부": "비흡연",
    "음주여부": "주 1회",
    "복용약": "해당 없음",
    "과거병력": "없음",
    "문진일": "2026. 6. 2.",
    "상호": "한빛소프트",
    "대표자": "한빛",
    "사업장소재지": "서울특별시 강남구",
    "업태": "정보통신",
    "종목": "소프트웨어 개발",
    "개업일": "2015. 4. 1.",
    "사업자단위과세": "해당 없음",
    "공동사업자": "김공동",
    "허가번호": "허가 제2026-17호",
    "대지위치": "세종특별자치시 나성동",
    "지번": "나성동 123-4",
    "대지면적": "450",
    "건축면적": "180",
    "연면적": "520",
    "용도": "업무시설",
    "자격번호": "건축사 제12345호",
    "등록번호": "토목공사업 제99호",
    "착공예정일": "2026. 9. 1.",
    "준공예정일": "2027. 8. 31.",
    "사업명": "스마트워크 센터 확대",
    "사업자": "행정안전부",
    "평가대행자": "한국환경연구원",
    "사업위치": "세종·대전",
    "사업기간": "2026-2028",
    "평가항목": "대기질",
    "협의기관": "환경부",
    "과제명": "공공 서식 자동 채움",
    "연구책임자": "이민준",
    "소속기관": "한국지능정보사회진흥원",
    "연구기간": "2026. 3. ~ 2026. 12.",
    "연구비": "180,000,000",
    "공동연구원": "박하린",
    "연구목표": "누름틀 반복 필드 지목 안정화",
    "연구내용": "이름[N] 계약 픽스처",
    "기대효과": "제출 전 빈칸 감소",
    "키워드": "누름틀, 메일머지, 서식",
    "평가회차": "1차",
    "평가대상": "시범 산출물",
    "평가자": "정평가",
    "배점": "10",
    "점수": "8",
    "총점": "64",
    "종합의견": "반복 필드 지목 양호",
    "입찰건명": "서식 자동화 용역",
    "사업자번호": "123-45-67890",
    "위임범위": "입찰·계약",
    "입찰보증금": "5,000,000",
    "결의번호": "지-2026-441",
    "결의일": "2026. 7. 15.",
    "지급처": "한국문서공사",
    "계정과목": "일반수용비",
    "적요": "서식 인쇄",
    "금액": "1,200,000",
    "합계": "4,800,000",
    "원인행위자": "주무관 이도윤",
    "검인자": "사무관 김가온",
    "지출관": "서기관 박서준",
    "품의번호": "품-2026-19",
    "기안부서": "혁신정책담당관",
    "품의내용": "표준 서식 배포",
    "소요예산": "12,000,000",
    "관련근거": "행정업무운영 편람",
    "기안일": "2026. 7. 21.",
    "발신부서": "혁신정책담당관",
    "수신부서": "디지털정부국",
    "협조내용": "필드 이름 통일",
    "회신기한": "2026. 8. 5.",
    "계약명": "서식 인쇄 구매",
    "계약번호": "계-2026-77",
    "계약상대자": "한빛인쇄",
    "품목": "복사용지",
    "규격": "A4 80g",
    "수량": "200",
    "검수일": "2026. 7. 18.",
    "검수자": "한검수",
    "입회자": "박입회",
    "판정": "합격",
    "신청구분": "신규",
    "출입구역": "본관 3층",
    "유효기간": "2026. 12. 31.",
    "동반자": "김동행",
    "청구기관": "행정안전부",
    "정보내용": "표준 서식 원문",
    "공개방법": "전자파일",
    "수령방법": "전자우편",
    "위임여부": "해당 없음",
    "청구일": "2026. 7. 22.",
    "처리기관": "개인정보보호위원회",
    "청구내용": "열람",
    "열람방법": "전자",
    "차량번호": "세종 12가 3456",
    "차종": "쏘나타",
    "운행월": "2026-07",
    "운전자": "최운전",
    "출발지": "세종청사",
    "도착지": "대전청사",
    "주행거리": "42",
    "연료량": "30",
    "근무일": "2026. 7. 23.",
    "시작시각": "18:30",
    "종료시각": "21:00",
    "업무내용": "공고 초안",
    "승인자": "과장 박서준",
    "과정명": "공공기록 관리",
    "주관기관": "국가기록원",
    "교육장소": "대전 본원",
    "교육기간": "2026. 9. 1. ~ 9. 3.",
    "교육비": "0",
    "교육생": "이도윤",
    "발주기관": "조달청",
    "제안사": "한빛소프트",
    "구성원": "문서팩토리",
    "제출일": "2026. 8. 1.",
    "유효기간": "90일",
    "서약일": "2026. 3. 2.",
    "서약내용": "비밀 준수",
    "신청부서": "운영지원과",
    "물품명": "노트북",
    "반출목적": "출장 업무",
    "반출일": "2026. 8. 3.",
    "반입예정일": "2026. 8. 6.",
    "반출지": "대전청사",
    "회의실": "중회의실 A",
    "사용목적": "실무협의",
    "참석인원": "12",
    "사용일": "2026. 8. 7.",
    "기자재": "프로젝터",
    "출장자": "홍길동",
    "일자": "2026. 8. 3.",
    "교통비": "38,400",
    "숙박비": "80,000",
    "식비": "25,000",
    "정산일": "2026. 8. 6.",
    "요구부서": "혁신정책담당관",
    "요구자": "이도윤",
    "사용목적": "교육",
    "품명": "토너",
    "추정단가": "85,000",
    "희망납기": "2026. 8. 20.",
    "예산과목": "일반수용비",
    "취득일": "2018. 4. 1.",
    "장부가액": "1",
    "폐기사유": "내용연수 만료",
    "처분방법": "폐기",
    "이관연도": "2026",
    "이관부서": "운영지원과",
    "인계자": "김인계",
    "인수자": "박인수",
    "철제목": "서식 관리",
    "생산연도": "2024",
    "보존기간": "10년",
    "이관일": "2026. 12. 15.",
    "평가위원회": "2026년 1차 평가위원회",
    "평가일": "2026. 11. 10.",
    "대상기록": "2014년 협조전",
    "평가의견": "한시 보존 종료",
    "처분": "폐기",
    "위원장": "최위원",
    "간사": "정간사",
    "피청구인": "세종특별자치시",
    "처분내용": "건축허가 거부",
    "처분일": "2026. 6. 1.",
    "청구취지": "거부처분 취소",
    "청구이유": "요건 충족",
    "원처분기관": "세종특별자치시",
    "원처분번호": "허가 제2026-3호",
    "원처분일": "2026. 6. 1.",
    "이의내용": "요건 재심사 요청",
    "청원제목": "마을버스 노선 신설",
    "청원요지": "나성동 노선 신설",
    "연명자": "이연명",
    "제출기관": "세종특별자치시의회",
    "민원내용": "과속 방지턱 설치",
    "처리기관": "세종특별자치시",
    "계약금액": "48,000,000",
    "납품기한": "2026. 9. 30.",
    "납품장소": "세종청사 자재창고",
    "특약": "검수 후 잔금",
    "계약일": "2026. 7. 25.",
    "공급자상호": "한빛인쇄",
    "공급자등록번호": "123-45-67890",
    "공급받는자상호": "행정안전부",
    "공급받는자등록번호": "111-82-00309",
    "작성일자": "2026. 7. 25.",
    "공급가액": "1,000,000",
    "세액": "100,000",
    "비고": "교육용",
    "명령번호": "출-2026-55",
    "기간": "2026. 8. 3. ~ 8. 4.",
    "여비": "국내여비",
    "발령권자": "혁신정책관",
    "발령일": "2026. 7. 29.",
    "발생연가": "16",
    "사용연가": "5",
    "신청일수": "3",
    "자녀성명": "홍아이",
    "자녀생년월일": "2022. 5. 5.",
    "휴직시작일": "2026. 9. 1.",
    "휴직종료일": "2027. 2. 28.",
    "급여계좌": "농협 123-456-789",
    "겸직기관": "사단법인 문서연구회",
    "겸직업무": "자문위원",
    "겸직기간": "2026. 8. ~ 2026. 12.",
    "겸직사유": "전문성 활용",
    "청구구분": "본인",
    "퇴직일": "2026. 6. 30.",
    "재직기간": "20년 3월",
    "기존은행": "국민",
    "기존계좌": "111-22-3333",
    "변경은행": "농협",
    "변경계좌": "123-456-789",
    "변경사유": "급여 이체 편의",
    "증명종류": "가족관계증명서",
    "부수": "2",
    "제출처": "국민연금공단",
    "정정항목": "주소",
    "정정전": "세종시 도움6로 1",
    "정정후": "세종시 도움6로 42",
    "첨부": "주민등록등본",
    "열람종류": "일반건축물대장",
    "소재지": "세종특별자치시 나성동 12-3",
    "수령방법": "전자발급",
}


OCCURRENCE_VALUES = {
    "피규제집단명": [
        "가상협회 회원사",
        "중소기업중앙회 소속 조합",
        "지방자치단체 산하 공단",
        "개인택시운송조합",
        "전국화물자동차운송사업연합회",
        "대한건설협회",
        "한국소프트웨어산업협회",
        "전국학원연합회",
        "대한의사협회",
        "대한약사회",
        "한국음식업중앙회",
        "전국버스운송사업조합연합회",
        "한국관광협회중앙회",
        "소상공인시장진흥공단 가맹점",
    ],
    "목차1": ["추진 배경", "현황 분석", "개선 과제", "추진 일정", "기대 효과"],
    "가족성명": ["홍판서", "춘섬", "홍길동", "홍아이"],
    "가족관계": ["부", "모", "본인", "자녀"],
    "참석자": ["김가온", "이도윤", "박서준", "최민서", "정하은", "한검수", "윤하린", "서준호"],
    "평가항목": ["창의성", "실현가능성", "비용효율", "파급효과", "지속가능성", "위험관리"],
    "공동연구원": ["박하린", "정하은", "윤서준", "한지민", "오세훈"],
    "계정과목": ["일반수용비", "국내여비", "공공요금", "자산취득비"],
    "적요": ["서식 인쇄", "출장 철도", "회선 사용료", "복합기"],
    "금액": ["1,200,000", "384,000", "220,000", "3,000,000"],
    "품목": ["복사용지", "토너", "파일박스", "스테이플러", "바인더"],
    "규격": ["A4 80g", "검정", "A4", "중형", "5cm"],
    "수량": ["200", "4", "30", "10", "20"],
    "운전자": ["최운전", "박운전", "이운전", "김운전", "한운전", "정운전"],
    "출발지": ["세종청사", "세종청사", "대전청사", "세종청사", "정부서울청사", "세종청사"],
    "도착지": ["대전청사", "오송역", "세종청사", "정부서울청사", "세종청사", "조치원"],
    "근무일": ["2026. 7. 20.", "2026. 7. 21.", "2026. 7. 22.", "2026. 7. 23.", "2026. 7. 24."],
    "철제목": [
        "서식 관리",
        "협조전",
        "지출결의",
        "출장명령",
        "보안서약",
        "회의록",
        "계약",
        "민원",
    ],
}


def sample_value(name: str, occurrence: int = 0) -> str:
    series = OCCURRENCE_VALUES.get(name)
    if series:
        return series[occurrence % len(series)]
    if name in SAMPLE_VALUES:
        base = SAMPLE_VALUES[name]
        if occurrence == 0:
            return base
        return f"{base}-{occurrence + 1}"
    if occurrence:
        return f"{name}값{occurrence + 1}"
    return f"{name}값"


def pick_fill_data(form: ff.FormCatalog, *, unique_limit: int = 3) -> dict[str, str]:
    data: dict[str, str] = {}
    for name in form.unique_names()[:unique_limit]:
        data[name] = sample_value(name)
    return data


def occurrence_data(form: ff.FormCatalog) -> dict[str, str]:
    data: dict[str, str] = {}
    for name in form.repeated_names():
        total = form.name_counts()[name]
        data[f"{name}[0]"] = sample_value(name, 0)
        if total >= 3:
            data[f"{name}[2]"] = sample_value(name, 2)
        elif total >= 2:
            data[f"{name}[1]"] = sample_value(name, 1)
    return data


def case_shell(
    *,
    ident: str,
    axis: str,
    form: ff.FormCatalog,
    title: str,
    why: str,
    argv: list[str],
    extra: dict[str, Any],
) -> dict[str, Any]:
    body = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": ff.KIND_FILL,
        "id": ident,
        "axis": axis,
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "form": form.ident,
        "title": title,
        "family": form.family,
        "genre": form.genre,
        "sample": form.sample,
        "format": form.fmt,
        "relatedIssue": form.related_issue,
        "why": why,
        "firstFieldName": form.first_name,
        "fieldCount": form.field_count,
        "argv": argv,
        "existingCliOnly": True,
    }
    body.update(extra)
    return body


def emit_form(bundle: Bundle, form: ff.FormCatalog) -> dict[str, Any]:
    survey = ff.survey_fields(form)
    payload = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": ff.KIND_CATALOG,
        "id": form.ident,
        "title": form.title,
        "family": form.family,
        "genre": form.genre,
        "sample": form.sample,
        "format": form.fmt,
        "relatedIssue": form.related_issue,
        "why": form.why,
        "notes": form.notes,
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "fieldCount": form.field_count,
        "firstFieldName": form.first_name,
        "repeatedNames": form.repeated_names(),
        "nameCounts": form.name_counts(),
        "hash": short_hash(json.dumps(survey, ensure_ascii=False, sort_keys=True)),
        "fields": [
            {
                "name": rec.name,
                "guide": rec.guide,
                "value": rec.value,
                "memo": rec.memo,
                "paragraph": rec.paragraph,
                "nested": list(rec.nested),
            }
            for rec in form.fields
        ],
        "surveyFieldCount": survey["fieldCount"],
        "textSecurity": survey["textSecurity"],
    }
    path = bundle.out_root / "fixtures" / "forms" / f"{form.ident}.json"
    write_form_catalog(record(bundle, path), payload)
    bundle.forms.append(payload)
    return payload


def emit_fill_plain(bundle: Bundle, form: ff.FormCatalog) -> None:
    if not form.unique_names() or form.ident not in CORE_FORM_IDS:
        return
    data = pick_fill_data(form)
    env = ff.fill_envelope(form, data, output=f"out/{form.ident}-plain.{ 'hwpx' if form.fmt=='hwpx' else 'hwp' }")
    case = case_shell(
        ident=f"{form.ident}-fill-plain",
        axis="fill-fields",
        form=form,
        title=f"{form.title} 고유 이름 채움",
        why="순번 없는 고유 이름은 단건 채움. notFound/ambiguous 가 비어야 완료.",
        argv=ff.argv_fill(form.sample, json.dumps(data, ensure_ascii=False), output=env.get("output")),
        extra={"data": data, "envelope": env, "exit": ff.exit_for_envelope(env), "gate": ff.gate_single(env)},
    )
    path = bundle.out_root / "fixtures" / "fill" / f"{form.ident}-plain.json"
    write_json(record(bundle, path), case)
    bundle.fills.append(case)


def emit_occurrence(bundle: Bundle, form: ff.FormCatalog) -> None:
    if not form.repeated_names() or form.ident not in CORE_FORM_IDS:
        return
    data = occurrence_data(form)
    env = ff.fill_envelope(form, data, output=f"out/{form.ident}-occ.hwp")
    after = ff.apply_values(form, ff.plan_fill(form, data))
    after_by_name = ff.values_by_name(after)
    targeted_after: dict[str, list[str]] = {}
    untouched: dict[str, list[int]] = {}
    for name in form.repeated_names():
        targeted = {ff.parse_field_key(key)[1] for key in data if ff.parse_field_key(key)[0] == name}
        keep = [i for i, _old in enumerate(form.values_of(name)) if i not in targeted]
        untouched[name] = keep
        targeted_after[name] = after_by_name[name]
        for index in keep:
            assert after_by_name[name][index] == form.values_of(name)[index]
    plain = {name: sample_value(name, 0) for name in form.repeated_names()}
    amb = ff.fill_envelope(form, plain, dry_run=True)
    oor: dict[str, str] = {}
    for name, total in form.name_counts().items():
        if total > 1:
            oor[f"{name}[{total + 100}]"] = "범위밖"
            break
    env_oor = ff.fill_envelope(form, oor, dry_run=True) if oor else None
    case = case_shell(
        ident=f"{form.ident}-occurrence",
        axis="이름[N]",
        form=form,
        title=f"{form.title} 반복 필드 순번 지목",
        why="이름[N] 은 fields 목록 순서 0 기준. 지목하지 않은 칸은 그대로. 순번 없음은 ambiguous, 범위 밖은 notFound.",
        argv=ff.argv_fill(form.sample, json.dumps(data, ensure_ascii=False), output=env.get("output")),
        extra={
            "data": data,
            "envelope": env,
            "exit": ff.exit_for_envelope(env),
            "untouchedOccurrences": untouched,
            "afterTargeted": targeted_after,
            "gate": ff.gate_single(env),
            "ambiguous": {
                "data": plain,
                "envelope": amb,
                "incomplete": bool(amb["ambiguous"]),
            },
            "outOfRange": {
                "data": oor,
                "envelope": env_oor,
            }
            if env_oor is not None
            else None,
        },
    )
    path = bundle.out_root / "fixtures" / "occurrence" / f"{form.ident}.json"
    write_json(record(bundle, path), case)
    bundle.occurrences.append(case)


def emit_dry_run(bundle: Bundle, form: ff.FormCatalog) -> None:
    if form.field_count == 0 or form.ident not in CORE_FORM_IDS:
        return
    data = pick_fill_data(form, unique_limit=1) or occurrence_data(form)
    if not data:
        data = {form.fields[0].name: sample_value(form.fields[0].name)}
    env = ff.fill_envelope(form, data, dry_run=True, output=f"out/{form.ident}-dry.hwp")
    case = case_shell(
        ident=f"{form.ident}-dry-run",
        axis="dry-run",
        form=form,
        title=f"{form.title} dry-run",
        why="--dry-run 은 파일을 만들지 않는다. output/outputFormat 없음.",
        argv=ff.argv_fill(
            form.sample,
            json.dumps(data, ensure_ascii=False),
            output=f"out/{form.ident}-dry.hwp",
            dry_run=True,
        ),
        extra={
            "data": data,
            "envelope": env,
            "writesFile": False,
            "hasOutputKey": "output" in env,
            "exit": ff.exit_for_envelope(env),
        },
    )
    path = bundle.out_root / "fixtures" / "dry_run" / f"{form.ident}.json"
    write_json(record(bundle, path), case)
    bundle.dry_runs.append(case)


def emit_verify(bundle: Bundle, form: ff.FormCatalog) -> None:
    if form.field_count == 0 or form.ident not in CORE_FORM_IDS:
        return
    data = pick_fill_data(form, unique_limit=1) or occurrence_data(form)
    if not data:
        data = {form.fields[0].name: sample_value(form.fields[0].name)}
    env = ff.fill_envelope(
        form,
        data,
        verify=True,
        output=f"out/{form.ident}-verify.{ 'hwpx' if form.fmt=='hwpx' else 'hwp' }",
    )
    case = case_shell(
        ident=f"{form.ident}-verify",
        axis="verify",
        form=form,
        title=f"{form.title} --verify",
        why="저장 직후 재파싱. identical:false 면 exit 3. 산출물은 남는다.",
        argv=ff.argv_fill(
            form.sample,
            json.dumps(data, ensure_ascii=False),
            output=env.get("output"),
            verify=True,
        ),
        extra={
            "data": data,
            "envelope": env,
            "exit": ff.exit_for_envelope(env),
            "leavesOutput": True,
        },
    )
    path = bundle.out_root / "fixtures" / "verify" / f"{form.ident}.json"
    write_json(record(bundle, path), case)
    bundle.verifies.append(case)

    if form.ident not in {"field-01", "gian-1", "reg-80168", "form-01", "hongbo"}:
        return
    missing = {"존재하지않는필드XYZ": "값"}
    env_nf = ff.fill_envelope(form, missing, dry_run=True)
    nf = case_shell(
        ident=f"{form.ident}-notfound",
        axis="notFound",
        form=form,
        title=f"{form.title} 없는 이름",
        why="없는 이름은 조용히 무시하지 않는다. notFound 에 호출자 키.",
        argv=ff.argv_fill(form.sample, json.dumps(missing, ensure_ascii=False), dry_run=True),
        extra={"data": missing, "envelope": env_nf, "exit": ff.exit_for_envelope(env_nf)},
    )
    path = bundle.out_root / "fixtures" / "fill" / f"{form.ident}-notfound.json"
    write_json(record(bundle, path), nf)
    bundle.fills.append(nf)


def emit_hong(bundle: Bundle, form: ff.FormCatalog) -> None:
    if form.field_count == 0 or form.ident not in CORE_FORM_IDS:
        return
    first = ff.first_field_honggildong_request(form)
    ok = ff.honggildong_case(form, first, intended=[form.first_name])
    ok_env = ff.fill_envelope(form, first, output=f"out/{form.ident}-hong.hwp")
    ok_case = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": ff.KIND_HONG,
        "id": f"{form.ident}-hong-first-only",
        "axis": "honggildong-4781",
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "form": form.ident,
        "title": f"{form.title} 첫 필드만 홍길동",
        "why": "T07/#4781. fields[0] 만 홍길동. 다른 칸에 복제하면 clone_forbidden.",
        "sample": form.sample,
        "relatedIssue": "#4781",
        "argv": ff.argv_fill(form.sample, json.dumps(first, ensure_ascii=False), output=ok_env.get("output")),
        "data": first,
        "envelope": ok_env,
        "detect": ok["detect"],
        "afterFirst": ok["afterValues"][0] if ok["afterValues"] else "",
        "otherHongCount": sum(1 for value in ok["afterValues"][1:] if value == ff.HONGGILDONG),
        "allowedClone": False,
        "verdict": ok["detect"]["verdict"],
    }
    path = bundle.out_root / "fixtures" / "honggildong_4781" / f"{form.ident}-first-only.json"
    write_json(record(bundle, path), ok_case)
    bundle.hongs.append(ok_case)

    if form.field_count < 2 or form.ident not in HONG_CLONE_IDS:
        return
    clone = ff.clone_honggildong_request(form)
    bad = ff.honggildong_case(form, clone, intended=[form.first_name])
    bad_env = ff.fill_envelope(form, clone, output=f"out/{form.ident}-hong-clone.hwp")
    bad_case = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": ff.KIND_HONG,
        "id": f"{form.ident}-hong-clone",
        "axis": "honggildong-4781",
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "form": form.ident,
        "title": f"{form.title} 홍길동 전칸 복제 금지",
        "why": "첫 필드 값을 모든 고유 이름에 복사하면 T07 과제 문구를 깨뜨린다.",
        "sample": form.sample,
        "relatedIssue": "#4781",
        "argv": ff.argv_fill(form.sample, json.dumps(clone, ensure_ascii=False), output=bad_env.get("output")),
        "data": clone,
        "envelope": bad_env,
        "detect": bad["detect"],
        "afterFirst": bad["afterValues"][0] if bad["afterValues"] else "",
        "otherHongCount": sum(1 for value in bad["afterValues"][1:] if value == ff.HONGGILDONG),
        "allowedClone": False,
        "verdict": bad["detect"]["verdict"],
    }
    path = bundle.out_root / "fixtures" / "honggildong_4781" / f"{form.ident}-clone.json"
    write_json(record(bundle, path), bad_case)
    bundle.hongs.append(bad_case)


def batch_people(form: ff.FormCatalog) -> list[dict[str, str]]:
    names = form.unique_names()[:2]
    if not names:
        if form.fields:
            names = [form.fields[0].name]
        else:
            return []
    people = [
        ("홍길동", "기획조정실"),
        ("김가온", "운영지원과"),
        ("이도윤", "혁신정책담당관"),
    ]
    rows: list[dict[str, str]] = []
    for person, dept in people:
        row: dict[str, str] = {}
        row[names[0]] = person if names[0] in {"성명", "작성자", "신청자", "기안자", "담당자"} else sample_value(names[0])
        if names[0] in {"성명", "작성자", "신청자", "기안자", "담당자", "출장자"}:
            row[names[0]] = person
        if len(names) > 1:
            if names[1] in {"소속", "부서명", "기안부서", "담당부서"}:
                row[names[1]] = dept
            else:
                row[names[1]] = f"{sample_value(names[1])}-{person}"
        rows.append(row)
    return rows


def emit_batch(bundle: Bundle, form: ff.FormCatalog) -> None:
    if form.field_count == 0 or form.ident not in BATCH_FORM_IDS:
        return
    rows = batch_people(form)
    parsed = [{"row": i, "data": row} for i, row in enumerate(rows)]
    recs = ff.batch_fill(form, parsed, out_dir=f"out/{form.ident}")
    jsonl = "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in rows)
    case = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": ff.KIND_BATCH,
        "id": f"{form.ident}-batch-jsonl",
        "axis": "batch-fill",
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "form": form.ident,
        "title": f"{form.title} JSONL 메일머지",
        "why": "행마다 단건 봉투 + row. 실패 행도 스트림에 남긴다.",
        "sample": form.sample,
        "relatedIssue": "#3719",
        "argv": ff.argv_batch(form.sample, f"data/{form.ident}.jsonl", f"out/{form.ident}"),
        "dataFormat": "jsonl",
        "dataText": jsonl,
        "records": recs,
        "exit": ff.batch_exit(recs),
        "gate": ff.gate_batch(recs),
        "writesFiles": True,
    }
    path = bundle.out_root / "fixtures" / "batch" / f"{form.ident}-jsonl.json"
    write_json(record(bundle, path), case)
    bundle.batches.append(case)

    dry = ff.batch_fill(form, parsed[:2], dry_run=True, out_dir=f"out/{form.ident}-dry")
    dry_case = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": ff.KIND_BATCH,
        "id": f"{form.ident}-batch-dry-run",
        "axis": "batch-dry-run",
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "form": form.ident,
        "title": f"{form.title} batch --dry-run",
        "why": "dry-run 이어도 --out-dir 는 필수. 폴더·파일을 만들지 않는다.",
        "sample": form.sample,
        "relatedIssue": "#3719",
        "argv": ff.argv_batch(
            form.sample, f"data/{form.ident}.jsonl", f"out/{form.ident}-dry", dry_run=True
        ),
        "records": dry,
        "exit": ff.batch_exit(dry),
        "writesFiles": False,
        "anyOutputKey": any("output" in rec for rec in dry),
    }
    path = bundle.out_root / "fixtures" / "batch" / f"{form.ident}-dry-run.json"
    write_json(record(bundle, path), dry_case)
    bundle.batches.append(dry_case)

    if form.ident != "field-01":
        return
    header = list(rows[0].keys())
    csv_buf = [",".join(header)]
    for row in rows[:3]:
        csv_buf.append(",".join(row.get(col, "") for col in header))
    csv_text = "\ufeff" + "\r\n".join(csv_buf) + "\r\n"
    csv_rows = ff.parse_csv_rows(csv_text)
    csv_recs = ff.batch_fill(form, csv_rows, out_dir=f"out/{form.ident}-csv")
    name_field = "작성자"
    named = ff.batch_fill(form, parsed[:3], name_field=name_field, out_dir=f"out/{form.ident}-named")
    extra_case = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": ff.KIND_BATCH,
        "id": f"{form.ident}-batch-csv-name",
        "axis": "batch-fill",
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "form": form.ident,
        "title": f"{form.title} CSV BOM + name-field",
        "why": "BOM 제거와 --name-field 동명 접미는 field-01 한 곳에서 닫는다.",
        "sample": form.sample,
        "relatedIssue": "#3719",
        "csv": {
            "dataText": csv_text,
            "records": csv_recs,
            "stripsBom": True,
            "exit": ff.batch_exit(csv_recs),
        },
        "nameField": {
            "field": name_field,
            "records": named,
            "exit": ff.batch_exit(named),
            "gate": ff.gate_batch(named, name_field=name_field),
        },
    }
    path = bundle.out_root / "fixtures" / "batch" / f"{form.ident}-csv-name.json"
    write_json(record(bundle, path), extra_case)
    bundle.batches.append(extra_case)


def emit_paths(bundle: Bundle) -> None:
    specs = [
        {
            "id": "path-fields-json",
            "axis": "fields",
            "title": "fields --json 조사",
            "why": "채우기 전 읽기 전용 조사. 키가 --data 에 그대로 복제된다.",
            "argv": ["fields", "samples/field-01.hwp", "--json"],
            "exit": 0,
            "writesFile": False,
        },
        {
            "id": "path-fields-human",
            "axis": "fields",
            "title": "fields 사람용 요약",
            "why": "기본 출력은 JSON 이 아니다. --json 전용 계약.",
            "argv": ["fields", "samples/field-01.hwp"],
            "exit": 0,
            "writesFile": False,
        },
        {
            "id": "path-fields-missing",
            "axis": "fields",
            "title": "fields 없는 파일",
            "why": "exit 1, stdout 비움.",
            "argv": ["fields", "없는파일-fields.hwp", "--json"],
            "exit": 1,
            "writesFile": False,
        },
        {
            "id": "path-fields-usage",
            "axis": "fields",
            "title": "fields 인자 없음",
            "why": "exit 2 사용법.",
            "argv": ["fields"],
            "exit": 2,
            "writesFile": False,
        },
        {
            "id": "path-fill-dry-run",
            "axis": "dry-run",
            "title": "fill-fields --dry-run",
            "why": "파일을 만들지 않고 filledCount 만 본다.",
            "argv": [
                "edit",
                "fill-fields",
                "samples/field-01.hwp",
                "--data",
                '{"회사명":"주식회사 A"}',
                "-o",
                "out/dry.hwp",
                "--dry-run",
                "--json",
            ],
            "exit": 0,
            "writesFile": False,
        },
        {
            "id": "path-fill-write",
            "axis": "fill-fields",
            "title": "fill-fields 실채움",
            "why": "산출물을 fields 로 재독해 값을 대조한다.",
            "argv": [
                "edit",
                "fill-fields",
                "samples/field-01.hwp",
                "--data",
                '{"회사명":"주식회사 검증"}',
                "-o",
                "out/write.hwp",
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
        },
        {
            "id": "path-fill-verify",
            "axis": "verify",
            "title": "fill-fields --verify",
            "why": "identical 과 exit 0/3 이 모순이면 안 된다.",
            "argv": [
                "edit",
                "fill-fields",
                "samples/field-01.hwp",
                "--data",
                '{"회사명":"검증사"}',
                "-o",
                "out/verify.hwp",
                "--verify",
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
        },
        {
            "id": "path-fill-occurrence",
            "axis": "이름[N]",
            "title": "피규제집단명[0]/[2] 지목",
            "why": "80168 규제영향분석서 반복 필드.",
            "argv": [
                "edit",
                "fill-fields",
                "samples/80168_regulatory_analysis.hwp",
                "--data",
                '{"피규제집단명[0]":"가상협회 회원사","피규제집단명[2]":"가상조합 조합원"}',
                "-o",
                "out/occ.hwp",
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
        },
        {
            "id": "path-fill-honggildong",
            "axis": "honggildong-4781",
            "title": "첫 필드만 홍길동",
            "why": "T07. 회사명=홍길동. 다른 필드 복제 금지.",
            "argv": [
                "edit",
                "fill-fields",
                "samples/field-01.hwp",
                "--data",
                '{"회사명":"홍길동"}',
                "-o",
                "filled.hwp",
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
            "cloneForbidden": True,
        },
        {
            "id": "path-fill-unknown",
            "axis": "notFound",
            "title": "없는 필드 이름",
            "why": "notFound 에 호출자 문자열만.",
            "argv": [
                "edit",
                "fill-fields",
                "samples/field-01.hwp",
                "--data",
                '{"회사명":"A","존재하지않는필드":"B"}',
                "--dry-run",
                "--json",
            ],
            "exit": 0,
            "writesFile": False,
        },
        {
            "id": "path-fill-missing-file",
            "axis": "runtime",
            "title": "없는 서식 파일",
            "why": "exit 1, 출력 파일 미생성.",
            "argv": [
                "edit",
                "fill-fields",
                "없는파일-edit.hwp",
                "--data",
                '{"a":"b"}',
                "-o",
                "out/missing.hwp",
                "--json",
            ],
            "exit": 1,
            "writesFile": False,
        },
        {
            "id": "path-fill-bad-json",
            "axis": "usage",
            "title": "잘못된 --data JSON",
            "why": "exit 2.",
            "argv": [
                "edit",
                "fill-fields",
                "samples/field-01.hwp",
                "--data",
                "{이건 JSON 이 아님",
                "--dry-run",
            ],
            "exit": 2,
            "writesFile": False,
        },
        {
            "id": "path-fill-missing-data",
            "axis": "usage",
            "title": "--data 없음",
            "why": "exit 2.",
            "argv": ["edit", "fill-fields", "samples/field-01.hwp"],
            "exit": 2,
            "writesFile": False,
        },
        {
            "id": "path-fill-data-at-file",
            "axis": "fill-fields",
            "title": "--data @파일 UTF-8",
            "why": "CP949 면 stream did not contain valid UTF-8, exit 1.",
            "argv": [
                "edit",
                "fill-fields",
                "samples/field-01.hwp",
                "--data",
                "@row.json",
                "-o",
                "out/at.hwp",
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
            "encoding": "utf-8",
        },
        {
            "id": "path-fill-hwpx-preserve",
            "axis": "format",
            "title": "HWPX 입력 형식 보존",
            "why": "#3383. outputFormat=hwpx.",
            "argv": [
                "edit",
                "fill-fields",
                "tools/forms/일반기안문_서식.hwpx",
                "--data",
                "@tools/forms/일반기안문_예시값.json",
                "-o",
                "out/gian.hwpx",
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
            "outputFormat": "hwpx",
        },
        {
            "id": "path-fill-default-next-to-input",
            "axis": "fill-fields",
            "title": "-o 생략 기본 산출",
            "why": "입력 옆 <이름>_filled.hwp(.hwpx).",
            "argv": [
                "edit",
                "fill-fields",
                "samples/field-01.hwp",
                "--data",
                '{"회사명":"옆파일"}',
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
            "defaultOutput": "field-01_filled.hwp",
        },
        {
            "id": "path-batch-jsonl",
            "axis": "batch-fill",
            "title": "batch fill JSONL",
            "why": "stdin 을 읽지 않는다. --form + --data 파일.",
            "argv": [
                "batch",
                "fill",
                "--form",
                "samples/field-01.hwp",
                "--data",
                "rows.jsonl",
                "--out-dir",
                "out/filled",
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
            "readsStdin": False,
        },
        {
            "id": "path-batch-csv",
            "axis": "batch-fill",
            "title": "batch fill CSV",
            "why": "확장자로 jsonl/csv 판별.",
            "argv": [
                "batch",
                "fill",
                "--form",
                "samples/field-01.hwp",
                "--data",
                "명단.csv",
                "--out-dir",
                "out/filled",
                "--name-field",
                "작성자",
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
        },
        {
            "id": "path-batch-dry-run",
            "axis": "batch-dry-run",
            "title": "batch fill --dry-run",
            "why": "out-dir 필수, 파일 없음.",
            "argv": [
                "batch",
                "fill",
                "--form",
                "samples/field-01.hwp",
                "--data",
                "rows.jsonl",
                "--out-dir",
                "out/filled",
                "--dry-run",
                "--json",
            ],
            "exit": 0,
            "writesFile": False,
        },
        {
            "id": "path-batch-verify",
            "axis": "verify",
            "title": "batch fill --verify",
            "why": "행별 verify. 하나라도 identical:false 면 exit 3.",
            "argv": [
                "batch",
                "fill",
                "--form",
                "samples/field-01.hwp",
                "--data",
                "rows.jsonl",
                "--out-dir",
                "out/filled",
                "--verify",
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
        },
        {
            "id": "path-batch-empty-data",
            "axis": "usage",
            "title": "헤더만 있는 빈 데이터",
            "why": "exit 2. 상류 명단 0건.",
            "argv": [
                "batch",
                "fill",
                "--form",
                "samples/field-01.hwp",
                "--data",
                "empty.csv",
                "--out-dir",
                "out/filled",
                "--json",
            ],
            "exit": 2,
            "writesFile": False,
        },
        {
            "id": "path-batch-threads",
            "axis": "batch-fill",
            "title": "batch fill --threads",
            "why": "단건과 같은 의미의 선택 옵션.",
            "argv": [
                "batch",
                "fill",
                "--form",
                "samples/field-01.hwp",
                "--data",
                "rows.jsonl",
                "--out-dir",
                "out/filled",
                "--threads",
                "4",
                "--json",
            ],
            "exit": 0,
            "writesFile": True,
        },
        {
            "id": "path-unknown-edit",
            "axis": "usage",
            "title": "없는 edit 하위명령",
            "why": "새 CLI 를 만들지 않는다. exit 2.",
            "argv": ["edit", "no-such-action"],
            "exit": 2,
            "writesFile": False,
        },
    ]
    for spec in specs:
        payload = {
            "schemaVersion": SCHEMA_VERSION,
            "kind": ff.KIND_PATH,
            "claim": CLAIM_ID,
            "generator": GENERATOR,
            "existingCliOnly": True,
            **spec,
        }
        path = bundle.out_root / "fixtures" / "paths" / f"{spec['id']}.json"
        write_json(record(bundle, path), payload)
        bundle.paths.append(payload)


def emit_schemas(bundle: Bundle) -> None:
    schemas = {
        "form_catalog.v1.json": {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.form_fill.form_catalog.v1",
            "title": "fields survey catalog",
            "type": "object",
            "required": ["schemaVersion", "kind", "id", "sample", "fields", "survey"],
            "properties": {
                "schemaVersion": {"const": "1.0"},
                "kind": {"const": ff.KIND_CATALOG},
                "id": {"type": "string"},
                "sample": {"type": "string"},
                "fieldCount": {"type": "integer", "minimum": 0},
                "firstFieldName": {"type": "string"},
                "fields": {"type": "array"},
                "survey": {"type": "object"},
            },
        },
        "fill_case.v1.json": {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.form_fill.fill_case.v1",
            "title": "fill-fields envelope case",
            "type": "object",
            "required": ["schemaVersion", "kind", "id", "axis", "argv", "envelope"],
            "properties": {
                "axis": {
                    "enum": [
                        "fill-fields",
                        "이름[N]",
                        "ambiguous",
                        "notFound",
                        "dry-run",
                        "verify",
                    ]
                },
                "existingCliOnly": {"const": True},
                "envelope": {
                    "type": "object",
                    "required": ["schemaVersion", "dryRun", "filledCount", "filled", "notFound", "ambiguous"],
                },
            },
        },
        "batch_row.v1.json": {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.form_fill.batch_row.v1",
            "title": "batch fill row case",
            "type": "object",
            "required": ["schemaVersion", "kind", "id", "argv", "records"],
            "properties": {
                "axis": {
                    "enum": [
                        "batch-fill",
                        "batch-dry-run",
                        "batch-name-field",
                    ]
                },
                "records": {"type": "array"},
            },
        },
        "path_contract.v1.json": {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.form_fill.path_contract.v1",
            "title": "existing CLI argv contract",
            "type": "object",
            "required": ["schemaVersion", "kind", "id", "argv", "exit"],
            "properties": {
                "existingCliOnly": {"const": True},
                "argv": {"type": "array", "items": {"type": "string"}},
                "exit": {"type": "integer", "enum": [0, 1, 2, 3]},
            },
        },
        "honggildong_4781.v1.json": {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.form_fill.honggildong_4781.v1",
            "title": "T07 first-field 홍길동 must not be cloned",
            "type": "object",
            "required": ["schemaVersion", "kind", "id", "detect", "verdict", "allowedClone"],
            "properties": {
                "relatedIssue": {"const": "#4781"},
                "allowedClone": {"const": False},
                "verdict": {"enum": ["pass", "clone_forbidden"]},
                "detect": {
                    "type": "object",
                    "required": ["firstFieldOk", "cloneCount", "verdict"],
                },
            },
        },
    }
    for name, schema in schemas.items():
        path = bundle.out_root / "schema" / name
        write_json(record(bundle, path), schema)


def tsv(rows: list[list[Any]]) -> str:
    return "\n".join("\t".join(ff.nfc(str(cell)) for cell in row) for row in rows) + "\n"


def emit_reports(bundle: Bundle) -> None:
    axis_counts = Counter(case.get("axis", "") for case in bundle.fills + bundle.occurrences + bundle.dry_runs + bundle.verifies + bundle.batches + bundle.hongs + bundle.paths)
    summary = {
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "generatedAt": bundle.generated_at,
        "formCount": len(bundle.forms),
        "fillCount": len(bundle.fills),
        "occurrenceCount": len(bundle.occurrences),
        "dryRunCount": len(bundle.dry_runs),
        "verifyCount": len(bundle.verifies),
        "batchCount": len(bundle.batches),
        "honggildongCount": len(bundle.hongs),
        "pathCount": len(bundle.paths),
        "writtenCount": len(bundle.written),
        "axisCounts": dict(axis_counts),
        "hongPass": sum(1 for c in bundle.hongs if c.get("verdict") == "pass"),
        "hongCloneForbidden": sum(1 for c in bundle.hongs if c.get("verdict") == "clone_forbidden"),
        "existingCliOnly": True,
        "inventedFillLogic": False,
        "touchedGym": False,
    }
    write_json(record(bundle, bundle.out_root / "reports" / "fatten_summary.json"), summary)

    md = [
        f"# M-fill 픽스처 고도화 요약",
        "",
        f"- 생성 시각: {bundle.generated_at}",
        f"- 서식 카탈로그: **{len(bundle.forms)}**",
        f"- fill-fields 케이스: **{len(bundle.fills)}**",
        f"- 이름[N]/ambiguous: **{len(bundle.occurrences)}**",
        f"- dry-run: **{len(bundle.dry_runs)}**",
        f"- verify: **{len(bundle.verifies)}**",
        f"- batch fill: **{len(bundle.batches)}**",
        f"- #4781 홍길동: **{len(bundle.hongs)}** (pass {summary['hongPass']} / clone_forbidden {summary['hongCloneForbidden']})",
        f"- CLI 경로: **{len(bundle.paths)}**",
        f"- 기록 파일: **{len(bundle.written)}**",
        "",
        "기존 CLI 만 사용한다. DocumentCore 채움 로직을 발명하지 않는다. gym 없음.",
        "",
        "## 축 집계",
        "",
        "| 축 | 건수 |",
        "|---|---:|",
    ]
    for axis, count in sorted(axis_counts.items()):
        md.append(f"| {md_cell(axis)} | {count} |")
    write_text(record(bundle, bundle.out_root / "reports" / "fatten_summary.md"), "\n".join(md) + "\n")

    axis_md = ["# 축별 건수", "", "| 축 | 건수 |", "|---|---:|"]
    for axis, count in sorted(axis_counts.items()):
        axis_md.append(f"| {md_cell(axis)} | {count} |")
    write_text(record(bundle, bundle.out_root / "reports" / "axis_counts.md"), "\n".join(axis_md) + "\n")

    hong_md = [
        "# T07 / #4781 첫 필드 홍길동 복제 금지",
        "",
        "첫 필드만 홍길동이어야 한다. 다른 칸에 같은 값을 복사하면 `clone_forbidden`.",
        "",
        "| 식별자 | 서식 | 첫 필드 | firstOk | cloneCount | 판정 |",
        "|---|---|---|---|---:|---|",
    ]
    for case in bundle.hongs:
        detect = case.get("detect") or {}
        hong_md.append(
            "| {id} | {form} | {first} | {ok} | {n} | {v} |".format(
                id=md_cell(case.get("id")),
                form=md_cell(case.get("form")),
                first=md_cell(detect.get("firstFieldName")),
                ok=md_cell(detect.get("firstFieldOk")),
                n=detect.get("cloneCount", 0),
                v=md_cell(case.get("verdict")),
            )
        )
    write_text(record(bundle, bundle.out_root / "reports" / "honggildong_4781.md"), "\n".join(hong_md) + "\n")

    write_text(
        record(bundle, bundle.out_root / "tables" / "forms.tsv"),
        tsv(
            [["id", "family", "sample", "format", "fieldCount", "firstField", "repeated", "issue"]]
            + [
                [
                    f["id"],
                    f["family"],
                    f["sample"],
                    f["format"],
                    f["fieldCount"],
                    f["firstFieldName"],
                    ",".join(f["repeatedNames"]),
                    f["relatedIssue"],
                ]
                for f in bundle.forms
            ]
        ),
    )
    write_text(
        record(bundle, bundle.out_root / "tables" / "fill_cases.tsv"),
        tsv(
            [["id", "axis", "form", "filledCount", "notFound", "ambiguous", "dryRun", "exit"]]
            + [
                [
                    c["id"],
                    c["axis"],
                    c.get("form", ""),
                    (c.get("envelope") or {}).get("filledCount", ""),
                    len((c.get("envelope") or {}).get("notFound") or []),
                    len((c.get("envelope") or {}).get("ambiguous") or []),
                    (c.get("envelope") or {}).get("dryRun", ""),
                    c.get("exit", ""),
                ]
                for c in bundle.fills + bundle.dry_runs + bundle.verifies
            ]
        ),
    )
    # occurrence/batch 상세는 픽스처 JSON 이 정본. 표는 서식·채움·홍길동·경로만.
    write_text(
        record(bundle, bundle.out_root / "tables" / "honggildong_4781.tsv"),
        tsv(
            [["id", "form", "firstField", "firstOk", "cloneCount", "verdict"]]
            + [
                [
                    c["id"],
                    c.get("form", ""),
                    (c.get("detect") or {}).get("firstFieldName", ""),
                    (c.get("detect") or {}).get("firstFieldOk", ""),
                    (c.get("detect") or {}).get("cloneCount", ""),
                    c.get("verdict", ""),
                ]
                for c in bundle.hongs
            ]
        ),
    )
    write_text(
        record(bundle, bundle.out_root / "tables" / "paths.tsv"),
        tsv(
            [["id", "axis", "exit", "writesFile", "argv"]]
            + [
                [c["id"], c["axis"], c["exit"], c.get("writesFile", ""), " ".join(c["argv"])]
                for c in bundle.paths
            ]
        ),
    )

    transcript = {
        "claim": CLAIM_ID,
        "existingCliOnly": True,
        "commands": [
            {"id": c["id"], "argv": c["argv"], "exit": c["exit"], "axis": c["axis"]}
            for c in bundle.paths
        ],
    }
    write_json(record(bundle, bundle.out_root / "transcripts" / "cli_paths.json"), transcript)

    index = {
        "claim": CLAIM_ID,
        "generatedAt": bundle.generated_at,
        "written": bundle.written,
    }
    write_json(record(bundle, bundle.out_root / "fixtures" / "index.json"), index)


def emit_indexes(bundle: Bundle) -> None:
    def dump(name: str, rows: list[dict[str, Any]]) -> None:
        path = bundle.out_root / "fixtures" / f"index-{name}.jsonl"
        text = "".join(json.dumps({"id": r.get("id"), "axis": r.get("axis"), "form": r.get("form")}, ensure_ascii=False) + "\n" for r in rows)
        write_text(record(bundle, path), text)

    dump("forms", bundle.forms)
    dump("fill", bundle.fills)
    dump("occurrence", bundle.occurrences)
    dump("dry_run", bundle.dry_runs)
    dump("verify", bundle.verifies)
    dump("batch", bundle.batches)
    dump("honggildong", bundle.hongs)
    dump("paths", bundle.paths)


def run(out_root: Path) -> Bundle:
    bundle = Bundle(generated_at=utc_now(), out_root=out_root)
    emit_schemas(bundle)
    for form in catalog_mod.FORMS():
        emit_form(bundle, form)
        emit_fill_plain(bundle, form)
        emit_occurrence(bundle, form)
        emit_dry_run(bundle, form)
        emit_verify(bundle, form)
        emit_hong(bundle, form)
        emit_batch(bundle, form)
    emit_paths(bundle)
    emit_reports(bundle)
    return bundle


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="M-fill fixture fatten")
    parser.add_argument("--out", type=Path, default=HERE)
    args = parser.parse_args(argv)
    bundle = run(args.out)
    print(
        json.dumps(
            {
                "claim": CLAIM_ID,
                "written": len(bundle.written),
                "forms": len(bundle.forms),
                "fills": len(bundle.fills),
                "occurrence": len(bundle.occurrences),
                "dryRun": len(bundle.dry_runs),
                "verify": len(bundle.verifies),
                "batch": len(bundle.batches),
                "honggildong": len(bundle.hongs),
                "paths": len(bundle.paths),
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
