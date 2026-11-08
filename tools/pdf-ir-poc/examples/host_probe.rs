use rhwp::model::{control::Control, document::Document, paragraph::Paragraph};
use serde_json::{json, Value};

fn visit(p: &Paragraph, d: &Document, owner: &str, out: &mut Vec<Value>) {
    for (ci, c) in p.controls.iter().enumerate() {
        if let Control::Table(t) = c {
            let mut style = d.doc_info.para_shapes[p.para_shape_id as usize].clone();
            style.raw_data = None;
            out.push(json!({"owner":owner,"control":ci,"hostText":p.text,
                "hostCharShapes":p.char_shapes.iter().map(|r|json!({"reference":r,
                    "shape":d.doc_info.char_shapes.get(r.char_shape_id as usize)})).collect::<Vec<_>>(),
                "hostStyle":style,"hostLineSegs":p.line_segs,
                "table":{"rows":t.row_count,"columns":t.col_count,"common":t.common,
                    "caption":t.caption,"outerMargins":[t.outer_margin_left,t.outer_margin_right,t.outer_margin_top,t.outer_margin_bottom],
                    "padding":t.padding,"cells":t.cells.iter().map(|c|json!({"row":c.row,"column":c.col,
                        "rowspan":c.row_span,"colspan":c.col_span,"width":c.width,"height":c.height,"padding":c.padding,
                        "flow":c.paragraphs.iter().map(|p|{
                            let mut ps=d.doc_info.para_shapes[p.para_shape_id as usize].clone();
                            ps.raw_data=None;
                            json!({"text":p.text,"style":ps,"lineSegs":p.line_segs,
                                "inlineTabs":p.tab_extended,"charShapes":p.char_shapes,
                                "controls":p.controls.iter().map(|c|match c {
                                    Control::Table(_)=>"Table",Control::Picture(_)=>"Picture",
                                    Control::Shape(_)=>"Shape",_=>"Other"
                                }).collect::<Vec<_>>()})
                        }).collect::<Vec<_>>()
                    })).collect::<Vec<_>>()}}));
            for cell in &t.cells {
                for (pi, p) in cell.paragraphs.iter().enumerate() {
                    visit(
                        p,
                        d,
                        &format!("{owner}/c{ci}/r{}c{}/p{pi}", cell.row, cell.col),
                        out,
                    );
                }
            }
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args().nth(1).ok_or("input document required")?;
    let output = std::env::args().nth(2).ok_or("output JSON required")?;
    let doc = rhwp::parse_document(&std::fs::read(&input)?)?;
    let mut hosts = Vec::new();
    for (si, s) in doc.sections.iter().enumerate() {
        for (pi, p) in s.paragraphs.iter().enumerate() {
            visit(p, &doc, &format!("s{si}/p{pi}"), &mut hosts);
        }
    }
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&json!({"input":input,"hosts":hosts,
        "scope":"source paragraph and table metrics, not rendered-output acceptance"}))?,
    )?;
    println!("{} table hosts captured", hosts.len());
    Ok(())
}
