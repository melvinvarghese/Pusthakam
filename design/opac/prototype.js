// Pusthakam OPAC Prototype Engine — Curated High-Craft Library Discovery
// Demonstration records with authentic bilingual metadata, Dewey Decimal classifications, and physical shelf coordinates.

const books = [
  {
    id: 1,
    title: "Balyakalasakhi",
    titleNative: "ബാല്യകാലസഖി",
    authors: ["Vaikom Muhammad Basheer"],
    year: 1944,
    publisher: "DC Books",
    classification: "894.8123 BAS",
    deweyClass: "800 Literature",
    subjects: ["Fiction", "Malayalam"],
    availableCopies: 2,
    totalCopies: 3,
    location: "Main Library · Wing A",
    shelf: "Stack 1 · Shelf 3",
    color: "#1E3A2F",
    ink: "#E6F3EC",
    accent: "#D4A373",
    language: "Malayalam",
    isbn: "978-81-264-0012-4",
    pages: 128,
    format: "Hardcover",
    description: "A landmark romantic tragedy in Malayalam literature depicting the childhood love and subsequent trials of Majeed and Suhra."
  },
  {
    id: 2,
    title: "The God of Small Things",
    titleNative: null,
    authors: ["Arundhati Roy"],
    year: 1997,
    publisher: "IndiaInk",
    classification: "823.914 ROY",
    deweyClass: "800 Literature",
    subjects: ["Fiction"],
    availableCopies: 0,
    totalCopies: 2,
    location: "Main Library · Wing A",
    shelf: "Stack 2 · Shelf 1",
    color: "#2C1E3A",
    ink: "#F2EBF7",
    accent: "#E0A96D",
    language: "English",
    isbn: "978-0-00-655068-6",
    pages: 340,
    format: "Paperback",
    description: "Booker Prize-winning story exploring the childhood experiences of fraternal twins in Ayemenem and the devastating impact of societal laws."
  },
  {
    id: 3,
    title: "Wings of Fire",
    titleNative: null,
    authors: ["A. P. J. Abdul Kalam", "Arun Tiwari"],
    year: 1999,
    publisher: "Universities Press",
    classification: "629.1092 KAL",
    deweyClass: "600 Technology & Applied Sciences",
    subjects: ["Biography", "Science"],
    availableCopies: 1,
    totalCopies: 1,
    location: "Main Library · Wing B",
    shelf: "Stack 1 · Shelf 4",
    color: "#4A2E18",
    ink: "#FAEDE2",
    accent: "#E28743",
    language: "English",
    isbn: "978-81-7371-146-6",
    pages: 180,
    format: "Paperback",
    description: "Autobiography of India's Missile Man and former President, tracing his early upbringing in Rameswaram through aerospace achievements."
  },
  {
    id: 4,
    title: "A Brief History of Time",
    titleNative: null,
    authors: ["Stephen Hawking"],
    year: 1988,
    publisher: "Bantam Books",
    classification: "523.1 HAW",
    deweyClass: "500 Natural Sciences & Mathematics",
    subjects: ["Science"],
    availableCopies: 1,
    totalCopies: 2,
    location: "Main Library · Wing C",
    shelf: "Stack 1 · Shelf 2",
    color: "#182A3A",
    ink: "#E1EEF7",
    accent: "#64B5F6",
    language: "English",
    isbn: "978-0-553-38016-3",
    pages: 212,
    format: "Hardcover",
    description: "A landmark popular-science introduction to cosmology, black holes, the Big Bang, and the search for a unifying theory of physics."
  },
  {
    id: 5,
    title: "The Discovery of India",
    titleNative: null,
    authors: ["Jawaharlal Nehru"],
    year: 1946,
    publisher: "Signet Press",
    classification: "954 NEH",
    deweyClass: "900 History & Geography",
    subjects: ["History"],
    availableCopies: 2,
    totalCopies: 2,
    location: "Main Library · Wing D",
    shelf: "Stack 1 · Shelf 1",
    color: "#3A3018",
    ink: "#F9F3DF",
    accent: "#D4AF37",
    language: "English",
    isbn: "978-0-14-303103-1",
    pages: 642,
    format: "Hardcover",
    description: "Written during Nehru's imprisonment at Ahmednagar Fort (1942–1946), synthesizing the philosophical, cultural, and political history of India."
  },
  {
    id: 6,
    title: "A Room of One’s Own",
    titleNative: null,
    authors: ["Virginia Woolf"],
    year: 1929,
    publisher: "Hogarth Press",
    classification: "824.912 WOO",
    deweyClass: "800 Literature",
    subjects: ["Essays", "Literature"],
    availableCopies: 0,
    totalCopies: 1,
    location: "Main Library · Wing D",
    shelf: "Stack 2 · Shelf 3",
    color: "#1C3632",
    ink: "#E4F4F0",
    accent: "#80CBC4",
    language: "English",
    isbn: "978-0-15-678733-8",
    pages: 112,
    format: "Paperback",
    description: "An essential feminist essay based on lectures given at Cambridge on women writers and the material conditions necessary for creative freedom."
  },
  {
    id: 7,
    title: "Little Women",
    titleNative: null,
    authors: ["Louisa May Alcott"],
    year: 1868,
    publisher: "Roberts Brothers",
    classification: "813.4 ALC",
    deweyClass: "800 Literature",
    subjects: ["Fiction"],
    availableCopies: 2,
    totalCopies: 2,
    location: "Main Library · Wing A",
    shelf: "Stack 3 · Shelf 2",
    color: "#46221B",
    ink: "#FAECE9",
    accent: "#E57373",
    language: "English",
    isbn: "978-0-14-751401-1",
    pages: 448,
    format: "Paperback",
    description: "Classic American coming-of-age novel following the four March sisters—Meg, Jo, Beth, and Amy—in 19th-century New England."
  },
  {
    id: 8,
    title: "The Guide",
    titleNative: null,
    authors: ["R. K. Narayan"],
    year: 1958,
    publisher: "Viking Press",
    classification: "823.912 NAR",
    deweyClass: "800 Literature",
    subjects: ["Fiction"],
    availableCopies: 1,
    totalCopies: 1,
    location: "Main Library · Wing A",
    shelf: "Stack 2 · Shelf 4",
    color: "#2F381E",
    ink: "#EEF4E3",
    accent: "#AED581",
    language: "English",
    isbn: "978-0-14-303964-8",
    pages: 220,
    format: "Paperback",
    description: "Sahitya Akademi Award-winning novel recounting the transformation of Raju from a corrupt tour guide in Malgudi to an unexpected spiritual guide."
  },
  {
    id: 9,
    title: "Randamoozham",
    titleNative: "രണ്ടാമൂഴം",
    authors: ["M. T. Vasudevan Nair"],
    year: 1984,
    publisher: "Current Books",
    classification: "894.8123 VAS",
    deweyClass: "800 Literature",
    subjects: ["Fiction", "Malayalam"],
    availableCopies: 3,
    totalCopies: 3,
    location: "Main Library · Wing A",
    shelf: "Stack 1 · Shelf 2",
    color: "#1E2C38",
    ink: "#E6EEF5",
    accent: "#90CAF9",
    language: "Malayalam",
    isbn: "978-81-226-0157-2",
    pages: 310,
    format: "Hardcover",
    description: "Vayalar Award-winning masterpiece retelling the Mahabharata epic from the introspective and poignant perspective of Bhima."
  },
  {
    id: 10,
    title: "Effective Java",
    titleNative: null,
    authors: ["Joshua Bloch"],
    year: 2018,
    publisher: "Addison-Wesley",
    classification: "005.133 BLO",
    deweyClass: "000 Computer Science & Information",
    subjects: ["Science"],
    availableCopies: 2,
    totalCopies: 3,
    location: "Main Library · Wing E",
    shelf: "Stack 2 · Shelf 1",
    color: "#252B35",
    ink: "#F0F2F5",
    accent: "#81D4FA",
    language: "English",
    isbn: "978-0-13-468599-1",
    pages: 412,
    format: "Paperback",
    description: "The definitive guide to best practices for the Java programming language, covering lambdas, streams, generics, and concurrent design."
  }
];

