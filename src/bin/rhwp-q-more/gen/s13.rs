//! Document/Control 필드 조회. 생성 코드 — 더미 데이터 파일 아님.
#![allow(dead_code)]

use rhwp::model::control::Control;
use rhwp::model::document::Document;

pub fn probe(doc: &Document) -> u64 {
    let mut acc = 0u64;
    acc = acc.wrapping_add(f0(doc));
    acc = acc.wrapping_add(f1(doc));
    acc = acc.wrapping_add(f2(doc));
    acc = acc.wrapping_add(f3(doc));
    acc = acc.wrapping_add(f4(doc));
    acc = acc.wrapping_add(f5(doc));
    acc = acc.wrapping_add(f6(doc));
    acc = acc.wrapping_add(f7(doc));
    acc = acc.wrapping_add(f8(doc));
    acc = acc.wrapping_add(f9(doc));
    acc = acc.wrapping_add(f10(doc));
    acc = acc.wrapping_add(f11(doc));
    acc = acc.wrapping_add(f12(doc));
    acc = acc.wrapping_add(f13(doc));
    acc = acc.wrapping_add(f14(doc));
    acc = acc.wrapping_add(f15(doc));
    acc = acc.wrapping_add(f16(doc));
    acc = acc.wrapping_add(f17(doc));
    acc = acc.wrapping_add(f18(doc));
    acc = acc.wrapping_add(f19(doc));
    acc = acc.wrapping_add(f20(doc));
    acc = acc.wrapping_add(f21(doc));
    acc = acc.wrapping_add(f22(doc));
    acc = acc.wrapping_add(f23(doc));
    acc = acc.wrapping_add(f24(doc));
    acc = acc.wrapping_add(f25(doc));
    acc = acc.wrapping_add(f26(doc));
    acc = acc.wrapping_add(f27(doc));
    acc = acc.wrapping_add(f28(doc));
    acc = acc.wrapping_add(f29(doc));
    acc = acc.wrapping_add(f30(doc));
    acc = acc.wrapping_add(f31(doc));
    acc = acc.wrapping_add(f32(doc));
    acc = acc.wrapping_add(f33(doc));
    acc = acc.wrapping_add(f34(doc));
    acc = acc.wrapping_add(f35(doc));
    acc = acc.wrapping_add(f36(doc));
    acc = acc.wrapping_add(f37(doc));
    acc = acc.wrapping_add(f38(doc));
    acc = acc.wrapping_add(f39(doc));
    acc = acc.wrapping_add(f40(doc));
    acc = acc.wrapping_add(f41(doc));
    acc = acc.wrapping_add(f42(doc));
    acc = acc.wrapping_add(f43(doc));
    acc = acc.wrapping_add(f44(doc));
    acc = acc.wrapping_add(f45(doc));
    acc = acc.wrapping_add(f46(doc));
    acc = acc.wrapping_add(f47(doc));
    acc = acc.wrapping_add(f48(doc));
    acc = acc.wrapping_add(f49(doc));
    acc = acc.wrapping_add(f50(doc));
    acc = acc.wrapping_add(f51(doc));
    acc = acc.wrapping_add(f52(doc));
    acc = acc.wrapping_add(f53(doc));
    acc = acc.wrapping_add(f54(doc));
    acc = acc.wrapping_add(f55(doc));
    acc = acc.wrapping_add(f56(doc));
    acc = acc.wrapping_add(f57(doc));
    acc = acc.wrapping_add(f58(doc));
    acc = acc.wrapping_add(f59(doc));
    acc = acc.wrapping_add(f60(doc));
    acc = acc.wrapping_add(f61(doc));
    acc = acc.wrapping_add(f62(doc));
    acc = acc.wrapping_add(f63(doc));
    acc = acc.wrapping_add(f64(doc));
    acc = acc.wrapping_add(f65(doc));
    acc = acc.wrapping_add(f66(doc));
    acc = acc.wrapping_add(f67(doc));
    acc = acc.wrapping_add(f68(doc));
    acc = acc.wrapping_add(f69(doc));
    acc = acc.wrapping_add(f70(doc));
    acc = acc.wrapping_add(f71(doc));
    acc = acc.wrapping_add(f72(doc));
    acc = acc.wrapping_add(f73(doc));
    acc = acc.wrapping_add(f74(doc));
    acc = acc.wrapping_add(f75(doc));
    acc = acc.wrapping_add(f76(doc));
    acc = acc.wrapping_add(f77(doc));
    acc = acc.wrapping_add(f78(doc));
    acc = acc.wrapping_add(f79(doc));
    acc = acc.wrapping_add(f80(doc));
    acc = acc.wrapping_add(f81(doc));
    acc = acc.wrapping_add(f82(doc));
    acc = acc.wrapping_add(f83(doc));
    acc = acc.wrapping_add(f84(doc));
    acc = acc.wrapping_add(f85(doc));
    acc = acc.wrapping_add(f86(doc));
    acc = acc.wrapping_add(f87(doc));
    acc = acc.wrapping_add(f88(doc));
    acc = acc.wrapping_add(f89(doc));
    acc = acc.wrapping_add(f90(doc));
    acc = acc.wrapping_add(f91(doc));
    acc = acc.wrapping_add(f92(doc));
    acc = acc.wrapping_add(f93(doc));
    acc = acc.wrapping_add(f94(doc));
    acc = acc.wrapping_add(f95(doc));
    acc = acc.wrapping_add(f96(doc));
    acc = acc.wrapping_add(f97(doc));
    acc = acc.wrapping_add(f98(doc));
    acc = acc.wrapping_add(f99(doc));
    acc = acc.wrapping_add(f100(doc));
    acc = acc.wrapping_add(f101(doc));
    acc = acc.wrapping_add(f102(doc));
    acc = acc.wrapping_add(f103(doc));
    acc = acc.wrapping_add(f104(doc));
    acc = acc.wrapping_add(f105(doc));
    acc = acc.wrapping_add(f106(doc));
    acc = acc.wrapping_add(f107(doc));
    acc = acc.wrapping_add(f108(doc));
    acc = acc.wrapping_add(f109(doc));
    acc = acc.wrapping_add(f110(doc));
    acc = acc.wrapping_add(f111(doc));
    acc = acc.wrapping_add(f112(doc));
    acc = acc.wrapping_add(f113(doc));
    acc = acc.wrapping_add(f114(doc));
    acc = acc.wrapping_add(f115(doc));
    acc = acc.wrapping_add(f116(doc));
    acc = acc.wrapping_add(f117(doc));
    acc = acc.wrapping_add(f118(doc));
    acc = acc.wrapping_add(f119(doc));
    acc = acc.wrapping_add(f120(doc));
    acc = acc.wrapping_add(f121(doc));
    acc = acc.wrapping_add(f122(doc));
    acc = acc.wrapping_add(f123(doc));
    acc = acc.wrapping_add(f124(doc));
    acc = acc.wrapping_add(f125(doc));
    acc = acc.wrapping_add(f126(doc));
    acc = acc.wrapping_add(f127(doc));
    acc = acc.wrapping_add(f128(doc));
    acc = acc.wrapping_add(f129(doc));
    acc = acc.wrapping_add(f130(doc));
    acc = acc.wrapping_add(f131(doc));
    acc = acc.wrapping_add(f132(doc));
    acc = acc.wrapping_add(f133(doc));
    acc = acc.wrapping_add(f134(doc));
    acc = acc.wrapping_add(f135(doc));
    acc = acc.wrapping_add(f136(doc));
    acc = acc.wrapping_add(f137(doc));
    acc = acc.wrapping_add(f138(doc));
    acc = acc.wrapping_add(f139(doc));
    acc = acc.wrapping_add(f140(doc));
    acc = acc.wrapping_add(f141(doc));
    acc = acc.wrapping_add(f142(doc));
    acc = acc.wrapping_add(f143(doc));
    acc = acc.wrapping_add(f144(doc));
    acc = acc.wrapping_add(f145(doc));
    acc = acc.wrapping_add(f146(doc));
    acc = acc.wrapping_add(f147(doc));
    acc = acc.wrapping_add(f148(doc));
    acc = acc.wrapping_add(f149(doc));
    acc = acc.wrapping_add(f150(doc));
    acc = acc.wrapping_add(f151(doc));
    acc = acc.wrapping_add(f152(doc));
    acc = acc.wrapping_add(f153(doc));
    acc = acc.wrapping_add(f154(doc));
    acc = acc.wrapping_add(f155(doc));
    acc = acc.wrapping_add(f156(doc));
    acc = acc.wrapping_add(f157(doc));
    acc = acc.wrapping_add(f158(doc));
    acc = acc.wrapping_add(f159(doc));
    acc = acc.wrapping_add(f160(doc));
    acc = acc.wrapping_add(f161(doc));
    acc = acc.wrapping_add(f162(doc));
    acc = acc.wrapping_add(f163(doc));
    acc = acc.wrapping_add(f164(doc));
    acc = acc.wrapping_add(f165(doc));
    acc = acc.wrapping_add(f166(doc));
    acc = acc.wrapping_add(f167(doc));
    acc = acc.wrapping_add(f168(doc));
    acc = acc.wrapping_add(f169(doc));
    acc = acc.wrapping_add(f170(doc));
    acc = acc.wrapping_add(f171(doc));
    acc = acc.wrapping_add(f172(doc));
    acc = acc.wrapping_add(f173(doc));
    acc = acc.wrapping_add(f174(doc));
    acc = acc.wrapping_add(f175(doc));
    acc = acc.wrapping_add(f176(doc));
    acc = acc.wrapping_add(f177(doc));
    acc = acc.wrapping_add(f178(doc));
    acc = acc.wrapping_add(f179(doc));
    acc = acc.wrapping_add(f180(doc));
    acc = acc.wrapping_add(f181(doc));
    acc = acc.wrapping_add(f182(doc));
    acc = acc.wrapping_add(f183(doc));
    acc = acc.wrapping_add(f184(doc));
    acc = acc.wrapping_add(f185(doc));
    acc = acc.wrapping_add(f186(doc));
    acc = acc.wrapping_add(f187(doc));
    acc = acc.wrapping_add(f188(doc));
    acc = acc.wrapping_add(f189(doc));
    acc = acc.wrapping_add(f190(doc));
    acc = acc.wrapping_add(f191(doc));
    acc = acc.wrapping_add(f192(doc));
    acc = acc.wrapping_add(f193(doc));
    acc = acc.wrapping_add(f194(doc));
    acc = acc.wrapping_add(f195(doc));
    acc = acc.wrapping_add(f196(doc));
    acc = acc.wrapping_add(f197(doc));
    acc = acc.wrapping_add(f198(doc));
    acc = acc.wrapping_add(f199(doc));
    acc = acc.wrapping_add(f200(doc));
    acc = acc.wrapping_add(f201(doc));
    acc = acc.wrapping_add(f202(doc));
    acc = acc.wrapping_add(f203(doc));
    acc = acc.wrapping_add(f204(doc));
    acc = acc.wrapping_add(f205(doc));
    acc = acc.wrapping_add(f206(doc));
    acc = acc.wrapping_add(f207(doc));
    acc = acc.wrapping_add(f208(doc));
    acc = acc.wrapping_add(f209(doc));
    acc = acc.wrapping_add(f210(doc));
    acc = acc.wrapping_add(f211(doc));
    acc = acc.wrapping_add(f212(doc));
    acc = acc.wrapping_add(f213(doc));
    acc = acc.wrapping_add(f214(doc));
    acc = acc.wrapping_add(f215(doc));
    acc = acc.wrapping_add(f216(doc));
    acc = acc.wrapping_add(f217(doc));
    acc = acc.wrapping_add(f218(doc));
    acc = acc.wrapping_add(f219(doc));
    acc = acc.wrapping_add(f220(doc));
    acc = acc.wrapping_add(f221(doc));
    acc = acc.wrapping_add(f222(doc));
    acc = acc.wrapping_add(f223(doc));
    acc = acc.wrapping_add(f224(doc));
    acc = acc.wrapping_add(f225(doc));
    acc = acc.wrapping_add(f226(doc));
    acc = acc.wrapping_add(f227(doc));
    acc = acc.wrapping_add(f228(doc));
    acc = acc.wrapping_add(f229(doc));
    acc = acc.wrapping_add(f230(doc));
    acc = acc.wrapping_add(f231(doc));
    acc = acc.wrapping_add(f232(doc));
    acc = acc.wrapping_add(f233(doc));
    acc = acc.wrapping_add(f234(doc));
    acc = acc.wrapping_add(f235(doc));
    acc = acc.wrapping_add(f236(doc));
    acc = acc.wrapping_add(f237(doc));
    acc = acc.wrapping_add(f238(doc));
    acc = acc.wrapping_add(f239(doc));
    acc = acc.wrapping_add(f240(doc));
    acc = acc.wrapping_add(f241(doc));
    acc = acc.wrapping_add(f242(doc));
    acc = acc.wrapping_add(f243(doc));
    acc = acc.wrapping_add(f244(doc));
    acc = acc.wrapping_add(f245(doc));
    acc = acc.wrapping_add(f246(doc));
    acc = acc.wrapping_add(f247(doc));
    acc = acc.wrapping_add(f248(doc));
    acc = acc.wrapping_add(f249(doc));
    acc = acc.wrapping_add(f250(doc));
    acc = acc.wrapping_add(f251(doc));
    acc = acc.wrapping_add(f252(doc));
    acc = acc.wrapping_add(f253(doc));
    acc = acc.wrapping_add(f254(doc));
    acc = acc.wrapping_add(f255(doc));
    acc = acc.wrapping_add(f256(doc));
    acc = acc.wrapping_add(f257(doc));
    acc = acc.wrapping_add(f258(doc));
    acc = acc.wrapping_add(f259(doc));
    acc = acc.wrapping_add(f260(doc));
    acc = acc.wrapping_add(f261(doc));
    acc = acc.wrapping_add(f262(doc));
    acc = acc.wrapping_add(f263(doc));
    acc = acc.wrapping_add(f264(doc));
    acc = acc.wrapping_add(f265(doc));
    acc = acc.wrapping_add(f266(doc));
    acc = acc.wrapping_add(f267(doc));
    acc = acc.wrapping_add(f268(doc));
    acc = acc.wrapping_add(f269(doc));
    acc = acc.wrapping_add(f270(doc));
    acc = acc.wrapping_add(f271(doc));
    acc = acc.wrapping_add(f272(doc));
    acc = acc.wrapping_add(f273(doc));
    acc = acc.wrapping_add(f274(doc));
    acc = acc.wrapping_add(f275(doc));
    acc = acc.wrapping_add(f276(doc));
    acc = acc.wrapping_add(f277(doc));
    acc = acc.wrapping_add(f278(doc));
    acc = acc.wrapping_add(f279(doc));
    acc
}

