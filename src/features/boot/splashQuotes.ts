/** Only source-reviewed entries belong in this runtime catalogue.
 * Unchecked candidates are archived in docs/splash-quote-candidates.json.
 * Source access limitations and edition notes are recorded per entry.
 */
export interface SplashQuote {
  id: string;
  text: string;
  author: string;
  work: string;
  sourceUrl: string;
  locator: string;
  edition: string;
  reviewNote: string;
  reviewedAt: string;
  rightsBasis: string;
  speaker?: string;
}

export const SPLASH_QUOTE_STORAGE_KEY = "pusthakam.last-splash-quote-id";

export const SPLASH_QUOTES: readonly SplashQuote[] = [
  {
    "id": "bacon-delight",
    "text": "Reading maketh a full man; conference a ready man; and writing an exact man.",
    "author": "Francis Bacon",
    "work": "Of Studies",
    "sourceUrl": "https://en.wikisource.org/wiki/The_Essays_of_Francis_Bacon/L_Of_Studies",
    "locator": "Essay L: Of Studies",
    "edition": "Original English essay, 1625; Gutenberg transcription.",
    "reviewNote": "Opened complete essay and checked sentence in context.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Original English work published before 1900; no modern translation or editorial commentary reproduced. Historical wording only; final edition review remains part of catalogue signoff."
  },
  {
    "id": "emerson-scholar",
    "text": "Books are the best of things, well used; abused, among the worst.",
    "author": "Ralph Waldo Emerson",
    "work": "The American Scholar",
    "sourceUrl": "https://www.gutenberg.org/files/16643/16643-h/16643-h.htm",
    "locator": "The American Scholar, paragraph beginning Books are the best",
    "edition": "Original English address, 1837; Essays, Gutenberg 16643.",
    "reviewNote": "Indexed primary-text excerpt checked; full-page request returned HTTP 503.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Original English work published before 1900; no modern translation or editorial commentary reproduced. Historical wording only; final edition review remains part of catalogue signoff."
  },
  {
    "id": "lamb-books",
    "text": "I love to lose myself in other men’s minds.",
    "author": "Charles Lamb",
    "work": "Detached Thoughts on Books and Reading",
    "sourceUrl": "https://www.gutenberg.org/cache/epub/10343/pg10343-images.html",
    "locator": "Detached Thoughts on Books and Reading, opening paragraph",
    "edition": "Original English essay, 1822; Works volume 2, edited by E. V. Lucas (1903).",
    "reviewNote": "Indexed primary-text excerpt checked; full-page request returned HTTP 503. Typographic apostrophe normalized.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Original English work published before 1900; no modern translation or editorial commentary reproduced. Historical wording only; final edition review remains part of catalogue signoff."
  },
  {
    "id": "dickinson-books",
    "text": "There is no frigate like a book To take us lands away,",
    "author": "Emily Dickinson",
    "work": "A Book",
    "sourceUrl": "https://www.gutenberg.org/cache/epub/12241/pg12241.html",
    "locator": "Third Series, XVI: A Book, opening two lines",
    "edition": "Poems, Third Series (1896), edited by Mabel Loomis Todd; early edited text, not a modern manuscript edition.",
    "reviewNote": "Indexed primary-text excerpt checked; full-page request returned HTTP 503. Line break replaced by a space; punctuation and casing retained.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Original English work published before 1900; no modern translation or editorial commentary reproduced. Historical wording only; final edition review remains part of catalogue signoff."
  },
  {
    "id": "thoreau-treasured",
    "text": "Books are the treasured wealth of the world and the fit inheritance of generations and nations.",
    "author": "Henry David Thoreau",
    "work": "Walden",
    "sourceUrl": "https://en.wikisource.org/wiki/Page:Walden,_or,_Life_in_the_Woods.djvu/116",
    "locator": "Reading, printed page 112",
    "edition": "Original English, 1854; scan-backed transcription.",
    "reviewNote": "Opened validated scan transcription and checked sentence in context.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Original English work published before 1900; no modern translation or editorial commentary reproduced. Historical wording only; final edition review remains part of catalogue signoff."
  },
  {
    "id": "shakespeare-library",
    "text": "Me, poor man, my library Was dukedom large enough.",
    "author": "William Shakespeare",
    "work": "The Tempest",
    "sourceUrl": "https://www.gutenberg.org/files/23042/23042-h/23042-h.htm",
    "locator": "Act I, scene ii, lines 109–110; Prospero",
    "edition": "Cambridge edition, 1863; Gutenberg 23042.",
    "reviewNote": "Indexed Cambridge primary-text excerpt checked; corroborated in opened Folger text, FTLN 0201–0202. Prospero speaks to Miranda; line break replaced by a space.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Original English work published before 1900; no modern translation or editorial commentary reproduced. Historical wording only; final edition review remains part of catalogue signoff.",
    "speaker": "Prospero"
  },
  {
    "id": "keller-utopia",
    "text": "In a word, literature is my Utopia.",
    "author": "Helen Keller",
    "work": "The Story of My Life",
    "sourceUrl": "https://en.wikisource.org/wiki/Page:Helen_Keller_-_The_Story_of_My_Life.pdf/155",
    "locator": "Chapter XXI, paragraph beginning In a word",
    "edition": "Original English, 1903; scan-backed transcription",
    "reviewNote": "Opened validated transcription; checked sentence in context.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  },
  {
    "id": "locke-reading",
    "text": "Reading furnishes the mind only with materials of knowledge; it is thinking makes what we read ours.",
    "author": "John Locke",
    "work": "Of the Conduct of the Understanding",
    "sourceUrl": "https://en.wikisource.org/wiki/An_Essay_Concerning_Human_Understanding_(Troutman_&_Hayes_1853)/Of_the_Conduct_of_the_Understanding",
    "locator": "Section 20: Reading",
    "edition": "Original English, 1706; Troutman & Hayes edition, 1853",
    "reviewNote": "Opened complete section; retains edition wording without adding that.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  },
  {
    "id": "jefferson-books",
    "text": "I cannot live without books",
    "author": "Thomas Jefferson",
    "work": "Letter to John Adams, 10 June 1815",
    "sourceUrl": "https://www.loc.gov/exhibits/jefferson/217.html",
    "locator": "Second sentence of displayed letter extract",
    "edition": "Original English letter, 1815; Library of Congress transcription",
    "reviewNote": "Opened archival transcription; exact opening clause ending before comma, without invented full stop.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  },
  {
    "id": "carlyle-university",
    "text": "The true University of these days is a Collection of Books.",
    "author": "Thomas Carlyle",
    "work": "On Heroes, Hero-Worship, and the Heroic in History",
    "sourceUrl": "https://www.gutenberg.org/files/1091/1091-h/1091-h.htm",
    "locator": "Lecture V, The Hero as Man of Letters; paragraph beginning It is clear",
    "edition": "Original English lectures 1840, published 1841; Gutenberg 1091",
    "reviewNote": "Indexed primary-text paragraph checked; full Gutenberg fetches unavailable (HTTP 503).",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  },
  {
    "id": "milton-living-books",
    "text": "books are not absolutely dead things, but do contain a potency of life in them",
    "author": "John Milton",
    "work": "Areopagitica",
    "sourceUrl": "https://dev.oll.libertyfund.org/titles/jebb-areopagitica-1644-jebb-ed/read",
    "locator": "Opening argument, paragraph beginning I deny not",
    "edition": "Original English, 1644; Jebb edition, Cambridge University Press, 1918",
    "reviewNote": "Opened primary text; exact contiguous clause after for and before to be; original lowercase and lack of closing punctuation retained.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  },
  {
    "id": "franklin-child-reading",
    "text": "From a Child I was fond of Reading, and all the little Money that came into my Hands was ever laid out in Books.",
    "author": "Benjamin Franklin",
    "work": "The Autobiography of Benjamin Franklin",
    "sourceUrl": "https://en.wikisource.org/wiki/The_Autobiography_of_Benjamin_Franklin/Section_Seven",
    "locator": "Section Seven, first sentence",
    "edition": "Original English, written from 1771; cited transcription retains manuscript-style capitalization",
    "reviewNote": "Opened primary-text transcription; capitalization retained. No modern editorial notes reproduced.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  },
  {
    "id": "wilde-books",
    "text": "There is no such thing as a moral or an immoral book. Books are well written, or badly written.",
    "author": "Oscar Wilde",
    "work": "The Picture of Dorian Gray",
    "sourceUrl": "https://www.gutenberg.org/cache/epub/174/pg174-images.html",
    "locator": "The Preface, paragraph beginning There is no such thing",
    "edition": "Original English novel, 1891 edition; Gutenberg 174",
    "reviewNote": "Indexed primary excerpt checked; full fetch returned HTTP 503. Two complete sentences from authorial preface, not dialogue.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  },
  {
    "id": "austen-novel",
    "text": "The person, be it gentleman or lady, who has not pleasure in a good novel, must be intolerably stupid.",
    "author": "Jane Austen",
    "work": "Northanger Abbey",
    "sourceUrl": "https://en.wikisource.org/wiki/Northanger_Abbey/Chapter_14",
    "locator": "Chapter XIV, Henry replies to Catherine on reading novels",
    "edition": "Original English novel, published 1817 with 1818 title page; primary transcription",
    "reviewNote": "Opened chapter and checked dialogue; Henry speaks, not a personal declaration by Austen.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above.",
    "speaker": "Henry Tilney"
  },
  {
    "id": "stevenson-books",
    "text": "Books are good enough in their own way, but they are a mighty bloodless substitute for life.",
    "author": "Robert Louis Stevenson",
    "work": "An Apology for Idlers",
    "sourceUrl": "https://en.wikisource.org/wiki/Virginibus_Puerisque_and_Other_Papers/An_Apology_for_Idlers",
    "locator": "Paragraph after anecdote about Johnson at Oxford",
    "edition": "Original English essay, 1877; Virginibus Puerisque and Other Papers, 1881",
    "reviewNote": "Opened essay; complete sentence. Values lived experience as well as reading, not unqualified praise of books.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  },
  {
    "id": "ruskin-books",
    "text": "For all books are divisible into two classes, the books of the hour, and the books of all time.",
    "author": "John Ruskin",
    "work": "Sesame and Lilies",
    "sourceUrl": "https://en.wikisource.org/wiki/Sesame_and_Lilies/Lecture_I",
    "locator": "Lecture I, Of Kings' Treasuries, section 8",
    "edition": "Original English lecture, 1864; published 1865; primary transcription",
    "reviewNote": "Opened lecture; complete sentence including initial For.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  },
  {
    "id": "montagu-reading",
    "text": "No entertainment is so cheap as reading, nor any pleasure so lasting.",
    "author": "Lady Mary Wortley Montagu",
    "work": "Letter to the Countess of Bute",
    "sourceUrl": "https://upload.wikimedia.org/wikipedia/commons/2/23/Selected_English_letters_%28xv-xix_centuries%29_%28IA_selectedenglishl00duckiala%29.pdf",
    "locator": "Montagu letter on educating her granddaughter; sentence beginning No entertainment",
    "edition": "Original English letter; Selected English Letters (XV-XIX Centuries), Duckitt and Wragg, 1913",
    "reviewNote": "Indexed primary-letter scan excerpt checked; full scan context and letter date require follow-up before final catalogue signoff.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  },
  {
    "id": "melville-books",
    "text": "To produce a mighty book, you must choose a mighty theme.",
    "author": "Herman Melville",
    "work": "Moby-Dick",
    "sourceUrl": "https://en.wikisource.org/wiki/Moby-Dick_(1851)_US_edition/Chapter_104",
    "locator": "Chapter CIV: The Fossil Whale, paragraph beginning Such, and so magnifying",
    "edition": "Original English, US edition, 1851; primary transcription",
    "reviewNote": "Opened chapter; exact sentence from fictional narrator Ishmael.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above.",
    "speaker": "Ishmael"
  },
  {
    "id": "dickens-books",
    "text": "there are books of which the backs and covers are by far the best parts.",
    "author": "Charles Dickens",
    "work": "Oliver Twist",
    "sourceUrl": "https://www.gutenberg.org/cache/epub/730/pg730-images.html",
    "locator": "Chapter XIV, Brownlow shows Oliver his study",
    "edition": "Original English novel, 1837-1839; Gutenberg 730",
    "reviewNote": "Indexed primary passage checked; full fetch returned HTTP 503. Exact clause following because, with original lowercase. Mr Brownlow speaks.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above.",
    "speaker": "Mr Brownlow"
  },
  {
    "id": "bacon-studies",
    "text": "Studies serve for delight, for ornament, and for ability.",
    "author": "Francis Bacon",
    "work": "Of Studies",
    "sourceUrl": "https://en.wikisource.org/wiki/The_Essays_of_Francis_Bacon/L_Of_Studies",
    "locator": "Essay L: Of Studies, first sentence",
    "edition": "Original English essay, final version 1625; primary transcription",
    "reviewNote": "Opened complete essay and checked first sentence.",
    "reviewedAt": "2026-11-23",
    "rightsBasis": "Historical original English text; no modern translation, editorial notes or website branding reproduced. Edition and review limits recorded above."
  }
];

/** Picks one quote while excluding a valid immediately previous id. */
export function selectSplashQuote(random: () => number = Math.random, previousId: string | null = null): SplashQuote {
  const choices = previousId && SPLASH_QUOTES.length > 1
    ? SPLASH_QUOTES.filter((entry) => entry.id !== previousId)
    : SPLASH_QUOTES;
  return choices[Math.min(Math.floor(random() * choices.length), choices.length - 1)]!;
}

function getStorage(): Storage | null {
  try {
    return typeof window === "undefined" ? null : window.localStorage;
  } catch {
    return null;
  }
}

/** Selects and remembers a quote without allowing storage access to break startup. */
export function selectAndRememberSplashQuote(random: () => number = Math.random): SplashQuote {
  const storage = getStorage();
  let previousId: string | null = null;
  try {
    previousId = storage?.getItem(SPLASH_QUOTE_STORAGE_KEY) ?? null;
  } catch {
    // Privacy-restricted environments may reject storage access; startup remains functional.
  }
  const selected = selectSplashQuote(random, previousId);
  try {
    storage?.setItem(SPLASH_QUOTE_STORAGE_KEY, selected.id);
  } catch {
    // The quote is still displayed even if it cannot be remembered.
  }
  return selected;
}

let launchQuote: SplashQuote | undefined;

/** A webview reload starts a new launch; React checks/remounts reuse its selection. */
export function getLaunchSplashQuote(): SplashQuote {
  launchQuote ??= selectAndRememberSplashQuote();
  return launchQuote;
}