// Determine view from URL params (with aliases)
const urlParams = new URLSearchParams(window.location.search);
let viewParam = urlParams.get("view");
let view = "reading-room";

if (viewParam === "swiss-stacks" || viewParam === "index") {
  view = "swiss-stacks";
} else if (viewParam === "sovereign-folio" || viewParam === "table") {
  view = "sovereign-folio";
} else if (viewParam === "reading-room" || viewParam === "shelf") {
  view = "reading-room";
}

document.body.dataset.view = view;

// Highlight active tab in switcher
document.querySelectorAll(".study-bar nav a").forEach(a => {
  const v = a.dataset.view;
  if (v === view) {
    a.setAttribute("aria-current", "page");
  } else {
    a.removeAttribute("aria-current");
  }
});

const escape = val => String(val || "").replace(/[&<>"']/g, c => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;"
}[c]));

// SVG Icons
const icons = {
  search: '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/></svg>',
  book: '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1-2.5-2.5Z"/><path d="M6 6h10"/><path d="M6 10h10"/></svg>',
  mapPin: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z"/><circle cx="12" cy="10" r="3"/></svg>',
  check: '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="20 6 9 17 4 12"/></svg>',
  clock: '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>',
  info: '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>',
  arrowUpRight: '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M7 17 17 7"/><path d="M7 7h10v10"/></svg>'
};