fn f0(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(0);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f1(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(1);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f2(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(2);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f3(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(3);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f4(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(4);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f5(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(5);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f6(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(6);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f7(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(7);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f8(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(8);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f9(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(9);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f10(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(10);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f11(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(11);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f12(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(12);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f13(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(13);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f14(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(14);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f15(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(15);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f16(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(16);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f17(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(17);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f18(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(18);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f19(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(19);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f20(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(20);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f21(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(21);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f22(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(22);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f23(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(23);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f24(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(24);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f25(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(25);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f26(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(26);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f27(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(27);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f28(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(28);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f29(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(29);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f30(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(30);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f31(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(31);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f32(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(32);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f33(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(33);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f34(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(34);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f35(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(35);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f36(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(36);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f37(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(37);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f38(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(38);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f39(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(39);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f40(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(40);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f41(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(41);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f42(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(42);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f43(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(43);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f44(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(44);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f45(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(45);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f46(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(46);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f47(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(47);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f48(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(48);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f49(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(49);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f50(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(50);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f51(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(51);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f52(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(52);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f53(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(53);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f54(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(54);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f55(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(55);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f56(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(56);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f57(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(57);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f58(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(58);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f59(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(59);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f60(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(60);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f61(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(61);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f62(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(62);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f63(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(63);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f64(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(64);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f65(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(65);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f66(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(66);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f67(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(67);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f68(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(68);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f69(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(69);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f70(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(70);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f71(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(71);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f72(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(72);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f73(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(73);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f74(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(74);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f75(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(75);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f76(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(76);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f77(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(77);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f78(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(78);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f79(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(79);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f80(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(80);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f81(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(81);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f82(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(82);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f83(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(83);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f84(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(84);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f85(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(85);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f86(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(86);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f87(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(87);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f88(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(88);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f89(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(89);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f90(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(90);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f91(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(91);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f92(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(92);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f93(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(93);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f94(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(94);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f95(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(95);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f96(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(96);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f97(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(97);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f98(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(98);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f99(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(99);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f100(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(100);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f101(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(101);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f102(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(102);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f103(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(103);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f104(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(104);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f105(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(105);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f106(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(106);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f107(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(107);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f108(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(108);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f109(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(109);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f110(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(110);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f111(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(111);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f112(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(112);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f113(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(113);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f114(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(114);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f115(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(115);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f116(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(116);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f117(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(117);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f118(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(118);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f119(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(119);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f120(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(120);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f121(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(121);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f122(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(122);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f123(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(123);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f124(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(124);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f125(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(125);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f126(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(126);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f127(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(127);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f128(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(128);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f129(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(129);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f130(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(130);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f131(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(131);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f132(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(132);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f133(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(133);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f134(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(134);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f135(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(135);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f136(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(136);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f137(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(137);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f138(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(138);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f139(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(139);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f140(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(140);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f141(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(141);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f142(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(142);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f143(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(143);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f144(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(144);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f145(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(145);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f146(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(146);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f147(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(147);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f148(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(148);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f149(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(149);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f150(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(150);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f151(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(151);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f152(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(152);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f153(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(153);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f154(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(154);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f155(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(155);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f156(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(156);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f157(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(157);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f158(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(158);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f159(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(159);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f160(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(160);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f161(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(161);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f162(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(162);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f163(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(163);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f164(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(164);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f165(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(165);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f166(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(166);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f167(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(167);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f168(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(168);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f169(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(169);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f170(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(170);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f171(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(171);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f172(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(172);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f173(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(173);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f174(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(174);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f175(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(175);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f176(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(176);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f177(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(177);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f178(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(178);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f179(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(179);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f180(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(180);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f181(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(181);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f182(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(182);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f183(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(183);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f184(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(184);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f185(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(185);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f186(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(186);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f187(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(187);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f188(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(188);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f189(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(189);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f190(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(190);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f191(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(191);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f192(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(192);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f193(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(193);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f194(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(194);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f195(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(195);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f196(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(196);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f197(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(197);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f198(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(198);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f199(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(199);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f200(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(200);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f201(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(201);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f202(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(202);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f203(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(203);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f204(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(204);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f205(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(205);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f206(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(206);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f207(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(207);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f208(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(208);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f209(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(209);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f210(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(210);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f211(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(211);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f212(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(212);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f213(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(213);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f214(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(214);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f215(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(215);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f216(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(216);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f217(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(217);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f218(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(218);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f219(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(219);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f220(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(220);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f221(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(221);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f222(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(222);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f223(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(223);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f224(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(224);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f225(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(225);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f226(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(226);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f227(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(227);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f228(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(228);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f229(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(229);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f230(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(230);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f231(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(231);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f232(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(232);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f233(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(233);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f234(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(234);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f235(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(235);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f236(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(236);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f237(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(237);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f238(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(238);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f239(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(239);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f240(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(240);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f241(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(241);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f242(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(242);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f243(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(243);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f244(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(244);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f245(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(245);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f246(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(246);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f247(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(247);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f248(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(248);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f249(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(249);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f250(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(250);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f251(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(251);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f252(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(252);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f253(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(253);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f254(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(254);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f255(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(255);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f256(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(256);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f257(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(257);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f258(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(258);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f259(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(259);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f260(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(260);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f261(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(261);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f262(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(262);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => (f.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f263(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(263);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f264(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(264);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => (n.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f265(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(265);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => (u64::from(s.common().width as u32)).wrapping_mul(3),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f266(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(266);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => {
                        (h.url.len() as u64 + h.text.len() as u64).wrapping_mul(3)
                    }
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f267(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(267);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => (r.main_text.len() as u64
                        + r.ruby_text.len() as u64
                        + u64::from(r.sz_ratio))
                    .wrapping_mul(3),
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f268(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(268);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => (u64::from(p.instance_id)
                        + u64::from(p.common.width as u32)
                        + u64::from(p.reverse)
                        + u64::from(p.lock))
                    .wrapping_mul(3),
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f269(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(269);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        (e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f270(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(270);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => (u64::from(p.hide_header)
                        + u64::from(p.hide_footer)
                        + u64::from(p.hide_page_num))
                    .wrapping_mul(3),
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f271(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(271);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => {
                        (m.first_key.len() as u64 + m.second_key.len() as u64).wrapping_mul(3)
                    }
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f272(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(272);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => {
                        (u64::from(a.number) + u64::from(a.format)).wrapping_mul(3)
                    }
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f273(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(273);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => (c.chars.len() as u64).wrapping_mul(3),
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f274(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(274);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        (f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled))
                            .wrapping_mul(3)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f275(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(275);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        (u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64)
                            .wrapping_mul(3)
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f276(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(276);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => {
                        (f.command.len() as u64 + u64::from(f.field_id)).wrapping_mul(3)
                    }
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f277(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(277);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => (b.name.len() as u64).wrapping_mul(3),
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f278(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(278);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Header(h) => h.paragraphs.len() as u64,
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}

fn f279(doc: &Document) -> u64 {
    let mut n = 13u64.wrapping_mul(10009).wrapping_add(279);
    for sec in &doc.sections {
        n = n.wrapping_add(sec.paragraphs.len() as u64);
        for para in &sec.paragraphs {
            n = n.wrapping_add(para.controls.len() as u64);
            n = n.wrapping_add(para.text.chars().count() as u64);
            for ctrl in &para.controls {
                n = n.wrapping_add(match ctrl {
                    Control::Hyperlink(h) => h.url.len() as u64 + h.text.len() as u64,
                    Control::Ruby(r) => {
                        r.main_text.len() as u64 + r.ruby_text.len() as u64 + u64::from(r.sz_ratio)
                    }
                    Control::Picture(p) => {
                        u64::from(p.instance_id)
                            + u64::from(p.common.width as u32)
                            + u64::from(p.reverse)
                            + u64::from(p.lock)
                    }
                    Control::Equation(e) => {
                        e.script.len() as u64 + u64::from(e.font_size) + e.font_name.len() as u64
                    }
                    Control::PageHide(p) => {
                        u64::from(p.hide_header)
                            + u64::from(p.hide_footer)
                            + u64::from(p.hide_page_num)
                    }
                    Control::IndexMark(m) => m.first_key.len() as u64 + m.second_key.len() as u64,
                    Control::AutoNumber(a) => u64::from(a.number) + u64::from(a.format),
                    Control::CharOverlap(c) => c.chars.len() as u64,
                    Control::Form(f) => {
                        f.name.len() as u64 + f.text.len() as u64 + u64::from(f.enabled)
                    }
                    Control::Table(t) => {
                        u64::from(t.row_count) + u64::from(t.col_count) + t.cells.len() as u64
                    }
                    Control::Field(f) => f.command.len() as u64 + u64::from(f.field_id),
                    Control::Bookmark(b) => b.name.len() as u64,
                    Control::HiddenComment(h) => h.paragraphs.len() as u64,
                    Control::Header(h) => (h.paragraphs.len() as u64).wrapping_mul(3),
                    Control::Footer(f) => f.paragraphs.len() as u64,
                    Control::Footnote(n) => n.paragraphs.len() as u64,
                    Control::Endnote(n) => n.paragraphs.len() as u64,
                    Control::Shape(s) => u64::from(s.common().width as u32),
                    _ => 1,
                });
                if let Control::Table(t) = ctrl {
                    n = n.wrapping_add(t.cells.len() as u64);
                    for cell in &t.cells {
                        n = n.wrapping_add(cell.paragraphs.len() as u64);
                        for cp in &cell.paragraphs {
                            n = n.wrapping_add(cp.text.chars().count() as u64);
                            n = n.wrapping_add(cp.controls.len() as u64);
                        }
                    }
                }
            }
        }
    }
    n
}
