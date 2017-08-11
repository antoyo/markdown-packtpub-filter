extern crate pandoc;
extern crate pandoc_ast;

use std::env;
use std::path::Path;

use pandoc::OutputKind;
use pandoc_ast::{Block, Format, Inline, MutVisitor};
use pandoc_ast::Block::{Div, Para, Plain};
use pandoc_ast::Inline::{Code, Image, RawInline, Span, Str, Strong};

use self::State::*;

enum State {
    IsImage,
    IsNote,
    Other,
}

struct Visitor;

struct InlineVisitor;

impl MutVisitor for InlineVisitor {
    fn visit_inline(&mut self, inline: &mut Inline) {
        let mut new_tag = None;
        if let Code(_, ref code) = *inline {
            let open_tag = RawInline(Format("html".to_string()), "<kbd>".to_string());
            let close_tag = RawInline(Format("html".to_string()), "</kbd>".to_string());
            let inlines = vec![open_tag, Str(code.clone()), close_tag];
            new_tag = Some(Span((String::new(), vec![], vec![]), inlines));
        }
        if let Some(tag) = new_tag {
            *inline = tag;
        }
    }
}

impl MutVisitor for Visitor {
    fn visit_block(&mut self, block: &mut Block) {
        let mut state = Other;
        if let Para(ref mut inlines) = *block {
            if let Some(inline) = inlines.first() {
                match inline {
                    &Image(_, _, _) => {
                        state = IsImage;
                    },
                    &Strong(ref inlines) => {
                        if inlines.get(0) == Some(&Str("Note:".to_string())) {
                            state = IsNote;
                        }
                    },
                    _ => (),
                }
            }
        }
        match state {
            IsImage => {
                let attributes = vec!["CDPAlignCenter".to_string(), "CDPAlign".to_string(), "packt_figref".to_string()];
                let inlines =
                    match *block {
                        Para(ref inlines) => inlines.clone(),
                        _ => unreachable!(),
                    };
                *block = Div((String::new(), attributes, vec![]), vec![Plain(inlines)]);
            },
            IsNote => {
                let attributes = vec!["packt_infobox".to_string()];
                *block = Div((String::new(), attributes, vec![]), vec![block.clone()]);
            },
            Other => (),
        }
        InlineVisitor::visit_block(&mut InlineVisitor, block);
    }
}

fn main() {
    let mut args = env::args();
    args.next();
    let input = args.next().expect("input filename");
    let mut output = Path::new(&input).to_path_buf();
    output.set_extension("html");
    let output = output.to_str().expect("output filename conversion failed");

    let mut pandoc = pandoc::new();
    pandoc.add_input(&input);
    pandoc.set_output(OutputKind::File(output.to_string()));

    pandoc.add_filter(|json| pandoc_ast::filter(json, |mut pandoc| {
        let mut visitor = Visitor;
        visitor.walk_pandoc(&mut pandoc);
        pandoc
    }));
    pandoc.execute().unwrap();
}
