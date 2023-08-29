use std::hash::Hash;
use std::{io, fs};
use std::path::PathBuf;
use std::{collections::HashSet, hash::Hasher, fmt::Display};


use pulldown_cmark;
use syntect::{parsing::SyntaxSet, util::LinesWithEndings};

use crate::push_str::push;
use crate::push_str::escape_href;
use crate::push_str::escape_html;
use super::push_str::PushStr;

use once_cell::sync::Lazy;
pub(crate) struct  Markdown {
    pub(crate) title: String,
    pub (crate) body: String,
    pub(crate) summary: String,
    pub(crate) outline: String,
}

pub(crate) fn parse(source: &str) -> Markdown {
    // Enable extra specs besides default common_mark specs
    let options = pulldown_cmark::Options::empty()
        | pulldown_cmark::Options::ENABLE_TABLES
        | pulldown_cmark::Options::ENABLE_HEADING_ATTRIBUTES
        | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
        | pulldown_cmark::Options::ENABLE_SMART_PUNCTUATION;

    Renderer {
        parser: pulldown_cmark::Parser::new_ext(source, options),
        title: String::new(),
        in_title: false,
        body: String::new(),
        summary: String::new(),
        in_summary: false,
        in_table_head: false,
        used_classes: HashSet::new(),
        outline: String::new(),
        outline_level: 1,
        in_heading: false,
        syntax_set: &SYNTAX_SET,
    }
    .render()
}

struct Renderer<'a> {
    parser: pulldown_cmark::Parser<'a, 'a>,
    title: String,
    /// Whether we are currently writing to the title instead of body
    in_title: bool,
    body: String,
    summary: String,
    /// Whether we are currently writing to the summary
    in_summary: bool,
    /// Whether we are in a `<thead>`
    /// Used to determine whether to output `<td>`s or `<th>`s
    in_table_head: bool,
    /// Class names that need to be generated in the resulting CSS
    used_classes: HashSet<Classes>,
    outline: String,
    /// The level of the currently opened heading `<li> in the outline.
    /// In the range [1..6]
    outline_level: u8,
    /// Whether we are in a `hN` tag.
    /// Used to determine whether to also write to the outline
    in_heading: bool,
    syntax_set: &'a SyntaxSet,
}

impl<'a> Renderer<'a>{
    fn render(mut self) -> Markdown {
        while let Some(event) =  self.parser.next(){
            match event {
                pulldown_cmark::Event::Start(tag) => self.start_tag(tag),
                pulldown_cmark::Event::End(tag) => self.end_tag(tag),
                pulldown_cmark::Event::Text(text) => {
                    self.push_summary(&text);
                    escape_html(&mut self, &text)
                }
                pulldown_cmark::Event::Code(text) => {
                    self.push_str("<code class ='scode'>");

                    let (language, code) = 
                        match text.strip_prefix('[').and_then(|rest| rest.split_once(']')) {
                            Some((language, code)) => (Some(language), code),
                            None => (None, &*text),
                        };
                    
                    if let Some(language) = language {
                        self.syntax_highlight(language, code);
                    } else {
                        escape_html(&mut self, &text);
                    }
                    self.push_summary(code);

                    self.push_str("</code>");
                }
                pulldown_cmark::Event::Html(html) => self.push_str(&html),
                pulldown_cmark::Event::SoftBreak => {
                    self.push_summary(" ");
                    self.push_str(" ");
                }
                pulldown_cmark::Event::HardBreak => {
                    self.push_summary(" ");
                    self.push_str("<br>");
                }
                pulldown_cmark::Event::Rule => self.push_str("<hr>"),
                // We do not enable these extensions
                pulldown_cmark::Event::FootnoteReference(_)
                | pulldown_cmark::Event::TaskListMarker(_) => unreachable!(),
            }
        }

        assert!(!self.in_table_head);
        assert!(!self.in_heading);

        // Close remaining opened tags in the outline
        for _ in 0..self.outline_level - 1 {
            self.outline.push_str("</li></ul>");
        }

        if !self.used_classes.is_empty() {
            self.push_str("<style>");
            for class in &self.used_classes {
                class.write_definition(&mut self.body);
            }
            self.push_str("</style>");
        }

        Markdown { 
            title: self.title, 
            body: self.body, 
            summary: self.summary, 
            outline: self.outline, 
        }

    }

