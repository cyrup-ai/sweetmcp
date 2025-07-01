# fetch

Retrieve and transform web content from a specified URL through a reliable multi-stage process:

1. First attempt: Use hyper/http libraries to fetch the content, render it with bevy engine
2. Fallback: If first attempt fails, leverage chromiumoxide for headless browser capabilities
3. Final contingency: Utilize firecrawl if previous methods are unsuccessful

For the retrieved content:

- Remove all script and style elements to extract core content
- Convert cleaned HTML to well-formatted markdown
- Preserve essential text formatting, links, and structural elements
- Return output in clean, readable markdown format suitable for documentation or analysis

## Options

- screenshot_format: one of base64, sixtel
- content_format: one of (markdown, json, txt)
- syntax_highlighting: boolean
- theme: themes from XX

## Returns 

- screenshot (base64 or sixtel)
- content (in requested formatting with or without highlighting)
- content-type (mirrors requested)