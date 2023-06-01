use crate::{parse::parse_document, render::render_document};

const MD: &str = r#"
# Test Markdown File

This is a test markdown file. It should

* Parse lists
    * Including sub lists
    * And **bold** and *italics*
    * And [links](https://owen.feik.xyz) and ![images](https://example.org/example.jpg)
"#;

const HTML: &str = r#"
<html>
 <head>
 </head>
 <body>
  <main>
   <h1>Test Markdown File</h1>
   This is a test markdown file. It should
   <ul>
    <li>Parse lists
     <ul>
      <li>Including sub lists</li>
      <li>And <b>bold</b> and <i>italics</i></li>
      <li>And <a href="https://owen.feik.xyz">links</a> and <img src="https://example.org/example.jpg" alt="images"></li>
     </ul>
    </li>
   </ul>
  </main>
 </body>
</html>
"#;

#[test]
pub fn test_integration() {
    assert_eq!(render_document(&parse_document(MD.trim())), HTML.trim());
}
