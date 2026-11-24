// Local design-study fixtures. Never imported into the desktop application.
const books = [
  { id: 1, title: "Balyakalasakhi", authors: ["Vaikom Muhammad Basheer"], year: 1944, publisher: "DC Books", classification: "894.8123", subjects: ["Fiction"], availableCopies: 2, totalCopies: 3, locations: ["Main library · A1"], color: "#D8E2CF", ink: "#243D2D", language: "Malayalam" },
  { id: 2, title: "The God of Small Things", authors: ["Arundhati Roy"], year: 1997, publisher: "IndiaInk", classification: "823", subjects: ["Fiction"], availableCopies: 0, totalCopies: 2, locations: ["Main library · A2"], color: "#DED4DF", ink: "#4D354D", language: "English" },
  { id: 3, title: "Wings of Fire", authors: ["A. P. J. Abdul Kalam", "Arun Tiwari"], year: 1999, publisher: "Universities Press", classification: "629.1092", subjects: ["Biography"], availableCopies: 1, totalCopies: 1, locations: ["Main library · B1"], color: "#E7C9AC", ink: "#573D28", language: "English" },
  { id: 4, title: "A Brief History of Time", authors: ["Stephen Hawking"], year: 1988, publisher: "Bantam Books", classification: "523.1", subjects: ["Science"], availableCopies: 1, totalCopies: 2, locations: ["Main library · C1"], color: "#CDDCE7", ink: "#263C54", language: "English" },
  { id: 5, title: "The Discovery of India", authors: ["Jawaharlal Nehru"], year: 1946, publisher: null, classification: "954", subjects: ["History"], availableCopies: 1, totalCopies: 2, locations: ["Main library · D1"], color: "#E1D6BA", ink: "#51462B", language: "English" },
  { id: 6, title: "A Room of One’s Own", authors: ["Virginia Woolf"], year: 1929, publisher: "Hogarth Press", classification: "824", subjects: ["Essays"], availableCopies: 0, totalCopies: 1, locations: ["Main library · D2"], color: "#D3DFDD", ink: "#304D48", language: "English" },
  { id: 7, title: "Little Women", authors: ["Louisa May Alcott"], year: 1868, publisher: null, classification: "813", subjects: ["Fiction"], availableCopies: 2, totalCopies: 2, locations: ["Main library · A3"], color: "#E3D2CD", ink: "#57342E", language: "English" },
  { id: 8, title: "The Guide", authors: ["R. K. Narayan"], year: 1958, publisher: null, classification: "823", subjects: ["Fiction"], availableCopies: 1, totalCopies: 1, locations: ["Main library · A2"], color: "#D8D7BF", ink: "#42472F", language: "English" },
];
const view = ["shelf", "index", "table"].includes(new URLSearchParams(window.location.search).get("view")) ? new URLSearchParams(window.location.search).get("view") : "shelf";
document.body.dataset.view = view;
document.querySelector(`.study-bar a[href="?view=${view}"]`).setAttribute("aria-current", "page");
const icon = '<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true"><circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/></svg>';
const brand = '<a class="brand" href="?view='+view+'"><img src="../../branding/pusthakam-logo.png" alt="" width="34" height="34"><span>Pusthakam</span></a>';
const masthead = `<header class="masthead">${brand}<span class="library-name">Town Library</span><nav aria-label="Library"><a href="#search" aria-current="page">Catalogue</a><button class="about-button">About the library</button></nav></header>`;
const form = `<form id="search-form" role="search"><label class="sr-only" for="search">Search the catalogue</label><div class="search-input">${icon}<input id="search" type="search" placeholder="Title, author or ISBN" autocomplete="off"><button type="button" id="clear" hidden aria-label="Clear search">×</button></div><button class="primary" type="submit">Search</button></form>`;
const subjects = ["All titles", "Fiction", "Biography", "Science", "History", "Essays"];
const filters = `<div class="subjects" role="group" aria-label="Filter by subject">${subjects.map((s,i)=>`<button data-subject="${s}" aria-pressed="${i === 0}">${s}</button>`).join("")}</div>`;
const availability = '<label class="availability-filter"><input type="checkbox" id="available"> Available copies only</label>';
const sort = '<label class="sort">Sort by <select id="sort"><option value="collection">Collection order</option><option value="title">Title A–Z</option><option value="year">Publication year</option></select></label>';
const intro = view === "index" ? `<section class="index-intro"><h1>Library catalogue</h1><p>Search the collection. Find your copy.</p>${form}</section>` : view === "table" ? `<section class="table-intro"><div><span class="table-rule"></span><h1>There’s a book<br>for that.</h1><p>A question. A quiet afternoon. A different way of seeing.</p></div><div class="table-search"><p>Start with a title, a name, or a subject.</p>${form}<a href="#collection">Or browse the collection <span aria-hidden="true">↓</span></a></div></section>` : `<section class="shelf-intro"><div class="intro-title"><h1>Find a book.</h1><p>Something familiar.<br>Something you haven’t met yet.</p></div>${form}</section>`;
document.querySelector("#app").innerHTML = `${masthead}<main>${intro}<section id="collection" class="collection">${view === "index" ? `<aside class="filter-rail"><h2>Narrow your search</h2>${filters}${availability}<p class="rail-help">Not sure of the title?<br>Try an author’s surname.</p></aside>` : `<div class="browse-bar">${filters}${availability}</div>`}<div class="collection-body"><div class="results-bar"><div><h2>${view === "table" ? "The reading table" : view === "index" ? "All titles" : "On the shelves"}</h2><p id="count" role="status" aria-live="polite"></p></div>${sort}</div><div id="results"></div><div id="empty" hidden><h3>No books found</h3><p>Try fewer words, another author, or remove a filter.</p><button id="reset" class="primary">Show all titles</button></div><footer class="collection-footer"><span id="range"></span><span>Town Library · Demonstration catalogue</span></footer></div></section></main><footer class="site-footer"><span>${view === "table" ? "A little time, well read." : "Find it here. Read it anywhere."}</span><span>Catalogue by <strong>Pusthakam</strong></span></footer>`;
let subject = "All titles";
let query = "";
const escape = value => String(value).replace(/[&<>"']/g, c=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]));
function jacket(book) {
  return `<div class="jacket" style="--jacket:${book.color};--jacket-ink:${book.ink}" aria-hidden="true"><div class="jacket-top"><span>Town Library</span><span>${book.classification}</span></div><div class="jacket-title">${escape(book.title)}</div><div class="jacket-author">${escape(book.authors[0])}</div><div class="jacket-bottom">Cover not added</div></div>`;
}
function status(book) { return book.availableCopies > 0 ? `<span class="status available"><i></i>${book.availableCopies} of ${book.totalCopies} available</span>` : '<span class="status borrowed"><i></i>All copies on loan</span>'; }
function render() {
  const available = document.querySelector("#available").checked;
  const order = document.querySelector("#sort").value;
  const words = query.toLocaleLowerCase().split(/\s+/).filter(Boolean);
  let rows = books.filter(b => (subject === "All titles" || b.subjects.includes(subject)) && (!available || b.availableCopies > 0) && words.every(w => `${b.title} ${b.authors.join(" ")} ${b.subjects.join(" ")} ${b.classification}`.toLocaleLowerCase().includes(w)));
  if (order === "title") rows = [...rows].sort((a,b)=>a.title.localeCompare(b.title));
  if (order === "year") rows = [...rows].sort((a,b)=>b.year-a.year);
  document.querySelector("#results").innerHTML = rows.map((b,i)=>`<article class="book" style="--book-color:${b.color}"><button class="cover-button" data-details="${b.id}" aria-label="View ${escape(b.title)}">${jacket(b)}</button><div class="book-information"><div class="book-subject">${escape(b.subjects[0])}<span class="book-position">${String(i+1).padStart(2,"0")}</span></div><h3><button data-details="${b.id}">${escape(b.title)}</button></h3><p class="author">${escape(b.authors.join(" & "))}</p><p class="bibliography">${b.year}<span>${escape(b.publisher || b.subjects[0])}</span><code>${b.classification}</code></p></div><div class="book-holdings">${status(b)}<p>${escape(b.locations[0])}</p><button class="details-link" data-details="${b.id}">View details <span aria-hidden="true">↗</span></button></div></article>`).join("");
  document.querySelector("#empty").hidden = rows.length > 0;
  document.querySelector("#count").textContent = `${rows.length} ${rows.length === 1 ? "title" : "titles"}${query ? ` matching “${query}”` : " in the collection"}`;
  document.querySelector("#range").textContent = rows.length ? `Showing ${rows.length} of ${rows.length} titles` : "No matching titles";
  document.querySelector("#clear").hidden = !document.querySelector("#search").value;
}
function showDetails(id) {
  const b = books.find(b=>b.id === Number(id));
  if (!b) return;
  document.querySelector("#detail-content").innerHTML = `<div class="detail-layout">${jacket(b)}<div><p class="detail-subject">${escape(b.subjects[0])}</p><h2 id="detail-title">${escape(b.title)}</h2><p class="detail-author">${escape(b.authors.join(" & "))}</p>${status(b)}<dl><dt>Publication year</dt><dd>${b.year}</dd><dt>Publisher</dt><dd>${escape(b.publisher || "Not recorded")}</dd><dt>Classification</dt><dd><code>${b.classification}</code></dd><dt>Location</dt><dd>${escape(b.locations.join(", "))}</dd><dt>Copies</dt><dd>${b.availableCopies} available / ${b.totalCopies} total</dd></dl><p class="detail-note">Ask at the library desk for help finding this book.</p></div></div>`;
  const dialog = document.querySelector("#details"); dialog.setAttribute("aria-labelledby","detail-title"); dialog.showModal();
}
document.querySelector("#search-form").addEventListener("submit", event => {event.preventDefault();query = document.querySelector("#search").value.trim();render();});
document.querySelector("#search").addEventListener("input", event => {query=event.target.value.trim();render();});
document.querySelector("#clear").addEventListener("click",()=>{document.querySelector("#search").value="";query="";render();document.querySelector("#search").focus();});
document.querySelectorAll("[data-subject]").forEach(button=>button.addEventListener("click",()=>{subject=button.dataset.subject;document.querySelectorAll("[data-subject]").forEach(b=>b.setAttribute("aria-pressed",String(b===button)));render();}));
document.querySelector("#available").addEventListener("change",render);
document.querySelector("#sort").addEventListener("change",render);
document.querySelector("#results").addEventListener("click",event=>{const button=event.target.closest("[data-details]");if(button)showDetails(button.dataset.details);});
document.querySelector("#reset").addEventListener("click",()=>{subject="All titles";query="";document.querySelector("#search").value="";document.querySelector("#available").checked=false;document.querySelectorAll("[data-subject]").forEach(b=>b.setAttribute("aria-pressed",String(b.dataset.subject===subject)));render();});
document.querySelectorAll("dialog .close").forEach(button=>button.addEventListener("click",()=>button.closest("dialog").close()));
document.querySelector(".about-button").addEventListener("click",()=>document.querySelector("#about").showModal());
render();