// Filter and state management
let activeSubject = "All";
let searchQuery = "";
let availableOnly = false;

// Main UI Construction
const app = document.querySelector("#app");

function buildShell() {
  const brand = `
    <div class="brand-cluster">
      <img src="../../branding/pusthakam-logo.png" alt="" width="32" height="32" class="brand-mark">
      <div class="brand-text">
        <span class="brand-title">Pusthakam</span>
        <span class="library-tag">Town Public Library · OPAC</span>
      </div>
    </div>
  `;

  const topNav = `
    <nav class="top-nav" aria-label="Library navigation">
      <span class="collection-pill"><span class="live-dot"></span> 10 volumes catalogued</span>
      <button class="nav-link about-trigger" type="button">About & Stacks</button>
    </nav>
  `;

  return `
    <header class="opac-header">
      <div class="header-inner">
        ${brand}
        ${topNav}
      </div>
    </header>
    <main id="main-content" class="opac-main"></main>
    <footer class="opac-footer">
      <div class="footer-inner">
        <div class="footer-col">
          <strong>Pusthakam Community & Research Library</strong>
          <p>Local-sovereign library management system. Fully offline & privacy respecting.</p>
        </div>
        <div class="footer-col right">
          <span>Dewey Decimal Classification Edition 23</span>
          <span class="muted">Powered by Pusthakam Desktop</span>
        </div>
      </div>
    </footer>
  `;
}

app.innerHTML = buildShell();
const main = document.querySelector("#main-content");

// -------------------------------------------------------------
// VIEW 1: THE READING ROOM (Archival Editorial & Cloth Spine Discovery)
// -------------------------------------------------------------
function renderReadingRoomHero() {
  return `
    <section class="reading-room-hero">
      <div class="hero-container">
        <div class="hero-title-group">
          <span class="hero-kicker">Town Public Library Catalogue</span>
          <h1 class="hero-headline">Every book has a quiet home on our shelves.</h1>
          <p class="hero-subtext">Search our open stacks across literature, Malayalam classics, sciences, and biographies.</p>
        </div>
        <div class="search-command-bar">
          <form id="search-form" class="reading-search-form" role="search">
            <div class="search-field-wrap">
              <span class="field-icon">${icons.search}</span>
              <input id="search-input" type="search" placeholder="Search by title, author, subject, or Dewey number…" autocomplete="off" spellcheck="false" value="${escape(searchQuery)}">
              <button type="button" id="search-clear" class="clear-btn" ${searchQuery ? "" : "hidden"} aria-label="Clear search">×</button>
            </div>
            <button type="submit" class="reading-search-submit">Search Catalogue</button>
          </form>
        </div>
        <div class="reading-filters-strip">
          <div class="filter-pills-scroll" role="group" aria-label="Subject filter">
            ${["All", "Fiction", "Malayalam", "Science", "Biography", "History", "Essays"].map(s => `
              <button type="button" class="subject-pill ${activeSubject === s ? "active" : ""}" data-subject="${s}">${s}</button>
            `).join("")}
          </div>
          <label class="avail-checkbox-label">
            <input type="checkbox" id="avail-check" ${availableOnly ? "checked" : ""}>
            <span>Available on shelf only</span>
          </label>
        </div>
      </div>
    </section>
  `;
}

