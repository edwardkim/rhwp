# extra pins

LINE = NAME = FIELD = CELL = ITEM = SHAPE = LEAF = None

def test_x0215_r_source():
    '''실측 잎'''
    assert LEAF == "C:\\Users\\swsz9\\rhwp-agent-cli-pack\\samples\\HWP5-password-123456.hwpx"

def test_x0215_r_runs_0_argv_0():
    '''실측 잎'''
    assert LEAF == "tables"

def test_x0215_r_runs_0_argv_1():
    '''실측 잎'''
    assert LEAF == "--json"

def test_x0215_r_runs_0_exit():
    '''실측 잎'''
    assert LEAF == 1

def test_x0215_r_runs_0_stdout():
    '''실측 잎'''
    assert LEAF == null

def test_x0215_r_runs_0_stderr_l0():
    '''stderr 줄 실측'''
    assert LINE == "오류: 문서를 열 수 없습니다 - C:\\Users\\swsz9\\rhwp-agent-cli-pack\\samples\\HWP5-password-123456.hwpx: 유효하지 않은 파일: 비밀번호가 필요한 암호 문서입니다 (parse_document_with_password 또는 parse_hwp_with_password 로 비밀번호를 전달하세요)"

def test_x0215_r_runs_1_argv_0():
    '''실측 잎'''
    assert LEAF == "table-count"

def test_x0215_r_runs_1_argv_1():
    '''실측 잎'''
    assert LEAF == "--json"

def test_x0215_r_runs_1_exit():
    '''실측 잎'''
    assert LEAF == 1

def test_x0215_r_runs_1_stdout():
    '''실측 잎'''
    assert LEAF == null

def test_x0215_r_runs_1_stderr_l0():
    '''stderr 줄 실측'''
    assert LINE == "오류: 문서를 열 수 없습니다 - C:\\Users\\swsz9\\rhwp-agent-cli-pack\\samples\\HWP5-password-123456.hwpx: 유효하지 않은 파일: 비밀번호가 필요한 암호 문서입니다 (parse_document_with_password 또는 parse_hwp_with_password 로 비밀번호를 전달하세요)"

def test_x0215_r_runs_2_argv_0():
    '''실측 잎'''
    assert LEAF == "table-inspect"

def test_x0215_r_runs_2_argv_1():
    '''실측 잎'''
    assert LEAF == "--json"

def test_x0215_r_runs_2_exit():
    '''실측 잎'''
    assert LEAF == 1

def test_x0215_r_runs_2_stdout():
    '''실측 잎'''
    assert LEAF == null

def test_x0215_r_runs_2_stderr_l0():
    '''stderr 줄 실측'''
    assert LINE == "오류: 문서를 열 수 없습니다 - C:\\Users\\swsz9\\rhwp-agent-cli-pack\\samples\\HWP5-password-123456.hwpx: 유효하지 않은 파일: 비밀번호가 필요한 암호 문서입니다 (parse_document_with_password 또는 parse_hwp_with_password 로 비밀번호를 전달하세요)"

def test_x0215_r_runs_3_argv_0():
    '''실측 잎'''
    assert LEAF == "merged-tables"

def test_x0215_r_runs_3_argv_1():
    '''실측 잎'''
    assert LEAF == "--json"

def test_x0215_r_runs_3_exit():
    '''실측 잎'''
    assert LEAF == 1

def test_x0215_r_runs_3_stdout():
    '''실측 잎'''
    assert LEAF == null

def test_x0215_r_runs_3_stderr_l0():
    '''stderr 줄 실측'''
    assert LINE == "오류: 문서를 열 수 없습니다 - C:\\Users\\swsz9\\rhwp-agent-cli-pack\\samples\\HWP5-password-123456.hwpx: 유효하지 않은 파일: 비밀번호가 필요한 암호 문서입니다 (parse_document_with_password 또는 parse_hwp_with_password 로 비밀번호를 전달하세요)"

