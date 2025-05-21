# `mdwiki`

`mdwiki` is a tool to generate a static wiki site from a directory structure
containing markdown documents and images.

## Syntax

`mdwiki` parses a markdown flavour inspired by Github markdown. Features
outside standard markdown that are supported are

* Github-style images: `![Alt-text](/url/for/image.png)`
* Automatic links. By leaving the URL field of a link blank, you can direct
    `mdwiki` to attempt to link to the resource indicated by the link text.
    This works by converting the link text to `kebab-case` and looking for a
    file with an appropriate name. The nearest relative in the directory
    struture of the provided page will be used if multiple matches are found. A
    warning will be emitted on failure. Examples:
    * `[My page]()` in a project with `/my-page.md` will be rendered as
        `<a href="/my-page.html">My page</a>`. Regular links will only go to
        markdown (rendered to HTML) pages.
    * `![World map]()` in a project with `/images/world-map.jpg` will be
        rendered as `<img src="/images/world-map.jpg" alt="World map">`. Image
        links will go to `png` or `jpe?g` images.
* Tagging. Tags of the form `#tag1 #tag2 #tag3` may be included to modify the
    following element. Note that these tags are differentiated from headings
    by requiring an alphabetic character immediately after the `#`. Thus a
    space is required for headings. Currently the following tags have an
    effect:
    * `#dm` causes the element to be encrypted, requiring a password to
        decrypt.