// -------------------------------------------------------------
// VIEW 2: THE SWISS STACKS (Architectural Modernism & Typographic Ledger)
// -------------------------------------------------------------
function renderSwissStacksHero() {
  return `
    <section class="swiss-stacks-hero">
      <div class="swiss-hero-inner">
        <div class="swiss-masthead">
          <div class="swiss-tagline-cluster">
            <span class="swiss-kicker">PUBLIC ACCESS CATALOGUE · BIBLIOGRAPHIC INDEX</span>
            <h1 class="swiss-headline">The Swiss Stacks</h1>
            <p class="swiss-subtext">A high-density typographic catalogue of town library holdings, structured for rapid scanning and shelf location.</p>
          </div>
          <div class="swiss-badge-cluster">
            <span class="swiss-badge-item"><strong>10</strong> TITLES INDEXED</span>
            <span class="swiss-badge-item"><strong>OPEN ACCESS</strong> STACKS</span>
          </div>
        </div>

        <div class="swiss-search-bar-wrap">
          <form id="search-form" class="swiss-search-form" role="search">
            <div class="swiss-input-wrap">
              <span class="swiss-search-icon">${icons.search}</span>
              <input id="search-input" type="search" placeholder="Search by title, author, Dewey class (e.g. 894, 523), or shelf location…" autocomplete="off" spellcheck="false" value="${escape(searchQuery)}">
              <button type="button" id="search-clear" class="clear-btn" ${searchQuery ? "" : "hidden"} aria-label="Clear search">×</button>
            </div>
            <button type="submit" class="swiss-search-submit">Search Stacks</button>
          </form>
        </div>

        <!-- Horizontal Classification Ribbon -->
        <div class="swiss-spectrum-nav">
          <span class="spectrum-label">Classification:</span>
          <div class="swiss-chips-scroll" role="group" aria-label="Classification facets">
            ${[
              { name: "All", label: "All Classes", count: 10 },
              { name: "Fiction", label: "800 Literature", count: 5 },
              { name: "Malayalam", label: "Malayalam Classics", count: 2 },
              { name: "Science", label: "500 & 600 Sciences", count: 3 },
              { name: "History", label: "900 History", count: 1 },
              { name: "Biography", label: "Biographies", count: 1 }
            ].map(f => `
              <button type="button" class="swiss-spectrum-chip ${activeSubject === f.name ? "active" : ""}" data-subject="${f.name}">
                <span class="chip-name">${f.label}</span>
                <span class="chip-count">${f.count}</span>
              </button>
            `).join("")}
          </div>
          <div class="swiss-nav-aux">
            <label class="swiss-checkbox-label">
              <input type="checkbox" id="avail-check" ${availableOnly ? "checked" : ""}>
              <span>On-Shelf Only</span>
            </label>
            <span class="swiss-kbd-hint"><kbd>/</kbd> to search</span>
          </div>
        </div>
      </div>
    </section>
  `;
}

// -------------------------------------------------------------
// VIEW 3: THE SOVEREIGN FOLIO (Contemporary Book Arts & Cultural Showcase)
// -------------------------------------------------------------
function renderSovereignFolioHero() {
  return `
    <section class="folio-hero">
      <div class="folio-hero-inner">
        <div class="folio-header-split">
          <div>
            <h1 class="folio-title">The Sovereign Folio</h1>
            <p class="folio-subtitle">Preserving classical and contemporary literature in English & മലയാളം.</p>
          </div>
          <div class="folio-badge-box">
            <span class="folio-kicker">OPEN ACCESS</span>
            <span class="folio-hours">Mon–Sat: 08:30 – 19:00</span>
          </div>
        </div>
        <form id="search-form" class="folio-search-form" role="search">
          <div class="folio-search-field">
            <span class="folio-icon">${icons.search}</span>
            <input id="search-input" type="search" placeholder="Type a title, author name, or ISBN…" autocomplete="off" value="${escape(searchQuery)}">
            <button type="button" id="search-clear" class="clear-btn" ${searchQuery ? "" : "hidden"} aria-label="Clear search">×</button>
          </div>
          <button type="submit" class="folio-submit-btn">Search Books</button>
        </form>
        <div class="folio-chips-bar">
          <span class="folio-chips-label">Collections:</span>
          <div class="folio-chips-list" role="group">
            ${["All", "Fiction", "Malayalam", "Science", "Biography", "History", "Essays"].map(s => `
              <button type="button" class="folio-chip ${activeSubject === s ? "active" : ""}" data-subject="${s}">${s}</button>
            `).join("")}
          </div>
          <label class="folio-avail-toggle">
            <input type="checkbox" id="avail-check" ${availableOnly ? "checked" : ""}>
            <span class="toggle-track"></span>
            <span>Show on-shelf titles only</span>
          </label>
        </div>
      </div>
    </section>
  `;
}

