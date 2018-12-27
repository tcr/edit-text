use failure::Error;
use oatie::doc::*;
use oatie::writer::CurWriter;
use oatie::rtf::*;

pub struct RandomCursorContext {
    cur: CurWriter,
    history: Vec<CurSpan>,
}

impl Default for RandomCursorContext {
    fn default() -> Self {
        RandomCursorContext {
            cur: CurWriter::new(),
            history: vec![],
        }
    }
}

pub fn random_cursor_span(ctx: &mut RandomCursorContext, span: &DocSpan<RtfSchema>) -> Result<(), Error> {
    for elem in span {
        match *elem {
            DocGroup(_, ref span) => {
                {
                    let mut c = ctx.cur.clone();
                    c.place(&CurElement::CurGroup);
                    c.exit_all();
                    ctx.history.push(c.result());
                }

                ctx.cur.begin();
                random_cursor_span(ctx, span)?;
                ctx.cur.exit();
            }
            DocChars(ref text, _) => {
                ensure!(text.char_len() > 0, "Empty char string");

                for _ in 0..text.char_len() {
                    // Push a cursor to this character.
                    let mut c = ctx.cur.clone();
                    c.place(&CurElement::CurChar);
                    c.exit_all();
                    ctx.history.push(c.result());

                    // But also increment the base cursor to skip this char.
                    ctx.cur.place(&CurElement::CurSkip(1));
                }
            }
        }
    }
    Ok(())
}

pub fn random_cursor(doc: &Doc<RtfSchema>) -> Result<Vec<CurSpan>, Error> {
    let mut ctx = RandomCursorContext::default();
    random_cursor_span(&mut ctx, &doc.0)?;
    Ok(ctx.history)
}