    fn start_tag(&mut self, tag: pulldown_cmark::Tag) {

        match tag {
            pulldown_cmark::Tag::Paragraph => {
                if self.summary.is_empty() {
                    self.in_summary = true;
                }
                self.push_str("<p>");
            }
            pulldown_cmark::Tag::Heading(pulldown_cmark::HeadingLevel::H1, id, classes) => {
                if !classes.is_empty() || id.is_some() {
                    self.error("title IDs and classes are disallowed");
                }
                self.in_title = true;
            }
            pulldown_cmark::Tag::Heading(level, id, classes) => {
                if !classes.is_empty() {
                    self.error("heading classes are disallowed");
                }

                let mut level = level as u8;

                // Update the outline and normalize heading levels.
                if let Some(levels_down) = self.outline_level.checked_sub(level) {
                    self.outline.push_str("</li>");
                    for _ in 0..levels_down {
                        self.outline.push_str("</ul></li>");
                    }
                } else {
                    self.outline.push_str("<ul>");

                    if level != self.outline_level + 1 {
                        let outline_level = self.outline_level;
                        self.error(format_args!(
                            "heading level jump: {outline_level} to {level}"
                        ));
                        level = self.outline_level + 1;
                    }
                }

                self.outline.push_str("<li><a href='#");
                if let Some(id) = id {
                    escape_href(&mut self.outline, id);
                }
                self.outline.push_str("'>");
                self.outline_level = level;

                if let Some(id) = id {
                    push!(self, "<h{level} id='");
                    escape_html(self, id);
                    self.push_str("'><a href='#");
                    escape_html(self, id);
                    self.push_str("' class='anchor'></a>");
                } else {
                    self.error("heading does not have id");
                    push!(self, "<h{level}>");
                }

                self.in_heading = true;
            }
            pulldown_cmark::Tag::Table(alignments) => {
                if alignments
                    .iter()
                    .all(|&align| align == pulldown_cmark::Alignment::None)
                {
                    self.push_str("<table>");
                } else {
                    let alignments = TableAlignments(alignments);
                    self.push_str("<table class='");
                    alignments.write_class_name(self);
                    self.push_str("'>");
                    self.used_classes.insert(Classes::Table(alignments));
                }
            }
            pulldown_cmark::Tag::TableHead => {
                self.push_str("<thead><tr>");
                self.in_table_head = true;
            }
            pulldown_cmark::Tag::TableRow => self.push_str("<tr>"),
            pulldown_cmark::Tag::TableCell => {
                self.push_str(match self.in_table_head {
                    true => "<th>",
                    false => "<td>",
                });
            }
            pulldown_cmark::Tag::BlockQuote => self.push_str("<blockquote>"),
            pulldown_cmark::Tag::CodeBlock(kind) => {
                self.push_str("<pre class='scode'><code>");

                let language = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) if lang.is_empty() => None,
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => Some(lang),
                    pulldown_cmark::CodeBlockKind::Indented => None,
                };

                fn event_text(
                    event: pulldown_cmark::Event<'_>,
                ) -> Option<pulldown_cmark::CowStr<'_>> {
                    match event {
                        pulldown_cmark::Event::End(_) => None,
                        pulldown_cmark::Event::Text(text) => Some(text),
                        // Other events shouldn't happen
                        _ => unreachable!("unexpected event in code block {:?}", event),
                    }
                }

                if let Some(language) = language {
                    let mut code = String::new();
                    while let Some(part) = self.parser.next().and_then(event_text) {
                        code.push_str(&part);
                    }
                    self.syntax_highlight(&language, &code);
                } else {
                    while let Some(part) = self.parser.next().and_then(event_text) {
                        escape_html(self, &part);
                    }
                }