function getFilteredBooks() {
  const queryWords = searchQuery.toLowerCase().split(/\s+/).filter(Boolean);
  return books.filter(b => {
    // Subject filter
    if (activeSubject !== "All" && !b.subjects.includes(activeSubject)) {
      return false;
    }
    // Availability filter
    if (availableOnly && b.availableCopies <= 0) {
      return false;
    }
    // Text search matching title, native title, authors, subject, or classification
    if (queryWords.length > 0) {
      const corpus = `${b.title} ${b.titleNative || ""} ${b.authors.join(" ")} ${b.subjects.join(" ")} ${b.classification} ${b.deweyClass} ${b.location} ${b.isbn}`.toLowerCase();
      const match = queryWords.every(w => corpus.includes(w));
      if (!match) return false;
    }
    return true;
  });
}

// -------------------------------------------------------------
// RENDERERS FOR RESULT LISTS
// -------------------------------------------------------------

function renderReadingRoomCards(filtered) {
  if (filtered.length === 0) {
    return `
      <div class="empty-state-box">
        <div class="empty-icon">${icons.book}</div>
        <h3>No books found in the collection</h3>
        <p>Try searching with another keyword or author name, or reset your subject filter.</p>
        <button type="button" class="reset-filter-btn" id="reset-filters">Show all collection titles</button>
      </div>
    `;
  }

  return `
    <div class="reading-room-grid">
      ${filtered.map(b => {
        const isAvail = b.availableCopies > 0;
        return `
          <article class="cloth-book-card" data-id="${b.id}" tabindex="0" role="button" aria-label="View details for ${escape(b.title)}">
            <div class="cloth-jacket" style="--cloth-bg: ${b.color}; --cloth-ink: ${b.ink}; --cloth-accent: ${b.accent};">
              <div class="jacket-emboss-frame">
                <div class="jacket-head">
                  <span class="jacket-lib">TOWN LIBRARY</span>
                  <span class="jacket-ddc">${escape(b.classification.split(" ")[0])}</span>
                </div>
                <div class="jacket-body">
                  <h3 class="jacket-title">${escape(b.title)}</h3>
                  ${b.titleNative ? `<div class="jacket-native">${escape(b.titleNative)}</div>` : ""}
                  <p class="jacket-author">${escape(b.authors[0])}</p>
                </div>
                <div class="jacket-foot">
                  <span class="jacket-year">${b.year}</span>
                  <span class="jacket-format">${escape(b.format)}</span>
                </div>
              </div>
            </div>
            <div class="card-meta">
              <div class="card-subject-row">
                <span class="card-subject-tag">${escape(b.subjects[0])}</span>
                <span class="card-ddc-code">${escape(b.classification)}</span>
              </div>
              <h2 class="card-book-title">${escape(b.title)}</h2>
              ${b.titleNative ? `<div class="card-book-native">${escape(b.titleNative)}</div>` : ""}
              <p class="card-book-authors">${escape(b.authors.join(", "))}</p>
              <div class="card-shelf-loc">
                <span class="loc-pin">${icons.mapPin}</span>
                <span>${escape(b.shelf)}</span>
              </div>
              <div class="card-status-bar">
                ${isAvail 
                  ? `<span class="avail-badge in-stock"><span class="badge-dot"></span> ${b.availableCopies} of ${b.totalCopies} on shelf</span>`
                  : `<span class="avail-badge on-loan"><span class="badge-dot"></span> All ${b.totalCopies} copies on loan</span>`
                }
                <button type="button" class="examine-btn" data-id="${b.id}">Examine Slip →</button>
              </div>
            </div>
          </article>
        `;
      }).join("")}
    </div>
  `;
}

