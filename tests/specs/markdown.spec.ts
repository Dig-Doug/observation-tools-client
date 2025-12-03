import {expect, test} from "../fixtures";
import { ObservationBuilder } from "observation-tools-client";
import {TestId} from "../helpers/testIds";

test("Markdown observation is rendered as HTML", async ({ page, server }) => {
    const client = server.createClient();
    const executionName = "execution-with-markdown";
    const exe = client.beginExecution(executionName);

    const observationName = "markdown-observation";
    
    const markdownContent = `
# h1 Heading 8-)
## h2 Heading
### h3 Heading
#### h4 Heading
##### h5 Heading
###### h6 Heading


## Horizontal Rules

___

---

***


## Typographic replacements

Enable typographer option to see result.

(c) (C) (r) (R) (tm) (TM) (p) (P) +-

test.. test... test..... test?..... test!....

!!!!!! ???? ,,  -- ---

"Smartypants, double quotes" and 'single quotes'


## Emphasis

**This is bold text**

__This is bold text__

*This is italic text*

_This is italic text_

~~Strikethrough~~


## Blockquotes


> Blockquotes can also be nested...
>> ...by using additional greater-than signs right next to each other...
> > > ...or with spaces between arrows.


## Lists

Unordered

+ Create a list by starting a line with \`+\`, \`-\`, or \`*\`
+ Sub-lists are made by indenting 2 spaces:
  - Marker character change forces new list start:
    * Ac tristique libero volutpat at
    + Facilisis in pretium nisl aliquet
    - Nulla volutpat aliquam velit
+ Very easy!

Ordered

1. Lorem ipsum dolor sit amet
2. Consectetur adipiscing elit
3. Integer molestie lorem at massa


1. You can use sequential numbers...
1. ...or keep all the numbers as \`1.\`

Start numbering with offset:

57. foo
1. bar


## Code

Inline \`code\`

Indented code

    // Some comments
    line 1 of code
    line 2 of code
    line 3 of code


Block code "fences"

\`\`\`
Sample text here...
\`\`\`

Syntax highlighting

\`\`\` js
var foo = function (bar) {
  return bar++;
};

console.log(foo(5));
\`\`\`

## Tables

| Option | Description |
| ------ | ----------- |
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |

Right aligned columns

| Option | Description |
| ------:| -----------:|
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |


## Links

[link text](http://dev.nodeca.com)

[link with title](http://nodeca.github.io/pica/demo/ "title text!")

Autoconverted link https://github.com/nodeca/pica (enable linkify to see)


## Images

![Minion](https://octodex.github.com/images/minion.png)
![Stormtroopocat](https://octodex.github.com/images/stormtroopocat.jpg "The Stormtroopocat")

Like links, Images also have a footnote style syntax

![Alt text][id]

With a reference later in the document defining the URL location:

[id]: https://octodex.github.com/images/dojocat.jpg  "The Dojocat"


## Plugins

The killer feature of \`markdown-it\` is very effective support of
[syntax plugins](https://www.npmjs.org/browse/keyword/markdown-it-plugin).


### [Emojies](https://github.com/markdown-it/markdown-it-emoji)

> Classic markup: :wink: :cry: :laughing: :yum:
>
> Shortcuts (emoticons): :-) :-( 8-) ;)

see [how to change output](https://github.com/markdown-it/markdown-it-emoji#change-output) with twemoji.


### [Subscript](https://github.com/markdown-it/markdown-it-sub) / [Superscript](https://github.com/markdown-it/markdown-it-sup)

- 19^th^
- H~2~O


### [\\<ins>](https://github.com/markdown-it/markdown-it-ins)

++Inserted text++


### [\\<mark>](https://github.com/markdown-it/markdown-it-mark)

==Marked text==


### [Footnotes](https://github.com/markdown-it/markdown-it-footnote)

Footnote 1 link[^first].

Footnote 2 link[^second].

Inline footnote^[Text of inline footnote] definition.

Duplicated footnote reference[^second].

[^first]: Footnote **can have markup**

    and multiple paragraphs.

[^second]: Footnote text.


### [Definition lists](https://github.com/markdown-it/markdown-it-deflist)

Term 1

:   Definition 1
with lazy continuation.

Term 2 with *inline markup*

:   Definition 2

        { some code, part of Definition 2 }

    Third paragraph of definition 2.

_Compact style:_

Term 1
  ~ Definition 1

Term 2
  ~ Definition 2a
  ~ Definition 2b


### [Abbreviations](https://github.com/markdown-it/markdown-it-abbr)

This is HTML abbreviation example.

It converts "HTML", but keep intact partial entries like "xxxHTMLyyy" and so on.

*[HTML]: Hyper Text Markup Language

### [Custom containers](https://github.com/markdown-it/markdown-it-container)

::: warning
*here be dragons*
:::
`;

    // Create observation with markdown payload using ObservationBuilder
    new ObservationBuilder(observationName)
        .markdownPayload(markdownContent)
        .send(exe);

    // Navigate to observation
    await page.goto(server.baseUrl);
    await page.getByTestId(TestId.NavExecutionsList).click();
    await page.getByTestId(TestId.ExecutionLink).filter({ hasText: executionName }).first().click();
    await page.getByTestId(TestId.ObservationListItemLink).filter({ hasText: observationName }).click();

    // Verify markdown is rendered as HTML
    const payloadElement = page.getByTestId(TestId.ObservationPayload);
    await expect(payloadElement).toBeVisible();

    // Check that markdown headers are rendered as HTML h1 elements
    await expect(payloadElement.locator("h1")).toContainText("Hello World");

    // Check that bold text is rendered
    await expect(payloadElement.locator("strong")).toContainText("bold");

    // Check that italic text is rendered
    await expect(payloadElement.locator("em")).toContainText("italic");

    // Check that list items are rendered
    await expect(payloadElement.locator("li").first()).toContainText("Item 1");
});

