# GitHub Pages website plan

Owner request, 20 November 2026. **Deferred: document the work now; build and publish later.** No website or deployment workflow has been implemented. This public website is separate from the library application's OPAC and desktop interface.

## Purpose and audience

Create an attractive, credible public site for librarians evaluating Pusthakam. Explain practical library workflows, benefits, development/release status and how to try an appropriate build. Make it convincing through clear examples and useful information, without invented testimonials, adoption figures or unverified feature/platform claims. Reflect the existing Pusthakam identity. Relevant installed design and writing skills are available; no additional skill is currently needed to plan the work.

## Required pages and content

- Homepage: clear explanation of Pusthakam, librarian-focused benefits, workflow/feature overview, relevant product visuals, honest release status and a clear route to trying the app.
- Blog tab: populated with original, useful articles for librarians. Each post must have its own static, crawlable URL, for example /Pusthakam/blog/library-software-evaluation/. Include article title, excerpt, author/project identity, real publication/update dates, headings and relevant internal links. Do not create a hash-only blog or pad the sitemap with thin keyword pages.
- FAQ tab: populated with practical answers about availability, price, platforms, offline operation, library data ownership, circulation, importing records, backups, local-network access and support, reflecting actual verified functionality at publication time.
- Getting started/try page: explain who can test the current version, where to obtain available releases or source instructions, requirements, and remaining release limitations. Do not invent an installer or download link.
- Useful footer links: project repository, releases, documentation and issue reporting; accurately describe website privacy behavior if a privacy page is included.

Suggested initial editorial topics: evaluating library management software; offline library workflows; accession numbers versus ISBNs and barcodes; planning migration from an existing system; school-library circulation policies; and a practical backup/restore drill. Confirm technical facts from the current application before writing or publishing.

## Search, AI search and generative discovery

Build static/pre-rendered HTML with ordinary crawlable links and readable main content without requiring JavaScript. Generate sitemap.xml containing every canonical public page and individual article URL, with truthful modification dates. Provide robots.txt with crawl access and the correct sitemap location. Add unique page titles/descriptions, canonical URLs, social-preview metadata, useful image descriptions and appropriate structured data matching visible content (website/project, articles and breadcrumbs as applicable).

Use useful question-led answers and explanatory articles to help people searching for library software. Avoid keyword stuffing, unsupported ranking claims, fabricated reviews and special AI-discovery promises. Google states that ordinary search fundamentals also apply to AI features; no special AI markup guarantees inclusion. FAQ content helps readers; do not promise FAQ rich results. Consider AI-search crawler access according to the owner's eventual policy, without silently deciding a training-crawler policy.

GitHub project Pages normally lives under a repository path. robots.txt rules are read at the host root, so /Pusthakam/robots.txt alone does not control host crawling. When publishing, decide between a custom domain/root-level hosting or placing the appropriate robots file on the account-root Pages site. Keep canonical URLs, asset paths and sitemap locations configurable for the final host. Default proposed project URL: https://melvinvarghese.github.io/Pusthakam/ (not claimed to be configured or live).

After publication, verify the domain/site in the relevant webmaster tools, submit the sitemap and inspect representative pages. Indexing, ranking, appearance in AI answers and citations are not guaranteed. No live website/publication/search-console changes are authorized or performed in this planning step.

## Future implementation and acceptance

1. Re-audit application/release status and existing brand assets. Read relevant installed design, writing and website-development guidance before implementation.
2. Design and review desktop/mobile layouts. Keep keyboard navigation, visible focus, readable contrast, fast pages and appropriately licensed local assets/fonts.
3. Implement the static page generator/site separately from the desktop build. Ensure every article is a real URL with its own useful content and metadata; populate both Blog and FAQ before considering the website complete.
4. Check links, canonical/base paths, sitemap coverage, robots deployment location, structured-data validity and 404 handling. Test direct loading of nested article URLs and navigation without JavaScript.
5. Visually inspect desktop/mobile pages and representative articles. Check content claims against the release being advertised. Prepare a GitHub Pages build/deployment workflow when implementation starts.
6. Publish only as a separate owner-directed step after the site is reviewable; then validate the live URLs and indexing setup. Do not claim local files are already online.

No product roadmap weight or completion credit is added for this website plan. Preserve the existing desktop recovery milestone and report-design request. Resume website implementation only when the owner asks.

## Initial visual direction, for later review

The preliminary concept is a librarian's desk and classification-spine composition using Pusthakam's shelf-ink blue, white surfaces, classification colors and existing brand assets. Use clear navigation, readable long-form articles and practical product examples rather than a generic repeated-card landing page. This is a starting proposal, not an approved final design.

## References checked during planning

- GitHub Pages custom workflow guidance: https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages
- Google Search AI-feature guidance: https://developers.google.com/search/docs/appearance/ai-features
- Google robots.txt guidance: https://developers.google.com/search/docs/crawling-indexing/robots/intro
- OpenAI crawler documentation: https://developers.openai.com/api/docs/bots

Recheck official deployment/indexing guidance when implementation begins. These references inform technical planning, not promises of discoverability.