function renderSwissStacksView(filtered) {
  return `
    <div class="swiss-stream-container">
      <div class="stream-header-strip">
        <span class="stream-count">${filtered.length} TITLES MATCHED</span>
        <span class="stream-hint">PHYSICAL SHELF ACCESSION ORDER</span>
      </div>

      ${filtered.length === 0 ? `
        <div class="empty-state-box">
          <div class="empty-icon">${icons.book}</div>
          <h3>No records match your filter criteria</h3>
          <p>Try searching with another keyword or author name, or clear your classification filters.</p>
          <button type="button" class="reset-filter-btn" id="reset-filters">Reset filters</button>
        </div>
      ` : `
        <div class="swiss-ledger-stream">
          ${filtered.map((b, idx) => {
            const isAvail = b.availableCopies > 0;
            return `
              <article class="swiss-row" data-id="${b.id}" tabindex="0" role="button" aria-label="View catalogue details for ${escape(b.title)}">
                <div class="col-ddc">
                  <span class="ddc-badge">${escape(b.classification)}</span>
                  <span class="row-seq">#${String(idx + 1).padStart(2, "0")} · ${escape(b.deweyClass.split(" ")[0])}</span>
                </div>
                <div class="col-biblio">
                  <div class="title-row">
                    <strong class="row-title">${escape(b.title)}</strong>
                    ${b.titleNative ? `<span class="row-native">${escape(b.titleNative)}</span>` : ""}
                  </div>
                  <div class="row-author-line">
                    <span class="author-name">By ${escape(b.authors.join(", "))}</span>
                    <span class="sep">/</span>
                    <span>${b.year}</span>
                    <span class="sep">/</span>
                    <span>${escape(b.publisher || "Library Edition")}</span>
                    <span class="sep">/</span>
                    <span class="format-pill">${escape(b.format)}</span>
                  </div>
                  <div class="row-desc">${escape(b.description)}</div>
                </div>
                <div class="col-shelf">
                  <span class="shelf-coord-label">SHELF LOCATION</span>
                  <div class="shelf-tag">${escape(b.shelf)}</div>
                  <div class="loc-text">${escape(b.location)}</div>
                </div>
                <div class="col-avail">
                  ${isAvail 
                    ? `<span class="swiss-status-pill in-stock">AVAILABLE (${b.availableCopies}/${b.totalCopies})</span>`
                    : `<span class="swiss-status-pill on-loan">ON LOAN (0/${b.totalCopies})</span>`
                  }
                  <span class="inspect-text">Inspect Details →</span>
                </div>
              </article>
            `;
          }).join("")}
        </div>
      `}
    </div>
  `;
}

function renderSovereignFolioCards(filtered) {
  return `
    <div class="folio-content-container">
      <div class="folio-results-bar">
        <div class="folio-result-count">
          Showing <strong>${filtered.length}</strong> of <strong>${books.length}</strong> volumes
        </div>
      </div>

      ${filtered.length === 0 ? `
        <div class="empty-state-box">
          <h3>No records located</h3>
          <p>Please adjust your query or collection filter.</p>
          <button type="button" class="reset-filter-btn" id="reset-filters">View all volumes</button>
        </div>
      ` : `
        <div class="folio-card-grid">
          ${filtered.map(b => {
            const isAvail = b.availableCopies > 0;
            return `
              <div class="folio-card" data-id="${b.id}" tabindex="0" role="button">
                <div class="folio-card-spine" style="background: ${b.color};">
                  <span class="spine-num">${escape(b.classification.split(" ")[0])}</span>
                  <span class="spine-mark">PUSTHAKAM</span>
                </div>
                <div class="folio-card-content">
                  <div class="folio-top-row">
                    <span class="folio-ddc-badge">${escape(b.classification)}</span>
                    <span class="folio-lang-badge">${escape(b.language)}</span>
                  </div>
                  <h3 class="folio-card-title">${escape(b.title)}</h3>
                  ${b.titleNative ? `<div class="folio-card-native">${escape(b.titleNative)}</div>` : ""}
                  <p class="folio-card-author">By ${escape(b.authors.join(", "))}</p>
                  <p class="folio-card-summary">${escape(b.description)}</p>
                  <div class="folio-card-footer">
                    <div class="folio-shelf-info">
                      <span class="shelf-icon">${icons.mapPin}</span>
                      <span>${escape(b.shelf)}</span>
                    </div>
                    ${isAvail 
                      ? `<span class="folio-status-badge available"><span class="dot"></span> ${b.availableCopies} Available</span>`
                      : `<span class="folio-status-badge borrowed"><span class="dot"></span> All on Loan</span>`
                    }
                  </div>
                </div>
              </div>
            `;
          }).join("")}
        </div>
      `}
    </div>
  `;
}