                self.push_str("</code></pre>");
            }
            pulldown_cmark::Tag::List(Some(1)) => self.push_str("<ol>"),
            pulldown_cmark::Tag::List(Some(start)) => {
                push!(self, "<ol start='{}'>", start);
            }
            pulldown_cmark::Tag::List(None) => self.push_str("<ul>"),
            pulldown_cmark::Tag::Item => self.push_str("<li>"),
            pulldown_cmark::Tag::Emphasis => self.push_str("<em>"),
            pulldown_cmark::Tag::Strong => self.push_str("<strong>"),
            pulldown_cmark::Tag::Strikethrough => self.push_str("<del>"),
            pulldown_cmark::Tag::Link(pulldown_cmark::LinkType::Email, ..) => {
                self.error("email links are not supported yet");
            }
            pulldown_cmark::Tag::Link(_type, href, title) => {
                self.push_str("<a href='");
                escape_href(self, &href);
                if !title.is_empty() {
                    self.push_str("' title='");
                    escape_html(self, &title);
                }
                self.push_str("'>");
            }
            pulldown_cmark::Tag::Image(_, url, title) => {
                self.push_str("<img src='");
                escape_href(self, &url);
                self.push_str("' alt='");
                while let Some(event) = self.parser.next() {
                    match event {
                        pulldown_cmark::Event::End(_) => break,
                        pulldown_cmark::Event::Text(text) => escape_html(self, &text),
                        _ => unreachable!(),
                    }
                }
                if !title.is_empty() {
                    self.push_str("' title='");
                    escape_html(self, &title);
                }
                self.push_str("'>");
            }
            // We do not enable this extension
            pulldown_cmark::Tag::FootnoteDefinition(_) => unreachable!(),
        }

    }

     fn end_tag(&mut self, tag: pulldown_cmark::Tag<'a>) {
        match tag {
            pulldown_cmark::Tag::Paragraph => {
                self.push_str("</p>");
                self.in_summary = false;
            }
            pulldown_cmark::Tag::Heading(pulldown_cmark::HeadingLevel::H1, _id, _classes) => {
                self.in_title = false;
            }
            pulldown_cmark::Tag::Heading(level, _id, _classes) => {
                self.in_heading = false;

                self.outline.push_str("</a>");

                self.push_str("</");
                push!(self, "{}", level);
                self.push_str(">");
            }
            pulldown_cmark::Tag::Table(_) => {
                self.push_str("</tbody></table>");
            }
            pulldown_cmark::Tag::TableHead => {
                self.push_str("</tr></thead><tbody>");
                self.in_table_head = false;
            }
            pulldown_cmark::Tag::TableRow => {
                self.push_str("</tr>");
            }
            pulldown_cmark::Tag::TableCell => {
                self.push_str(match self.in_table_head {
                    true => "</th>",
                    false => "</td>",
                });
            }
            pulldown_cmark::Tag::BlockQuote => self.push_str("</blockquote>"),
            pulldown_cmark::Tag::List(Some(_)) => self.push_str("</ol>"),
            pulldown_cmark::Tag::List(None) => self.push_str("</ul>"),
            pulldown_cmark::Tag::Item => self.push_str("</li>"),
            pulldown_cmark::Tag::Emphasis => self.push_str("</em>"),
            pulldown_cmark::Tag::Strong => self.push_str("</strong>"),
            pulldown_cmark::Tag::Strikethrough => self.push_str("</del>"),
            pulldown_cmark::Tag::Link(_, _, _) => self.push_str("</a>"),
            // We do not enable this extension
            pulldown_cmark::Tag::FootnoteDefinition(_)
            // We handle closing of these tags in the opening logic
            | pulldown_cmark::Tag::Image(_, _, _)
                | pulldown_cmark::Tag::CodeBlock(_)
                => unreachable!(),
        }
    }

    fn syntax_highlight(&mut self, language: &str, code: &str) {
        let Some(syntax) = self.syntax_set.find_syntax_by_token(language) else {
            self.error(format_args!("no known language {language}"));
            self.push_str(code);
            return;
        };

        let mut generator = syntect::html::ClassedHTMLGenerator::new_with_class_style(
            syntax,
            self.syntax_set,
            SYNTECT_CLASS_STYLE,
        );

        for line in LinesWithEndings::from(code) {
            generator.parse_html_for_line_which_includes_newline(line)
                .expect("thanks syntect, really good API design where you return a `Result` but don’t specify when it can even fail");
        }

        self.push_str(&generator.finalize());
    }

    fn error(&mut self, msg: impl Display) {
        self.push_str("<span style='color:red'>");
        push!(self, "{}", msg);
        self.push_str("</span>");
    }

    fn push_summary(&mut self, s: &str) {
        if self.in_summary {
            self.summary.push_str(s);
        }
    }

}

impl PushStr for Renderer<'_> {
    fn push_str(&mut self, s: &str) {
        if self.in_title {
            self.title.push_str(s);
        } else {
            self.body.push_str(s);
            if self.in_heading {
                self.outline.push_str(s);
            }
        }
    }
}

struct TableAlignments(Vec<pulldown_cmark::Alignment>);

impl TableAlignments {
    fn write_class_name(&self, buf: &mut impl PushStr) {
        buf.push_str("t");
        for alignment in &self.0 {
            buf.push_str(match alignment {
                pulldown_cmark::Alignment::None => "n",
                pulldown_cmark::Alignment::Left => "l",
                pulldown_cmark::Alignment::Center => "c",
                pulldown_cmark::Alignment::Right => "r",
            });
        }
    }
}

impl PartialEq for TableAlignments {
    fn eq(&self, other: &TableAlignments) -> bool {
        Iterator::eq(
            self.0.iter().map(|&alignment| alignment as u8),
            other.0.iter().map(|&alignment| alignment as u8),
        )
    }
}

