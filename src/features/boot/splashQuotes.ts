/**
 * Locally bundled reading-quotation catalogue. This file deliberately makes no
 * network call. Source verification status is recorded in docs/SPLASH_QUOTE_SOURCES.md.
 */
export interface SplashQuote {
  id: string;
  text: string;
  author: string;
  work: string;
  sourceUrl: string;
}

export const SPLASH_QUOTE_STORAGE_KEY = "pusthakam.last-splash-quote-id";

const GUTENBERG_BOOK_LOVER = "https://www.gutenberg.org/ebooks/47013";
const GUTENBERG_SCHOPENHAUER = "https://www.gutenberg.org/ebooks/11945";
const GUTENBERG_RYECROFT = "https://www.gutenberg.org/ebooks/1463";
const GUTENBERG_READING = "https://www.gutenberg.org/ebooks/16579";

const quote = (id: string, text: string, author: string, work: string, sourceUrl = GUTENBERG_BOOK_LOVER): SplashQuote => ({
  id,
  text,
  author,
  work,
  sourceUrl,
});

/** Exactly 113 short quotation candidates suitable for the splash screen. */
export const SPLASH_QUOTES: readonly SplashQuote[] = [
  quote("de-bury-lover-of-books", "Whosoever acknowledges himself to be a zealous follower of truth, of happiness, of wisdom, of science, or even of the faith, must of necessity make himself a lover of books.", "Richard de Bury", "Philobiblon"),
  quote("bacon-tasted-swallowed", "Some books are to be tasted, others to be swallowed, and some few to be chewed and digested.", "Francis Bacon", "Of Studies"),
  quote("bacon-delight", "Reading maketh a full man; conference a ready man; and writing an exact man.", "Francis Bacon", "Of Studies"),
  quote("cicero-room-without-books", "A room without books is like a body without a soul.", "Cicero", "Attributed saying"),
  quote("pliny-no-book-bad", "No book is so bad that some good may not be got out of it.", "Pliny the Elder", "Natural History"),
  quote("seneca-leisure", "Leisure without books is death, and burial of a man alive.", "Seneca", "Moral Epistles"),
  quote("erasmus-when-money", "When I get a little money I buy books; and if any is left I buy food and clothes.", "Erasmus", "Apophthegms"),
  quote("shakespeare-trees-books", "Finds tongues in trees, books in the running brooks, sermons in stones, and good in everything.", "William Shakespeare", "As You Like It"),
  quote("milton-good-book", "A good book is the precious lifeblood of a master spirit, embalmed and treasured up on purpose to a life beyond life.", "John Milton", "Areopagitica"),
  quote("milton-kill-book", "He who kills a good book kills reason itself.", "John Milton", "Areopagitica"),
  quote("sydney-speaking-pictures", "Look in thy heart and write.", "Sir Philip Sidney", "Astrophil and Stella"),
  quote("burton-library", "The chiefest of all pleasures, and the chiefest of all recreations, are in books.", "Robert Burton", "The Anatomy of Melancholy"),
  quote("dr-johnson-read", "The greatest part of a writer's time is spent in reading, in order to write; a man will turn over half a library to make one book.", "Samuel Johnson", "The Rambler"),
  quote("dr-johnson-companions", "A man will turn over half a library to make one book.", "Samuel Johnson", "The Rambler"),
  quote("swift-book", "When I am reading a book, whether wise or silly, it seems to me to be alive and talking to me.", "Jonathan Swift", "A Tale of a Tub"),
  quote("pope-learning", "A little learning is a dangerous thing; drink deep, or taste not the Pierian spring.", "Alexander Pope", "An Essay on Criticism"),
  quote("addison-reading", "Reading is to the mind what exercise is to the body.", "Joseph Addison", "The Tatler"),
  quote("fielding-knowledge", "A knowledge of the world is only to be acquired in the world, and not in a closet.", "Henry Fielding", "Tom Jones"),
  quote("goldsmith-writing", "Of all kinds of knowledge, the most useful is that which teaches us how to live well.", "Oliver Goldsmith", "The Bee"),
  quote("blake-world", "To see a World in a Grain of Sand and a Heaven in a Wild Flower.", "William Blake", "Auguries of Innocence"),
  quote("wordsworth-books", "Books! 'tis a dull and endless strife: Come, hear the woodland linnet.", "William Wordsworth", "The Tables Turned"),
  quote("coleridge-read", "Read not to contradict and confute, nor to believe and take for granted, nor to find talk and discourse, but to weigh and consider.", "Samuel Taylor Coleridge", "Aids to Reflection"),
  quote("lamb-books", "I love to lose myself in other men's minds.", "Charles Lamb", "Detached Thoughts on Books and Reading"),
  quote("lamb-old-books", "That is a noble library of yours; I have no such thing, but I have books.", "Charles Lamb", "Letters"),
  quote("hazlitt-reading", "One would think that the whole essence of reading was to be found in the word itself: reading.", "William Hazlitt", "On Reading Old Books"),
  quote("hazlitt-books", "The things we read become a part of us.", "William Hazlitt", "On Reading Old Books"),
  quote("carlyle-university", "The true university of these days is a collection of books.", "Thomas Carlyle", "On Heroes, Hero-Worship, and the Heroic in History"),
  quote("carlyle-book", "All that mankind has done, thought, gained, or been: it is lying as in magic preservation in the pages of books.", "Thomas Carlyle", "On the Choice of Books"),
  quote("ruskin-books", "All books are divisible into two classes, the books of the hour, and the books of all time.", "John Ruskin", "Sesame and Lilies"),
  quote("ruskin-author", "You might read all the books in the British Museum, and remain an utterly illiterate, uneducated person.", "John Ruskin", "Sesame and Lilies"),
  quote("emerson-best-books", "The best books are not read even by those whom they are supposed to inspire.", "Ralph Waldo Emerson", "Books"),
  quote("emerson-scholar", "Books are the best of things, well used; abused, among the worst.", "Ralph Waldo Emerson", "The American Scholar"),
  quote("thoreau-books", "How many a man has dated a new era in his life from the reading of a book.", "Henry David Thoreau", "Walden"),
  quote("thoreau-read", "Read the best books first, or you may not have a chance to read them at all.", "Henry David Thoreau", "A Week on the Concord and Merrimack Rivers"),
  quote("melville-books", "To produce a mighty book, you must choose a mighty theme.", "Herman Melville", "Moby-Dick"),
  quote("dickens-books", "There are books of which the backs and covers are by far the best parts.", "Charles Dickens", "Oliver Twist"),
  quote("eliot-books", "The strongest principle of growth lies in human choice.", "George Eliot", "Daniel Deronda"),
  quote("bronte-reading", "I am no bird; and no net ensnares me: I am a free human being with an independent will.", "Charlotte Brontë", "Jane Eyre"),
  quote("austen-reading", "If a book is well written, I always find it too short.", "Jane Austen", "Letters"),
  quote("alcott-books", "I like good strong words that mean something.", "Louisa May Alcott", "Little Women"),
  quote("twain-classics", "A classic is something that everybody wants to have read and nobody wants to read.", "Mark Twain", "Following the Equator"),
  quote("twain-good-friends", "Good friends, good books, and a sleepy conscience: this is the ideal life.", "Mark Twain", "Notebook"),
  quote("whitman-book", "To have great poets, there must be great audiences.", "Walt Whitman", "Preface to Leaves of Grass"),
  quote("longfellow-book", "The human voice is the organ of the soul.", "Henry Wadsworth Longfellow", "Table Talk"),
  quote("stevenson-books", "Books are good enough in their own way, but they are a mighty bloodless substitute for life.", "Robert Louis Stevenson", "An Inland Voyage"),
  quote("stevenson-reading", "The most beautiful adventures are not those we go to seek.", "Robert Louis Stevenson", "Virginibus Puerisque"),
  quote("wilde-books", "There is no such thing as a moral or an immoral book. Books are well written, or badly written.", "Oscar Wilde", "The Picture of Dorian Gray"),
  quote("wilde-reading", "If one cannot enjoy reading a book over and over again, there is no use in reading it at all.", "Oscar Wilde", "The Picture of Dorian Gray"),
  quote("gissing-reading", "I can sit reading, quietly reading, all day long.", "George Gissing", "The Private Papers of Henry Ryecroft", GUTENBERG_RYECROFT),
  quote("gissing-catalogues", "It is a joy to go through booksellers' catalogues, ticking here and there a possible purchase.", "George Gissing", "The Private Papers of Henry Ryecroft", GUTENBERG_RYECROFT),
  quote("gissing-museum", "I settled myself at a desk in the great Reading-Room with books before me which by no possibility could be a source of immediate profit.", "George Gissing", "The Private Papers of Henry Ryecroft", GUTENBERG_RYECROFT),
  quote("schopenhauer-other-thinks", "When we read, another person thinks for us: we merely repeat his mental process.", "Arthur Schopenhauer", "On Reading and Books", GUTENBERG_SCHOPENHAUER),
  quote("schopenhauer-twice", "Any kind of important book should immediately be read twice.", "Arthur Schopenhauer", "On Reading and Books", GUTENBERG_SCHOPENHAUER),
  quote("schopenhauer-works", "Works are the quintessence of a mind.", "Arthur Schopenhauer", "On Reading and Books", GUTENBERG_SCHOPENHAUER),
  quote("schopenhauer-classics", "There is nothing that so greatly recreates the mind as the works of the old classic writers.", "Arthur Schopenhauer", "On Reading and Books", GUTENBERG_SCHOPENHAUER),
  quote("quiller-couch-practice", "I propose to talk of the Art and Practice of Reading.", "Arthur Quiller-Couch", "On the Art of Reading", GUTENBERG_READING),
  quote("quiller-couch-mastering", "Reading is the capacity for silent reading, taking a book apart and mastering it.", "Arthur Quiller-Couch", "On the Art of Reading", GUTENBERG_READING),
  quote("quiller-couch-literature", "Great literature never condescends.", "Arthur Quiller-Couch", "On the Art of Reading", GUTENBERG_READING),
  quote("baldwin-choice", "The most important question for you to ask yourself is this: What books shall I read?", "James Baldwin", "The Book-lover", GUTENBERG_BOOK_LOVER),
  quote("baldwin-reading", "It is not every harmless book, nor indeed every good book, that will make your mind the richer for the reading of it.", "James Baldwin", "The Book-lover", GUTENBERG_BOOK_LOVER),
  quote("baldwin-treasure", "A pure, fresh book for a little child is a treasure to be sought for and appreciated.", "James Baldwin", "The Book-lover", GUTENBERG_BOOK_LOVER),
  quote("franklin-reading", "From my infancy, I was passionately fond of reading, and all the money that came into my hands was laid out in purchasing books.", "Benjamin Franklin", "Autobiography"),
  quote("franklin-reading-advantage", "There was among them Plutarch's Lives, which I read abundantly, and I still think the time spent to great advantage.", "Benjamin Franklin", "Autobiography"),
  quote("miller-reading", "The art of reading is the art of finding stories in books.", "Hugh Miller", "My Schools and Schoolmasters"),
  quote("de-quincey-books", "Books are not absolutely dead things, but do preserve as in a vial the purest efficacy and extraction of that living intellect that bred them.", "Thomas De Quincey", "Literary Reminiscences"),
  quote("newman-reading", "A great memory does not make a philosopher, any more than a dictionary can be called a grammar.", "John Henry Newman", "The Idea of a University"),
  quote("lowell-books", "Books are the bees which carry the quickening pollen from one to another mind.", "James Russell Lowell", "Among My Books"),
  quote("lowell-new-books", "Reading new books is like eating new bread.", "James Russell Lowell", "Fables for Critics"),
  quote("holmes-library", "A man's library is a sort of harem.", "Oliver Wendell Holmes Sr.", "The Professor at the Breakfast-Table"),
  quote("chesterton-reading", "The way to love anything is to realize that it may be lost.", "G. K. Chesterton", "Tremendous Trifles"),
  quote("conrad-books", "A work that aspires, however humbly, to the condition of art should carry its justification in every line.", "Joseph Conrad", "Preface to The Nigger of the Narcissus"),
  quote("yeats-reading", "Education is not the filling of a pail, but the lighting of a fire.", "W. B. Yeats", "Ideas of Good and Evil"),
  quote("woolf-books", "I can only note that the past is beautiful because one never realises an emotion at the time.", "Virginia Woolf", "The Common Reader"),
  quote("wells-books", "The path to education lies in a study of the things that interest us.", "H. G. Wells", "The Outline of History"),
  quote("kipling-books", "If history were taught in the form of stories, it would never be forgotten.", "Rudyard Kipling", "Collected Speeches"),
  quote("dickinson-books", "To travel far, there is no better ship than a book.", "Emily Dickinson", "There Is No Frigate Like a Book"),
  quote("dickinson-page", "Nor any coursers like a page of prancing poetry.", "Emily Dickinson", "There Is No Frigate Like a Book"),
  quote("emily-bronte-books", "No coward soul is mine.", "Emily Brontë", "Last Lines"),
  quote("mary-wollstonecraft-reading", "I do not wish them to have power over men; but over themselves.", "Mary Wollstonecraft", "A Vindication of the Rights of Woman"),
  quote("mary-shelley-books", "The beginning is always today.", "Mary Wollstonecraft Shelley", "Letters"),
  quote("goethe-books", "He who is ignorant of foreign languages knows nothing of his own.", "Johann Wolfgang von Goethe", "Maxims and Reflections"),
  quote("goethe-reading", "The person who understands nothing but chemistry does not understand that either.", "Johann Wolfgang von Goethe", "Conversations with Eckermann"),
  quote("cervantes-books", "He who reads much and walks much, sees much and knows much.", "Miguel de Cervantes", "Don Quixote"),
  quote("calvino-ancestors", "A classic is a book that has never finished saying what it has to say.", "Italo Calvino", "Why Read the Classics?"),
  quote("tolstoy-books", "All great literature is one of two stories; a man goes on a journey or a stranger comes to town.", "Leo Tolstoy", "Attributed saying"),
  quote("dostoevsky-reading", "Taking a new step, uttering a new word, is what people fear most.", "Fyodor Dostoevsky", "Crime and Punishment"),
  quote("pessoa-reading", "Literature is the most agreeable way of ignoring life.", "Fernando Pessoa", "The Book of Disquiet"),
  quote("pessoa-books", "To read is to dream, guided by another's hand.", "Fernando Pessoa", "The Book of Disquiet"),
  quote("tagore-books", "The butterfly counts not months but moments, and has time enough.", "Rabindranath Tagore", "Stray Birds"),
  quote("tagore-reading", "The highest education is that which does not merely give us information but makes our life in harmony with all existence.", "Rabindranath Tagore", "Sadhana"),
  quote("gandhi-books", "Live as if you were to die tomorrow. Learn as if you were to live forever.", "Mahatma Gandhi", "Attributed saying"),
  quote("tolkien-books", "Not all those who wander are lost.", "J. R. R. Tolkien", "The Fellowship of the Ring"),
  quote("joyce-read", "Think you're escaping and run into yourself. Longest way round is the shortest way home.", "James Joyce", "Ulysses"),
  quote("rainer-reading", "The only journey is the one within.", "Rainer Maria Rilke", "Letters to a Young Poet"),
  quote("anatole-books", "Never lend books, for no one ever returns them; the only books I have in my library are those which other people have lent me.", "Anatole France", "The Crime of Sylvestre Bonnard"),
  quote("mauriac-reading", "Tell me what you read and I will tell you who you are.", "François Mauriac", "Mémoires intérieurs"),
  quote("cather-books", "Only the stupid and the phlegmatic should teach.", "Willa Cather", "The Professor's House"),
  quote("montaigne-books", "I have always considered that the principal use of books was to afford me pleasure.", "Michel de Montaigne", "Essays"),
  quote("montaigne-library", "I have three kinds of commerce: with my friends, with the fair ones, and with books.", "Michel de Montaigne", "Essays"),
  quote("pascal-reading", "The last thing we find in making a book is to know what we must put first.", "Blaise Pascal", "Pensées"),
  quote("voltaire-books", "Let us read, and let us dance; these two amusements will never do any harm to the world.", "Voltaire", "Candide"),
  quote("diderot-books", "Only passions, great passions, can elevate the soul to great things.", "Denis Diderot", "Pensées philosophiques"),
  quote("locke-reading", "Reading furnishes the mind only with materials of knowledge; it is thinking that makes what we read ours.", "John Locke", "Of the Conduct of the Understanding"),
  quote("hume-books", "Be a philosopher; but, amidst all your philosophy, be still a man.", "David Hume", "An Enquiry Concerning Human Understanding"),
  quote("kant-reading", "Science is organized knowledge. Wisdom is organized life.", "Immanuel Kant", "Lectures on Pedagogy"),
  quote("aristotle-books", "The energy of the mind is the essence of life.", "Aristotle", "Nicomachean Ethics"),
  quote("plato-books", "Books give a soul to the universe, wings to the mind, flight to the imagination, and life to everything.", "Plato", "Attributed saying"),
  quote("confucius-reading", "No matter how busy you may think you are, you must find time for reading, or surrender yourself to self-chosen ignorance.", "Confucius", "Attributed saying"),
  quote("ranganathan-library", "Books are for use.", "S. R. Ranganathan", "The Five Laws of Library Science"),
  quote("ranganathan-reader", "Every reader his or her book.", "S. R. Ranganathan", "The Five Laws of Library Science"),
  quote("ranganathan-book", "Every book its reader.", "S. R. Ranganathan", "The Five Laws of Library Science"),
  quote("ranganathan-time", "Save the time of the reader.", "S. R. Ranganathan", "The Five Laws of Library Science"),
  quote("ranganathan-growing", "The library is a growing organism.", "S. R. Ranganathan", "The Five Laws of Library Science"),
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
