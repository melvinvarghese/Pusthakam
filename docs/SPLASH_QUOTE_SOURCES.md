# Splash quotation sources

## Current collection (23 November 2026)

Twenty short entries are enabled. The requested approximately 100 authentic quotations remain a work in progress. The previous 113-entry set is preserved in `splash-quote-candidates.json` solely as an unreviewed research archive. It includes unreliable attributions, paraphrases, overlapping fragments and lines unrelated to reading. It must never be imported into the application.

The authoritative text and metadata for enabled entries are in `src/features/boot/splashQuotes.ts`. Each entry carries its exact wording, credited author, original work, source URL, locator, edition, review date, review method and reuse basis. Fictional speakers are identified separately. Tests check record completeness, not literary authenticity.

| Entry ID | Source and locator | Review on 23 November |
| --- | --- | --- |
| `bacon-delight` | [Bacon, Of Studies](https://www.gutenberg.org/cache/epub/575/pg575-images.html), essay L; original English, 1625 | Opened complete primary-text transcription and checked wording in context; updated source URL in runtime metadata. |
| `emerson-scholar` | [Emerson, The American Scholar](https://www.gutenberg.org/files/16643/16643-h/16643-h.htm), paragraph beginning Books are the best; address, 1837 | Indexed primary-text excerpt checked. Full-page retrieval returned HTTP 503. |
| `lamb-books` | [Lamb, Detached Thoughts on Books and Reading](https://www.gutenberg.org/cache/epub/10343/pg10343-images.html), opening paragraph; essay 1822, E. V. Lucas edition 1903 | Indexed primary-text excerpt checked. Full-page retrieval returned HTTP 503. Typographic apostrophe normalized. |
| `dickinson-books` | [Dickinson, Poems, Third Series](https://www.gutenberg.org/cache/epub/12241/pg12241.html), XVI, A Book, first two lines; Mabel Loomis Todd edition, 1896 | Indexed primary-text excerpt checked. Full-page retrieval returned HTTP 503. Early edited wording retained, line break replaced by a space; no claim to reproduce the manuscript version. |
| `thoreau-treasured` | [Walden, Reading, printed page 112](https://en.wikisource.org/wiki/Page:Walden,_or,_Life_in_the_Woods.djvu/116), 1854 | Opened validated scan-backed transcription; sentence checked in context. |
| `shakespeare-library` | [The Tempest, Cambridge edition 1863](https://www.gutenberg.org/files/23042/23042-h/23042-h.htm), I.ii.109-110 | Indexed primary-text excerpt checked; corroborated in the opened [Folger text](https://www.folger.edu/explore/shakespeares-works/the-tempest/read/), FTLN 0201-0202. Prospero speaks to Miranda. Line break replaced by a space. |

These are original English texts from historical editions, not modern translations. Only the short quoted text is bundled, without website headers, editorial commentary or Gutenberg branding. Historical text dates and author/edition details inform reuse review; this is not a claim of worldwide legal clearance. Full Gutenberg page availability needs a follow-up check before final catalogue signoff.

## Corrections and exclusions

- The previous `coleridge-read` entry reproduces wording found in Bacon's Of Studies. It must not be re-enabled with the Coleridge attribution.
- The previous Dickinson entry was a paraphrase; replaced with the first two lines from the identified early edition.
- Austen's popular reading line in Pride and Prejudice is spoken by Caroline Bingley while trying to impress Darcy, followed by abandoning her book. It was researched but excluded from this first batch to avoid presenting the irony as Austen's personal endorsement.
- Entries labeled Attributed saying, vague Letters references, recent translations and generic motivational lines remain excluded pending individual review.

## Next audit steps

Expand the enabled collection towards approximately 100 distinct, relevant, short entries. Check original wording and context; identify editions/translators and fictional speakers. Add primary locators and honest review methods. Remove unsupported candidates instead of silently editing or paraphrasing them into quotations. Keep each enabled entry independently traceable and favor brief wording readable during the existing three-second splash.

The picker runs entirely locally. Source URLs are documentation metadata, never runtime requests. Unchecked candidates are outside the source import tree and excluded from production bundles.

## Expanded batch - 23 November 2026

Fourteen entries added; 20 enabled. Exact text lives in runtime records to avoid divergent copies. Full-text review and indexed-excerpt review are distinguished. Montagu full-scan context/date check and final edition review remain open.

| ID | Primary source | Locator and review |
| --- | --- | --- |
| `keller-utopia` | [Helen Keller: The Story of My Life](https://en.wikisource.org/wiki/Page:Helen_Keller_-_The_Story_of_My_Life.pdf/155) | Chapter XXI, paragraph beginning In a word. Opened validated transcription; checked sentence in context. |
| `locke-reading` | [John Locke: Of the Conduct of the Understanding](https://en.wikisource.org/wiki/An_Essay_Concerning_Human_Understanding_(Troutman_&_Hayes_1853)/Of_the_Conduct_of_the_Understanding) | Section 20: Reading. Opened complete section; retains edition wording without adding that. |
| `jefferson-books` | [Thomas Jefferson: Letter to John Adams, 10 June 1815](https://www.loc.gov/exhibits/jefferson/217.html) | Second sentence of displayed letter extract. Opened archival transcription; exact opening clause ending before comma, without invented full stop. |
| `carlyle-university` | [Thomas Carlyle: On Heroes, Hero-Worship, and the Heroic in History](https://www.gutenberg.org/files/1091/1091-h/1091-h.htm) | Lecture V, The Hero as Man of Letters; paragraph beginning It is clear. Indexed primary-text paragraph checked; full Gutenberg fetches unavailable (HTTP 503). |
| `milton-living-books` | [John Milton: Areopagitica](https://dev.oll.libertyfund.org/titles/jebb-areopagitica-1644-jebb-ed/read) | Opening argument, paragraph beginning I deny not. Opened primary text; exact contiguous clause after for and before to be; original lowercase and lack of closing punctuation retained. |
| `franklin-child-reading` | [Benjamin Franklin: The Autobiography of Benjamin Franklin](https://en.wikisource.org/wiki/The_Autobiography_of_Benjamin_Franklin/Section_Seven) | Section Seven, first sentence. Opened primary-text transcription; capitalization retained. No modern editorial notes reproduced. |
| `wilde-books` | [Oscar Wilde: The Picture of Dorian Gray](https://www.gutenberg.org/cache/epub/174/pg174-images.html) | The Preface, paragraph beginning There is no such thing. Indexed primary excerpt checked; full fetch returned HTTP 503. Two complete sentences from authorial preface, not dialogue. |
| `austen-novel` | [Jane Austen: Northanger Abbey](https://en.wikisource.org/wiki/Northanger_Abbey/Chapter_14) | Chapter XIV, Henry replies to Catherine on reading novels. Opened chapter and checked dialogue; Henry speaks, not a personal declaration by Austen. |
| `stevenson-books` | [Robert Louis Stevenson: An Apology for Idlers](https://en.wikisource.org/wiki/Virginibus_Puerisque_and_Other_Papers/An_Apology_for_Idlers) | Paragraph after anecdote about Johnson at Oxford. Opened essay; complete sentence. Values lived experience as well as reading, not unqualified praise of books. |
| `ruskin-books` | [John Ruskin: Sesame and Lilies](https://en.wikisource.org/wiki/Sesame_and_Lilies/Lecture_I) | Lecture I, Of Kings' Treasuries, section 8. Opened lecture; complete sentence including initial For. |
| `montagu-reading` | [Lady Mary Wortley Montagu: Letter to the Countess of Bute](https://upload.wikimedia.org/wikipedia/commons/2/23/Selected_English_letters_%28xv-xix_centuries%29_%28IA_selectedenglishl00duckiala%29.pdf) | Montagu letter on educating her granddaughter; sentence beginning No entertainment. Indexed primary-letter scan excerpt checked; full scan context and letter date require follow-up before final catalogue signoff. |
| `melville-books` | [Herman Melville: Moby-Dick](https://en.wikisource.org/wiki/Moby-Dick_(1851)_US_edition/Chapter_104) | Chapter CIV: The Fossil Whale, paragraph beginning Such, and so magnifying. Opened chapter; exact sentence from fictional narrator Ishmael. |
| `dickens-books` | [Charles Dickens: Oliver Twist](https://www.gutenberg.org/cache/epub/730/pg730-images.html) | Chapter XIV, Brownlow shows Oliver his study. Indexed primary passage checked; full fetch returned HTTP 503. Exact clause following because, with original lowercase. Mr Brownlow speaks. |
| `bacon-studies` | [Francis Bacon: Of Studies](https://en.wikisource.org/wiki/The_Essays_of_Francis_Bacon/L_Of_Studies) | Essay L: Of Studies, first sentence. Opened complete essay and checked first sentence. |

The selected quote is cached for the webview lifetime, so React StrictMode initializer checks and component remounts cannot remember a different entry than the displayed one. Reloading the app/webview starts the next selection; tray hide/restore preserves the launch.