test("Markdown observation sanitizes malicious HTML", async ({ page, server }) => {
    const client = server.createClient();
    const executionName = "execution-with-malicious-markdown";
    const exe = client.beginExecution(executionName);

    const observationName = "malicious-markdown-observation";
    // Try to inject malicious HTML/JavaScript
    const maliciousMarkdown = `# Safe Header

<script>alert('XSS')</script>

<img src="x" onerror="alert('XSS')">

[Click me](javascript:alert('XSS'))

Safe paragraph.
`;

    // Create observation with markdown payload using ObservationBuilder
    new ObservationBuilder(observationName)
        .markdownPayload(maliciousMarkdown)
        .send(exe);

    // Navigate to observation
    await page.goto(server.baseUrl);
    await page.getByTestId(TestId.NavExecutionsList).click();
    await page.getByTestId(TestId.ExecutionLink).filter({ hasText: executionName }).first().click();
    await page.getByTestId(TestId.ObservationListItemLink).filter({ hasText: observationName }).click();

    const payloadElement = page.getByTestId(TestId.ObservationPayload);
    await expect(payloadElement).toBeVisible();

    // Verify safe content is rendered
    await expect(payloadElement.locator("h1")).toContainText("Safe Header");
    await expect(payloadElement).toContainText("Safe paragraph");

    // Verify malicious script tags are removed
    await expect(payloadElement.locator("script")).toHaveCount(0);

    // Verify javascript: links are sanitized (ammonia removes the href entirely or sanitizes it)
    const links = payloadElement.locator("a");
    const linkCount = await links.count();
    for (let i = 0; i < linkCount; i++) {
        const href = await links.nth(i).getAttribute("href");
        // href should either be null (removed) or not contain javascript:
        if (href !== null) {
            expect(href).not.toContain("javascript:");
        }
    }
});