impl Eq for TableAlignments {}

// pulldown_cmark::Alignment isn't Hash
impl Hash for TableAlignments {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for &alignment in &self.0 {
            state.write_u8(alignment as u8);
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
enum Classes {
    Table(TableAlignments),
}

impl Classes {
    fn write_definition(&self, buf: &mut impl PushStr) {
        match self {
            Self::Table(alignments) => {
                for (i, alignment) in alignments.0.iter().copied().enumerate() {
                    if alignment == pulldown_cmark::Alignment::None {
                        continue;
                    }
                    buf.push_str(".");
                    alignments.write_class_name(buf);
                    push!(buf, " td:nth-child({})", i + 1);
                    buf.push_str("{text-align:");
                    buf.push_str(match alignment {
                        pulldown_cmark::Alignment::None => unreachable!(),
                        pulldown_cmark::Alignment::Left => "left",
                        pulldown_cmark::Alignment::Center => "center",
                        pulldown_cmark::Alignment::Right => "right",
                    });
                    buf.push_str("}");
                }
            }
        }
    }
}

const SYNTECT_CLASS_STYLE: syntect::html::ClassStyle =
    syntect::html::ClassStyle::SpacedPrefixed { prefix: "s" };

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);


pub(crate) fn generate_html(posts: &[(PathBuf, String)]) -> Result<(), io::Error> {

    let output_dir = PathBuf::from("blog");

    match fs::create_dir_all(&output_dir) {
        Ok(dir) => dir,
        Err(error) => return  Err(error),
    }

    let mut handlebars = handlebars::Handlebars::new();
        handlebars.register_template_file("template", "templates/posts.hbs")
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    for (path, content) in posts {
       let markdown = parse(&content);

       let markdown_data = serde_json::json!({
        "title": markdown.title,
        "content": markdown.body,
        "summary": markdown.summary,
        "outline": markdown.outline
       });

       let rendered_html = handlebars.render("template", &markdown_data)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        let file_name = path.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        
        let output_file = output_dir.join(format!("{}.html", file_name));
        fs::write(&output_file, rendered_html)?;
    }

    Ok(())
}









































































// use std::{path::PathBuf, fs};
// use handlebars::Handlebars;
// use pulldown_cmark::{ Parser, Options, html, Event, Tag, CodeBlockKind, CowStr };
// use serde_yaml;
// use serde_json::{self, json};
// use std::io;
// // use syntect::html::highlighted_html_for_string;
// // use syntect::parsing::SyntaxSet;
// // use syntect::highlighting::ThemeSet;

// fn md_to_html(markdown: &str) -> String {
//     // let syntax_set  = SyntaxSet::load_defaults_newlines();
//     // let theme_set = ThemeSet::load_defaults();

//     let parser = Parser::new_ext(&markdown, Options::empty());

   
    
//     let mut html_output = String::new();
//     html::push_html(&mut html_output, parser);

//     html_output
// }





// pub fn generate_html(posts: &[(PathBuf, String)]) -> Result<(), io::Error> {
//     let mut handlebars = Handlebars::new();
//         handlebars.register_template_file("blog_template", "templates/posts.hbs")
//         .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

//     let output_dir = PathBuf::from("blog");
//     match fs::create_dir_all(&output_dir) {
//         Ok(dir) => dir,
//         Err(err) => return Err(err),
//     }
    
//     for (path, contents) in posts {
//         let split_front_matter: Vec<&str> = contents.splitn(3, "---").collect();
//         let yaml_front_matter = split_front_matter.get(1).unwrap_or(&"");
//         let markdown = split_front_matter.get(2).unwrap_or(&"");

//         let yaml_data: serde_yaml::Value = serde_yaml::from_str(yaml_front_matter).unwrap();

//         let post_title = yaml_data["title"].as_str();
//         let post_description = yaml_data["description"].as_str();
//         let content = md_to_html(markdown);
//         let post_date = yaml_data["date"].as_str();

//         let rendered_html = match handlebars.render("blog_template", &json!({
//             "post_title": post_title,
//             "post_description": post_description,
//             "post_content": content,
//             "post_date": post_date,
//         })) {
//             Ok(html) => html,
//             Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),    
//         }; 

//         let file_name = path.file_stem()
//         .and_then(|stem| stem.to_str())
//         .unwrap_or("");
//         let output_file = output_dir.join(format!("{}.html", file_name));
//         fs::write(&output_file, rendered_html)?;
//     }

//     Ok(())
// }