// -------------------------------------------------------------
// MAIN RENDER LOOP
// -------------------------------------------------------------
function render() {
  const filtered = getFilteredBooks();

  let heroHtml = "";
  let bodyHtml = "";

  if (view === "reading-room") {
    heroHtml = renderReadingRoomHero();
    bodyHtml = renderReadingRoomCards(filtered);
  } else if (view === "swiss-stacks") {
    heroHtml = renderSwissStacksHero();
    bodyHtml = renderSwissStacksView(filtered);
  } else if (view === "sovereign-folio") {
    heroHtml = renderSovereignFolioHero();
    bodyHtml = renderSovereignFolioCards(filtered);
  }

  main.innerHTML = heroHtml + `<div class="main-body-wrap">${bodyHtml}</div>`;
  attachEventListeners();
}

function attachEventListeners() {
  // Search Form Submit
  const form = document.querySelector("#search-form");
  if (form) {
    form.addEventListener("submit", e => {
      e.preventDefault();
      const input = document.querySelector("#search-input");
      searchQuery = input ? input.value.trim() : "";
      render();
    });
  }

  // Live search input
  const input = document.querySelector("#search-input");
  if (input) {
    input.addEventListener("input", e => {
      searchQuery = e.target.value.trim();
      const clearBtn = document.querySelector("#search-clear");
      if (clearBtn) clearBtn.hidden = !searchQuery;
      // Re-render results container without jumping focus
      updateResultsOnly();
    });
  }

  // Clear search button
  const clearBtn = document.querySelector("#search-clear");
  if (clearBtn) {
    clearBtn.addEventListener("click", () => {
      searchQuery = "";
      render();
      const inp = document.querySelector("#search-input");
      if (inp) inp.focus();
    });
  }

  // Subject pill filters
  document.querySelectorAll("[data-subject]").forEach(btn => {
    btn.addEventListener("click", () => {
      activeSubject = btn.dataset.subject;
      render();
    });
  });

  // Available copies only toggle
  const availCheck = document.querySelector("#avail-check");
  if (availCheck) {
    availCheck.addEventListener("change", e => {
      availableOnly = e.target.checked;
      render();
    });
  }

  // Reset filters button
  const resetBtn = document.querySelector("#reset-filters");
  if (resetBtn) {
    resetBtn.addEventListener("click", () => {
      searchQuery = "";
      activeSubject = "All";
      availableOnly = false;
      render();
    });
  }

  // Book Card Click & Keyboard Enter
  document.querySelectorAll("[data-id]").forEach(card => {
    const id = Number(card.dataset.id);
    card.addEventListener("click", () => showBookDetails(id));
    card.addEventListener("keydown", e => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        showBookDetails(id);
      }
    });
  });

  // About modal button
  const aboutBtn = document.querySelector(".about-trigger");
  if (aboutBtn) {
    aboutBtn.addEventListener("click", () => {
      const modal = document.querySelector("#about-dialog");
      if (modal) modal.showModal();
    });
  }
}

function updateResultsOnly() {
  const filtered = getFilteredBooks();
  const bodyWrap = document.querySelector(".main-body-wrap");
  if (!bodyWrap) return;

  if (view === "reading-room") {
    bodyWrap.innerHTML = renderReadingRoomCards(filtered);
  } else if (view === "swiss-stacks") {
    bodyWrap.innerHTML = renderSwissStacksView(filtered);
  } else if (view === "sovereign-folio") {
    bodyWrap.innerHTML = renderSovereignFolioCards(filtered);
  }

  // Live search replaces the empty state, so its reset action needs a listener too.
  document.querySelector("#reset-filters")?.addEventListener("click", () => {
    searchQuery = "";
    activeSubject = "All";
    availableOnly = false;
    render();
    document.querySelector("#search-input")?.focus();
  });

  // Re-attach card clicks
  document.querySelectorAll("[data-id]").forEach(card => {
    const id = Number(card.dataset.id);
    card.addEventListener("click", () => showBookDetails(id));
    card.addEventListener("keydown", e => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        showBookDetails(id);
      }
    });
  });
}

// -------------------------------------------------------------
// DETAIL SLIP MODAL
// -------------------------------------------------------------
function showBookDetails(id) {
  const book = books.find(b => b.id === id);
  if (!book) return;

  const modal = document.querySelector("#details-dialog");
  const content = document.querySelector("#details-dialog-content");
  if (!modal || !content) return;

  content.innerHTML = `
    <div class="detail-slip-layout">
      <div class="slip-spine-col" style="--book-color: ${book.color}; --book-ink: ${book.ink};">
        <div class="slip-spine-preview">
          <div class="spine-top-call">${escape(book.classification)}</div>
          <div class="spine-mid-title">${escape(book.title)}</div>
          <div class="spine-bot-author">${escape(book.authors[0])}</div>
        </div>
      </div>
      <div class="slip-info-col">
        <div class="slip-header">
          <span class="slip-dewey-badge">${escape(book.deweyClass)}</span>
          <span class="slip-call-no">${escape(book.classification)}</span>
        </div>
        <h2 class="slip-title">${escape(book.title)}</h2>
        ${book.titleNative ? `<div class="slip-native-title">${escape(book.titleNative)}</div>` : ""}
        <p class="slip-authors">By <strong>${escape(book.authors.join(", "))}</strong></p>
        <p class="slip-description">${escape(book.description)}</p>

        <div class="shelf-locator-panel">
          <div class="locator-header">
            <span class="loc-icon">${icons.mapPin}</span>
            <strong>Physical Shelf Location:</strong>
          </div>
          <div class="locator-coords">
            <div class="coord-box">
              <span class="coord-label">WING / FLOOR</span>
              <span class="coord-value">${escape(book.location.split("·")[1]?.trim() || "Main Ground")}</span>
            </div>
            <div class="coord-box">
              <span class="coord-label">STACK AISLE</span>
              <span class="coord-value">${escape(book.shelf.split("·")[0]?.trim() || "Stack 1")}</span>
            </div>
            <div class="coord-box">
              <span class="coord-label">SHELF TIER</span>
              <span class="coord-value">${escape(book.shelf.split("·")[1]?.trim() || "Tier 3")}</span>
            </div>
          </div>
        </div>

        <div class="holdings-summary-table">
          <table class="inventory-table">
            <thead>
              <tr>
                <th>Copy Barcode</th>
                <th>Accession</th>
                <th>Status</th>
                <th>Action</th>
              </tr>
            </thead>
            <tbody>
              ${Array.from({ length: book.totalCopies }).map((_, i) => {
                const copyAvail = i < book.availableCopies;
                const barcode = `PST-${book.id}-${String(i + 1).padStart(3, "0")}`;
                return `
                  <tr>
                    <td><code>${barcode}</code></td>
                    <td>ACC-${book.year % 100}${String(book.id).padStart(3, "0")}-${i + 1}</td>
                    <td>
                      ${copyAvail 
                        ? `<span class="table-status in-stock"><span class="dot"></span> In Stacks</span>` 
                        : `<span class="table-status on-loan"><span class="dot"></span> On Loan</span>`
                      }
                    </td>
                    <td>
                      ${copyAvail 
                        ? `<span class="note-staff">Available at desk</span>`
                        : `<span class="note-due">Due back soon</span>`
                      }
                    </td>
                  </tr>
                `;
              }).join("")}
            </tbody>
          </table>
        </div>

        <div class="slip-footer">
          <div class="meta-tags">
            <span>ISBN: <code>${escape(book.isbn)}</code></span>
            <span>Pages: ${book.pages}</span>
            <span>Format: ${escape(book.format)}</span>
            <span>Publisher: ${escape(book.publisher || "Library Edition")}</span>
          </div>
          <p class="staff-help-note">Need assistance retrieving this volume? Present this call number (<strong>${escape(book.classification)}</strong>) to the circulation desk librarian.</p>
        </div>
      </div>
    </div>
  `;

  modal.showModal();
}

// Global modal close handlers
document.querySelectorAll(".modal-close-btn").forEach(btn => {
  btn.addEventListener("click", () => {
    btn.closest("dialog").close();
  });
});

document.querySelectorAll("dialog").forEach(dialog => {
  dialog.addEventListener("click", e => {
    if (e.target === dialog) dialog.close();
  });
});

// Keyboard navigation global helper (slash focuses search)
window.addEventListener("keydown", e => {
  if (e.key === "/" && document.activeElement.tagName !== "INPUT") {
    e.preventDefault();
    const inp = document.querySelector("#search-input");
    if (inp) {
      inp.focus();
      inp.select();
    }
  }
});

// Initial boot
render();
