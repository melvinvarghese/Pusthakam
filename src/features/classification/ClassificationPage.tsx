import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { PageHeader } from "@/components/PageHeader";
import { Button } from "@/components/ui/Button";
import { TextField, SelectField } from "@/components/ui/Field";
import { Modal } from "@/components/ui/Modal";
import { Chip } from "@/components/ui/Chip";
import { Panel, PanelHeader } from "@/components/ui/Panel";
import { classificationApi, toApiError } from "@/lib/api";
import type {
  ClassificationEntry,
  AuthorAuthority,
  PublisherAuthority,
  SubjectAuthority,
  MarcTestResult,
} from "@/lib/api/types";
import { useSession, can } from "@/lib/session";

type Tab = "schedules" | "authors" | "publishers" | "subjects" | "marc";

export default function ClassificationPage() {
  const [tab, setTab] = useState<Tab>("schedules");
  const { session } = useSession();
  const writable = can(session, "classification.write") || can(session, "catalogue.write");

  return (
    <div className="staff-classification space-y-6">
      <PageHeader
        title="Classification & Authorities"
        description="Browse classification schedules (DDC, UDC, LCC, CC), manage controlled author, publisher, and subject authorities, and inspect MARC mappings."
      />

      <div className="flex border-b border-line gap-2 overflow-x-auto" role="tablist">
        <button
          role="tab"
          aria-selected={tab === "schedules"}
          onClick={() => setTab("schedules")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors whitespace-nowrap ${
            tab === "schedules" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Classification Schedules
        </button>
        <button
          role="tab"
          aria-selected={tab === "authors"}
          onClick={() => setTab("authors")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors whitespace-nowrap ${
            tab === "authors" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Author Authorities
        </button>
        <button
          role="tab"
          aria-selected={tab === "publishers"}
          onClick={() => setTab("publishers")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors whitespace-nowrap ${
            tab === "publishers" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Publisher Authorities
        </button>
        <button
          role="tab"
          aria-selected={tab === "subjects"}
          onClick={() => setTab("subjects")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors whitespace-nowrap ${
            tab === "subjects" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Subject Authorities
        </button>
        <button
          role="tab"
          aria-selected={tab === "marc"}
          onClick={() => setTab("marc")}
          className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors whitespace-nowrap ${
            tab === "marc" ? "border-primary text-primary" : "border-transparent text-muted hover:text-fg"
          }`}
        >
          Controlled MARC Mappings
        </button>
      </div>

      {tab === "schedules" && <SchedulesTab writable={writable} />}
      {tab === "authors" && <AuthorsTab writable={writable} />}
      {tab === "publishers" && <PublishersTab writable={writable} />}
      {tab === "subjects" && <SubjectsTab writable={writable} />}
      {tab === "marc" && <MarcMappingsTab />}
    </div>
  );
}

// ============================================================================
// Schedules Tab
// ============================================================================

function SchedulesTab({ writable }: { writable: boolean }) {
  const qc = useQueryClient();
  const schemesQuery = useQuery({
    queryKey: ["classification-schemes"],
    queryFn: classificationApi.listSchemes,
  });

  const [selectedSchemeId, setSelectedSchemeId] = useState<number | null>(null);
  const [parentFilter, setParentFilter] = useState<string>("");
  const [searchQuery, setSearchQuery] = useState("");
  const [showAddEntryModal, setShowAddEntryModal] = useState(false);
  const [showAddSchemeModal, setShowAddSchemeModal] = useState(false);
  const [editingEntry, setEditingEntry] = useState<ClassificationEntry | null>(null);

  const activeSchemeId = selectedSchemeId || schemesQuery.data?.[0]?.id || 1;
  const currentScheme = schemesQuery.data?.find((s) => s.id === activeSchemeId);

  const entriesQuery = useQuery({
    queryKey: ["classification-entries", activeSchemeId, parentFilter, searchQuery],
    queryFn: () =>
      classificationApi.listEntries({
        schemeId: activeSchemeId,
        parentCode: parentFilter || null,
        searchQuery: searchQuery || null,
      }),
    enabled: !!activeSchemeId,
  });

  const deleteEntryMutation = useMutation({
    mutationFn: classificationApi.deleteEntry,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["classification-entries"] });
      qc.invalidateQueries({ queryKey: ["classification-schemes"] });
    },
  });

  return (
    <div className="space-y-4">
      {/* Scheme bar */}
      <div className="flex flex-wrap items-center justify-between gap-4 p-4 rounded-lg border border-line bg-surface">
        <div className="flex items-center gap-3">
          <label htmlFor="scheme-select" className="text-sm font-medium whitespace-nowrap">
            Scheme:
          </label>
          <select
            id="scheme-select"
            value={activeSchemeId}
            onChange={(e) => {
              setSelectedSchemeId(Number(e.target.value));
              setParentFilter("");
              setSearchQuery("");
            }}
            className="rounded border border-line bg-bg px-3 py-1.5 text-sm"
          >
            {schemesQuery.data?.map((s) => (
              <option key={s.id} value={s.id}>
                {s.code.toUpperCase()} — {s.name} ({s.recordCount} titles)
              </option>
            ))}
          </select>
          {currentScheme?.isCustom && (
            <Chip tone="neutral">Custom Scheme</Chip>
          )}
        </div>

        <div className="flex items-center gap-2">
          {writable && (
            <>
              <Button variant="secondary" onClick={() => setShowAddSchemeModal(true)}>
                + New Scheme
              </Button>
              <Button onClick={() => setShowAddEntryModal(true)}>
                + Add Code
              </Button>
            </>
          )}
        </div>
      </div>

      {currentScheme && (
        <div className="text-xs text-muted px-1 flex items-center justify-between">
          <span>{currentScheme.description || currentScheme.name}</span>
          <span>
            {currentScheme.entryCount} schedule entries · {currentScheme.recordCount} titles classified
          </span>
        </div>
      )}

      {/* Filter bar */}
      <div className="flex flex-wrap items-center gap-3">
        <div className="w-64">
          <input
            type="search"
            placeholder="Search code or label..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full rounded border border-line bg-surface px-3 py-1.5 text-sm"
          />
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted">Filter Parent:</span>
          <select
            value={parentFilter}
            onChange={(e) => setParentFilter(e.target.value)}
            className="rounded border border-line bg-surface px-2 py-1 text-sm"
          >
            <option value="">All Divisions</option>
            <option value="000">000 Generalities</option>
            <option value="100">100 Philosophy</option>
            <option value="200">200 Religion</option>
            <option value="300">300 Social sciences</option>
            <option value="400">400 Language</option>
            <option value="500">500 Science</option>
            <option value="600">600 Technology</option>
            <option value="700">700 Arts</option>
            <option value="800">800 Literature</option>
            <option value="900">900 History</option>
          </select>
        </div>
      </div>

      {/* Entries table */}
      <div className="overflow-x-auto rounded-lg border border-line bg-surface">
        <table className="w-full text-left text-sm">
          <thead className="border-b border-line bg-surface-hover text-xs font-medium text-muted">
            <tr>
              <th className="px-4 py-2.5">Code</th>
              <th className="px-4 py-2.5">Classification Title / Heading</th>
              <th className="px-4 py-2.5">Parent</th>
              <th className="px-4 py-2.5">Description</th>
              <th className="px-4 py-2.5">Titles</th>
              <th className="px-4 py-2.5 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-line">
            {entriesQuery.isLoading ? (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-muted">
                  Loading classification schedule...
                </td>
              </tr>
            ) : entriesQuery.data?.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-muted">
                  No classification entries matched your search.
                </td>
              </tr>
            ) : (
              entriesQuery.data?.map((entry) => (
                <tr key={entry.id} className="hover:bg-surface-hover">
                  <td className="px-4 py-2.5 font-mono font-medium">{entry.code}</td>
                  <td className="px-4 py-2.5 font-medium">
                    {entry.title}
                    {entry.isCustom && (
                      <span className="ml-2 text-xs text-primary font-normal">(custom)</span>
                    )}
                  </td>
                  <td className="px-4 py-2.5 font-mono text-xs text-muted">
                    {entry.parentCode || "—"}
                  </td>
                  <td className="px-4 py-2.5 text-xs text-muted max-w-xs truncate">
                    {entry.description || "—"}
                  </td>
                  <td className="px-4 py-2.5">
                    {entry.recordCount > 0 ? (
                      <Link
                        to={`/catalogue?query=${encodeURIComponent(entry.code)}`}
                        className="inline-flex items-center gap-1 text-xs text-primary hover:underline font-mono"
                      >
                        <Chip tone="fine">{entry.recordCount} titles</Chip>
                      </Link>
                    ) : (
                      <span className="text-xs text-muted">0</span>
                    )}
                  </td>
                  <td className="px-4 py-2.5 text-right">
                    {writable && (
                      <div className="flex items-center justify-end gap-2">
                        <Button
                          variant="ghost"
                          className="h-7 px-2 text-xs"
                          onClick={() => setEditingEntry(entry)}
                        >
                          Edit
                        </Button>
                        {entry.isCustom && (
                          <Button
                            variant="ghost"
                            className="h-7 px-2 text-xs text-danger"
                            onClick={() => {
                              if (window.confirm(`Delete classification code ${entry.code}?`)) {
                                deleteEntryMutation.mutate(entry.id);
                              }
                            }}
                          >
                            Delete
                          </Button>
                        )}
                      </div>
                    )}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {showAddEntryModal && (
        <EntryModal
          schemeId={activeSchemeId}
          onClose={() => setShowAddEntryModal(false)}
        />
      )}
      {editingEntry && (
        <EntryModal
          schemeId={activeSchemeId}
          initial={editingEntry}
          onClose={() => setEditingEntry(null)}
        />
      )}
      {showAddSchemeModal && (
        <SchemeModal onClose={() => setShowAddSchemeModal(false)} />
      )}
    </div>
  );
}

// ============================================================================
// Authors Tab
// ============================================================================

function AuthorsTab({ writable }: { writable: boolean }) {
  const [search, setSearch] = useState("");
  const [kindFilter, setKindFilter] = useState("");
  const [showAddModal, setShowAddModal] = useState(false);
  const [editingAuthor, setEditingAuthor] = useState<AuthorAuthority | null>(null);
  const [mergingAuthor, setMergingAuthor] = useState<AuthorAuthority | null>(null);

  const authorsQuery = useQuery({
    queryKey: ["classification-authors", search, kindFilter],
    queryFn: () => classificationApi.listAuthors({ searchQuery: search, kindFilter }),
  });

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <input
            type="search"
            placeholder="Search author name, variant, or notes..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-72 rounded border border-line bg-surface px-3 py-1.5 text-sm"
          />
          <select
            value={kindFilter}
            onChange={(e) => setKindFilter(e.target.value)}
            className="rounded border border-line bg-surface px-2 py-1.5 text-sm"
          >
            <option value="">All Kinds</option>
            <option value="personal">Personal Author</option>
            <option value="corporate">Corporate Body</option>
            <option value="meeting">Meeting / Conference</option>
            <option value="uniform_title">Uniform Title</option>
          </select>
        </div>

        {writable && (
          <Button onClick={() => setShowAddModal(true)}>+ Add Author Authority</Button>
        )}
      </div>

      <div className="overflow-x-auto rounded-lg border border-line bg-surface">
        <table className="w-full text-left text-sm">
          <thead className="border-b border-line bg-surface-hover text-xs font-medium text-muted">
            <tr>
              <th className="px-4 py-2.5">Heading / Name</th>
              <th className="px-4 py-2.5">Kind</th>
              <th className="px-4 py-2.5">Dates</th>
              <th className="px-4 py-2.5">MARC Tag</th>
              <th className="px-4 py-2.5">Authority Note / Variants</th>
              <th className="px-4 py-2.5">Linked Records</th>
              <th className="px-4 py-2.5 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-line">
            {authorsQuery.isLoading ? (
              <tr>
                <td colSpan={7} className="px-4 py-8 text-center text-muted">
                  Loading authors...
                </td>
              </tr>
            ) : authorsQuery.data?.length === 0 ? (
              <tr>
                <td colSpan={7} className="px-4 py-8 text-center text-muted">
                  No author authorities found.
                </td>
              </tr>
            ) : (
              authorsQuery.data?.map((author) => (
                <tr key={author.id} className="hover:bg-surface-hover">
                  <td className="px-4 py-2.5 font-medium">
                    <div>{author.name}</div>
                    {author.sortName && author.sortName !== author.name && (
                      <div className="text-xs text-muted">Sort: {author.sortName}</div>
                    )}
                  </td>
                  <td className="px-4 py-2.5 capitalize text-xs text-muted">
                    {author.kind || "personal"}
                  </td>
                  <td className="px-4 py-2.5 text-xs font-mono text-muted">
                    {author.birthYear || author.deathYear
                      ? `${author.birthYear ?? "?"}–${author.deathYear ?? ""}`
                      : "—"}
                  </td>
                  <td className="px-4 py-2.5 font-mono text-xs text-muted">
                    {author.marcTag}
                  </td>
                  <td className="px-4 py-2.5 text-xs text-muted max-w-xs truncate">
                    {author.authorityNote || author.variantNames || "—"}
                  </td>
                  <td className="px-4 py-2.5">
                    {author.recordCount > 0 ? (
                      <Link
                        to={`/catalogue?query=${encodeURIComponent(author.name)}`}
                        className="inline-flex items-center gap-1 text-xs text-primary hover:underline font-mono"
                      >
                        <Chip tone="fine">{author.recordCount} titles</Chip>
                      </Link>
                    ) : (
                      <span className="text-xs text-muted">0</span>
                    )}
                  </td>
                  <td className="px-4 py-2.5 text-right">
                    {writable && (
                      <div className="flex items-center justify-end gap-2">
                        <Button
                          variant="ghost"
                          className="h-7 px-2 text-xs"
                          onClick={() => setEditingAuthor(author)}
                        >
                          Edit
                        </Button>
                        <Button
                          variant="ghost"
                          className="h-7 px-2 text-xs"
                          onClick={() => setMergingAuthor(author)}
                        >
                          Merge...
                        </Button>
                      </div>
                    )}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {showAddModal && <AuthorModal onClose={() => setShowAddModal(false)} />}
      {editingAuthor && (
        <AuthorModal
          initial={editingAuthor}
          onClose={() => setEditingAuthor(null)}
        />
      )}
      {mergingAuthor && (
        <MergeAuthorModal
          source={mergingAuthor}
          onClose={() => setMergingAuthor(null)}
        />
      )}
    </div>
  );
}

// ============================================================================
// Publishers Tab
// ============================================================================

function PublishersTab({ writable }: { writable: boolean }) {
  const [search, setSearch] = useState("");
  const [showAddModal, setShowAddModal] = useState(false);
  const [editingPublisher, setEditingPublisher] = useState<PublisherAuthority | null>(null);
  const [mergingPublisher, setMergingPublisher] = useState<PublisherAuthority | null>(null);

  const publishersQuery = useQuery({
    queryKey: ["classification-publishers", search],
    queryFn: () => classificationApi.listPublishers({ searchQuery: search }),
  });

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <input
          type="search"
          placeholder="Search publisher name, place, notes..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-72 rounded border border-line bg-surface px-3 py-1.5 text-sm"
        />

        {writable && (
          <Button onClick={() => setShowAddModal(true)}>+ Add Publisher Authority</Button>
        )}
      </div>

      <div className="overflow-x-auto rounded-lg border border-line bg-surface">
        <table className="w-full text-left text-sm">
          <thead className="border-b border-line bg-surface-hover text-xs font-medium text-muted">
            <tr>
              <th className="px-4 py-2.5">Publisher Name</th>
              <th className="px-4 py-2.5">Place of Publication</th>
              <th className="px-4 py-2.5">MARC Tag</th>
              <th className="px-4 py-2.5">Notes</th>
              <th className="px-4 py-2.5">Linked Records</th>
              <th className="px-4 py-2.5 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-line">
            {publishersQuery.isLoading ? (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-muted">
                  Loading publishers...
                </td>
              </tr>
            ) : publishersQuery.data?.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-muted">
                  No publisher authorities found.
                </td>
              </tr>
            ) : (
              publishersQuery.data?.map((pub) => (
                <tr key={pub.id} className="hover:bg-surface-hover">
                  <td className="px-4 py-2.5 font-medium">{pub.name}</td>
                  <td className="px-4 py-2.5 text-xs text-muted">{pub.place || "—"}</td>
                  <td className="px-4 py-2.5 font-mono text-xs text-muted">{pub.marcTag}</td>
                  <td className="px-4 py-2.5 text-xs text-muted max-w-xs truncate">
                    {pub.notes || "—"}
                  </td>
                  <td className="px-4 py-2.5">
                    {pub.recordCount > 0 ? (
                      <Link
                        to={`/catalogue?query=${encodeURIComponent(pub.name)}`}
                        className="inline-flex items-center gap-1 text-xs text-primary hover:underline font-mono"
                      >
                        <Chip tone="fine">{pub.recordCount} titles</Chip>
                      </Link>
                    ) : (
                      <span className="text-xs text-muted">0</span>
                    )}
                  </td>
                  <td className="px-4 py-2.5 text-right">
                    {writable && (
                      <div className="flex items-center justify-end gap-2">
                        <Button
                          variant="ghost"
                          className="h-7 px-2 text-xs"
                          onClick={() => setEditingPublisher(pub)}
                        >
                          Edit
                        </Button>
                        <Button
                          variant="ghost"
                          className="h-7 px-2 text-xs"
                          onClick={() => setMergingPublisher(pub)}
                        >
                          Merge...
                        </Button>
                      </div>
                    )}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {showAddModal && <PublisherModal onClose={() => setShowAddModal(false)} />}
      {editingPublisher && (
        <PublisherModal
          initial={editingPublisher}
          onClose={() => setEditingPublisher(null)}
        />
      )}
      {mergingPublisher && (
        <MergePublisherModal
          source={mergingPublisher}
          onClose={() => setMergingPublisher(null)}
        />
      )}
    </div>
  );
}

// ============================================================================
// Subjects Tab
// ============================================================================

function SubjectsTab({ writable }: { writable: boolean }) {
  const [search, setSearch] = useState("");
  const [schemeFilter, setSchemeFilter] = useState("");
  const [showAddModal, setShowAddModal] = useState(false);
  const [editingSubject, setEditingSubject] = useState<SubjectAuthority | null>(null);
  const [mergingSubject, setMergingSubject] = useState<SubjectAuthority | null>(null);

  const subjectsQuery = useQuery({
    queryKey: ["classification-subjects", search, schemeFilter],
    queryFn: () => classificationApi.listSubjects({ searchQuery: search, schemeFilter }),
  });

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <input
            type="search"
            placeholder="Search subject headings, scope notes..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-72 rounded border border-line bg-surface px-3 py-1.5 text-sm"
          />
          <select
            value={schemeFilter}
            onChange={(e) => setSchemeFilter(e.target.value)}
            className="rounded border border-line bg-surface px-2 py-1.5 text-sm"
          >
            <option value="">All Schemes</option>
            <option value="local">Local Library</option>
            <option value="lcsh">LCSH (Library of Congress)</option>
            <option value="fast">FAST Headings</option>
            <option value="mesh">MeSH (Medical)</option>
          </select>
        </div>

        {writable && (
          <Button onClick={() => setShowAddModal(true)}>+ Add Subject Authority</Button>
        )}
      </div>

      <div className="overflow-x-auto rounded-lg border border-line bg-surface">
        <table className="w-full text-left text-sm">
          <thead className="border-b border-line bg-surface-hover text-xs font-medium text-muted">
            <tr>
              <th className="px-4 py-2.5">Subject Heading</th>
              <th className="px-4 py-2.5">Scheme</th>
              <th className="px-4 py-2.5">MARC Tag</th>
              <th className="px-4 py-2.5">Broader / Narrower Terms</th>
              <th className="px-4 py-2.5">See Also</th>
              <th className="px-4 py-2.5">Linked Records</th>
              <th className="px-4 py-2.5 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-line">
            {subjectsQuery.isLoading ? (
              <tr>
                <td colSpan={7} className="px-4 py-8 text-center text-muted">
                  Loading subjects...
                </td>
              </tr>
            ) : subjectsQuery.data?.length === 0 ? (
              <tr>
                <td colSpan={7} className="px-4 py-8 text-center text-muted">
                  No subject authorities found.
                </td>
              </tr>
            ) : (
              subjectsQuery.data?.map((subject) => (
                <tr key={subject.id} className="hover:bg-surface-hover">
                  <td className="px-4 py-2.5 font-medium">{subject.heading}</td>
                  <td className="px-4 py-2.5 uppercase text-xs text-muted">{subject.scheme}</td>
                  <td className="px-4 py-2.5 font-mono text-xs text-muted">{subject.marcTag}</td>
                  <td className="px-4 py-2.5 text-xs text-muted max-w-xs truncate">
                    {subject.broaderTerm || subject.narrowerTerms || "—"}
                  </td>
                  <td className="px-4 py-2.5 text-xs text-muted max-w-xs truncate">
                    {subject.seeAlso || "—"}
                  </td>
                  <td className="px-4 py-2.5">
                    {subject.recordCount > 0 ? (
                      <Link
                        to={`/catalogue?query=${encodeURIComponent(subject.heading)}`}
                        className="inline-flex items-center gap-1 text-xs text-primary hover:underline font-mono"
                      >
                        <Chip tone="fine">{subject.recordCount} titles</Chip>
                      </Link>
                    ) : (
                      <span className="text-xs text-muted">0</span>
                    )}
                  </td>
                  <td className="px-4 py-2.5 text-right">
                    {writable && (
                      <div className="flex items-center justify-end gap-2">
                        <Button
                          variant="ghost"
                          className="h-7 px-2 text-xs"
                          onClick={() => setEditingSubject(subject)}
                        >
                          Edit
                        </Button>
                        <Button
                          variant="ghost"
                          className="h-7 px-2 text-xs"
                          onClick={() => setMergingSubject(subject)}
                        >
                          Merge...
                        </Button>
                      </div>
                    )}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {showAddModal && <SubjectModal onClose={() => setShowAddModal(false)} />}
      {editingSubject && (
        <SubjectModal
          initial={editingSubject}
          onClose={() => setEditingSubject(null)}
        />
      )}
      {mergingSubject && (
        <MergeSubjectModal
          source={mergingSubject}
          onClose={() => setMergingSubject(null)}
        />
      )}
    </div>
  );
}

// ============================================================================
// MARC Mappings Tab & Tester Tool
// ============================================================================

function MarcMappingsTab() {
  const mappingsQuery = useQuery({
    queryKey: ["classification-marc-mappings"],
    queryFn: classificationApi.listMarcMappings,
  });

  const [testLine, setTestLine] = useState("100 1# $a Basheer, Vaikom Muhammad, $d 1908-1994");
  const [testResult, setTestResult] = useState<MarcTestResult | null>(null);
  const [testError, setTestError] = useState<string | null>(null);

  const testMutation = useMutation({
    mutationFn: classificationApi.testMarcMapping,
    onSuccess: (res) => {
      setTestResult(res);
      setTestError(null);
    },
    onError: (err) => {
      setTestError(toApiError(err).message);
      setTestResult(null);
    },
  });

  const handleTest = (line: string) => {
    setTestLine(line);
    testMutation.mutate({ rawMarcLine: line });
  };

  return (
    <div className="space-y-6">
      {/* Live MARC Parser & Tester */}
      <Panel>
        <PanelHeader
          title="Interactive MARC Authority & Classification Tester"
          aside="Test parsing and mapping resolution for sample MARC authority lines"
        />
        <div className="p-4 space-y-4">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-xs text-muted font-medium">Examples:</span>
            <Button
              variant="secondary"
              className="h-6 text-xs px-2"
              onClick={() => handleTest("100 1# $a Basheer, Vaikom Muhammad, $d 1908-1994")}
            >
              Author (100)
            </Button>
            <Button
              variant="secondary"
              className="h-6 text-xs px-2"
              onClick={() => handleTest("264 #1 $a Kottayam : $b DC Books, $c 1944")}
            >
              Publisher (264)
            </Button>
            <Button
              variant="secondary"
              className="h-6 text-xs px-2"
              onClick={() => handleTest("650 #0 $a Malayalam fiction $x History.")}
            >
              Subject (650)
            </Button>
            <Button
              variant="secondary"
              className="h-6 text-xs px-2"
              onClick={() => handleTest("082 04 $a 894.8123")}
            >
              Dewey DDC (082)
            </Button>
            <Button
              variant="secondary"
              className="h-6 text-xs px-2"
              onClick={() => handleTest("050 00 $a PK2798.B3 $b B35 1944")}
            >
              LC Class (050)
            </Button>
          </div>

          <form
            onSubmit={(e) => {
              e.preventDefault();
              handleTest(testLine);
            }}
            className="flex gap-2"
          >
            <input
              type="text"
              value={testLine}
              onChange={(e) => setTestLine(e.target.value)}
              placeholder="Enter a raw MARC tag line (e.g. 100 1# $a Name $d Dates)"
              className="flex-1 font-mono text-sm rounded border border-line bg-surface px-3 py-1.5"
            />
            <Button type="submit" disabled={testMutation.isPending}>
              {testMutation.isPending ? "Testing..." : "Test Mapping"}
            </Button>
          </form>

          {testError && (
            <p role="alert" className="text-sm text-danger font-medium">
              {testError}
            </p>
          )}

          {testResult && (
            <div className="p-3 rounded-lg border border-line bg-surface-hover text-sm space-y-2">
              <div className="flex flex-wrap items-center gap-4 text-xs font-mono">
                <div>
                  <span className="text-muted">Tag: </span>
                  <span className="font-bold text-fg">{testResult.tag}</span>
                </div>
                <div>
                  <span className="text-muted">Indicators: </span>
                  <span className="font-bold text-fg">{testResult.indicators}</span>
                </div>
                <div>
                  <span className="text-muted">Resolved Authority Type: </span>
                  <Chip tone="fine">{testResult.mappedAuthorityType || "unmapped"}</Chip>
                </div>
              </div>

              <div>
                <span className="text-xs text-muted block mb-1">Extracted Bibliographic Fields:</span>
                <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
                  {Object.entries(testResult.targetFields).map(([key, val]) => (
                    <div key={key} className="p-2 rounded bg-surface border border-line text-xs">
                      <span className="text-muted block font-mono">{key}:</span>
                      <span className="font-semibold">{val}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>
      </Panel>

      {/* Controlled Mappings Registry Table */}
      <Panel>
        <PanelHeader
          title="Controlled MARC Tag & Subfield Mappings"
          aside={`${mappingsQuery.data?.length || 0} active mappings`}
        />
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead className="border-b border-line bg-surface-hover text-xs font-medium text-muted">
              <tr>
                <th className="px-4 py-2.5">MARC Tag</th>
                <th className="px-4 py-2.5">Subfield</th>
                <th className="px-4 py-2.5">Authority / Classification Type</th>
                <th className="px-4 py-2.5">Target Field</th>
                <th className="px-4 py-2.5">Description</th>
                <th className="px-4 py-2.5 text-right">Standard</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-line font-mono text-xs">
              {mappingsQuery.data?.map((m) => (
                <tr key={m.id} className="hover:bg-surface-hover font-normal">
                  <td className="px-4 py-2 font-bold text-fg">{m.marcTag}</td>
                  <td className="px-4 py-2">${m.subfieldCode}</td>
                  <td className="px-4 py-2 text-primary font-sans">{m.authorityType}</td>
                  <td className="px-4 py-2 text-fg">{m.targetField}</td>
                  <td className="px-4 py-2 text-muted font-sans text-xs">{m.description}</td>
                  <td className="px-4 py-2 text-right">
                    {m.isDefault ? (
                      <Chip tone="neutral">Standard</Chip>
                    ) : (
                      <Chip tone="fine">Custom</Chip>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Panel>
    </div>
  );
}

// ============================================================================
// Modals
// ============================================================================

function EntryModal({
  schemeId,
  initial,
  onClose,
}: {
  schemeId: number;
  initial?: ClassificationEntry;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [code, setCode] = useState(initial?.code || "");
  const [parentCode, setParentCode] = useState(initial?.parentCode || "");
  const [title, setTitle] = useState(initial?.title || "");
  const [description, setDescription] = useState(initial?.description || "");
  const [error, setError] = useState<string | null>(null);

  const saveMutation = useMutation({
    mutationFn: classificationApi.saveEntry,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["classification-entries"] });
      qc.invalidateQueries({ queryKey: ["classification-schemes"] });
      onClose();
    },
    onError: (err) => setError(toApiError(err).message),
  });

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    saveMutation.mutate({
      id: initial?.id ?? null,
      schemeId,
      code: code.trim(),
      parentCode: parentCode.trim() || null,
      title: title.trim(),
      description: description.trim() || null,
    });
  };

  return (
    <Modal
      title={initial ? `Edit Classification Code ${initial.code}` : "Add Classification Code"}
      onClose={onClose}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {error && (
          <p role="alert" className="text-sm text-danger font-medium">
            {error}
          </p>
        )}
        <TextField
          label="Classification Code"
          value={code}
          onChange={(e) => setCode(e.target.value)}
          placeholder="e.g. 510 or QA or B13"
          required
        />
        <TextField
          label="Parent Code (Optional)"
          value={parentCode}
          onChange={(e) => setParentCode(e.target.value)}
          placeholder="e.g. 500 (leave blank for top-level)"
        />
        <TextField
          label="Title / Heading"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="e.g. Mathematics"
          required
        />
        <TextField
          label="Description / Scope Note"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Detailed scope or explanatory note"
        />
        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" type="button" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" disabled={saveMutation.isPending}>
            {saveMutation.isPending ? "Saving..." : "Save Code"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function SchemeModal({ onClose }: { onClose: () => void }) {
  const qc = useQueryClient();
  const [code, setCode] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [error, setError] = useState<string | null>(null);

  const saveMutation = useMutation({
    mutationFn: classificationApi.saveScheme,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["classification-schemes"] });
      onClose();
    },
    onError: (err) => setError(toApiError(err).message),
  });

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    saveMutation.mutate({
      id: null,
      code: code.trim(),
      name: name.trim(),
      description: description.trim() || null,
    });
  };

  return (
    <Modal title="Create Custom Classification Scheme" onClose={onClose}>
      <form onSubmit={handleSubmit} className="space-y-4">
        {error && (
          <p role="alert" className="text-sm text-danger font-medium">
            {error}
          </p>
        )}
        <TextField
          label="Scheme Code (lowercase, max 16 chars)"
          value={code}
          onChange={(e) => setCode(e.target.value)}
          placeholder="e.g. school or special"
          required
        />
        <TextField
          label="Scheme Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. School Curriculum Scheme"
          required
        />
        <TextField
          label="Description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Description or notes about this classification system"
        />
        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" type="button" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" disabled={saveMutation.isPending}>
            {saveMutation.isPending ? "Creating..." : "Create Scheme"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function AuthorModal({
  initial,
  onClose,
}: {
  initial?: AuthorAuthority;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [name, setName] = useState(initial?.name || "");
  const [sortName, setSortName] = useState(initial?.sortName || "");
  const [kind, setKind] = useState(initial?.kind || "personal");
  const [birthYear, setBirthYear] = useState(initial?.birthYear ? String(initial.birthYear) : "");
  const [deathYear, setDeathYear] = useState(initial?.deathYear ? String(initial.deathYear) : "");
  const [authorityNote, setAuthorityNote] = useState(initial?.authorityNote || "");
  const [variantNames, setVariantNames] = useState(initial?.variantNames || "");
  const [marcTag, setMarcTag] = useState(initial?.marcTag || "100");
  const [error, setError] = useState<string | null>(null);

  const saveMutation = useMutation({
    mutationFn: classificationApi.saveAuthor,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["classification-authors"] });
      onClose();
    },
    onError: (err) => setError(toApiError(err).message),
  });

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    saveMutation.mutate({
      id: initial?.id ?? null,
      name: name.trim(),
      sortName: sortName.trim() || null,
      kind,
      birthYear: birthYear ? Number(birthYear) : null,
      deathYear: deathYear ? Number(deathYear) : null,
      authorityNote: authorityNote.trim() || null,
      variantNames: variantNames.trim() || null,
      marcTag,
    });
  };

  return (
    <Modal
      title={initial ? `Edit Author Authority: ${initial.name}` : "Add Author Authority"}
      onClose={onClose}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {error && (
          <p role="alert" className="text-sm text-danger font-medium">
            {error}
          </p>
        )}
        <TextField
          label="Author Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. Basheer, Vaikom Muhammad"
          required
        />
        <TextField
          label="Sort Name"
          value={sortName}
          onChange={(e) => setSortName(e.target.value)}
          placeholder="e.g. Basheer, Vaikom Muhammad"
        />
        <div className="grid grid-cols-2 gap-3">
          <SelectField
            label="Kind"
            value={kind}
            onChange={(e) => setKind(e.target.value)}
          >
            <option value="personal">Personal Name (100)</option>
            <option value="corporate">Corporate Body (110)</option>
            <option value="meeting">Meeting / Conference (111)</option>
            <option value="uniform_title">Uniform Title (130)</option>
          </SelectField>
          <TextField
            label="MARC Tag"
            value={marcTag}
            onChange={(e) => setMarcTag(e.target.value)}
            placeholder="100, 110, 111, 700"
          />
        </div>
        <div className="grid grid-cols-2 gap-3">
          <TextField
            label="Birth Year"
            value={birthYear}
            onChange={(e) => setBirthYear(e.target.value)}
            placeholder="e.g. 1908"
          />
          <TextField
            label="Death Year"
            value={deathYear}
            onChange={(e) => setDeathYear(e.target.value)}
            placeholder="e.g. 1994"
          />
        </div>
        <TextField
          label="Variant Names / Pseudonyms"
          value={variantNames}
          onChange={(e) => setVariantNames(e.target.value)}
          placeholder="e.g. Beypore Sultan; V. M. Basheer"
        />
        <TextField
          label="Authority Note / Biographical Summary"
          value={authorityNote}
          onChange={(e) => setAuthorityNote(e.target.value)}
          placeholder="Biographical or authority scope note"
        />
        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" type="button" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" disabled={saveMutation.isPending}>
            {saveMutation.isPending ? "Saving..." : "Save Authority"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function MergeAuthorModal({
  source,
  onClose,
}: {
  source: AuthorAuthority;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [targetId, setTargetId] = useState<number | "">("");
  const [error, setError] = useState<string | null>(null);

  const authorsQuery = useQuery({
    queryKey: ["classification-authors", ""],
    queryFn: () => classificationApi.listAuthors({}),
  });

  const availableTargets =
    authorsQuery.data?.filter((a) => a.id !== source.id) || [];

  const mergeMutation = useMutation({
    mutationFn: ({ fromId, toId }: { fromId: number; toId: number }) =>
      classificationApi.mergeAuthors(fromId, toId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["classification-authors"] });
      onClose();
    },
    onError: (err) => setError(toApiError(err).message),
  });

  const handleMerge = (e: FormEvent) => {
    e.preventDefault();
    if (!targetId) return;
    setError(null);
    mergeMutation.mutate({ fromId: source.id, toId: Number(targetId) });
  };

  return (
    <Modal title={`Merge Author Authority: ${source.name}`} onClose={onClose}>
      <form onSubmit={handleMerge} className="space-y-4">
        {error && (
          <p role="alert" className="text-sm text-danger font-medium">
            {error}
          </p>
        )}
        <p className="text-sm text-muted">
          Merging <strong>{source.name}</strong> will re-link all {source.recordCount} catalogue
          records to the target author authority and delete the source heading.
        </p>

        <div>
          <label htmlFor="target-author-select" className="text-sm font-medium block mb-1">Target Author Authority:</label>
          <select
            id="target-author-select"
            value={targetId}
            onChange={(e) => setTargetId(Number(e.target.value))}
            className="w-full rounded border border-line bg-bg px-3 py-1.5 text-sm"
            required
          >
            <option value="">Select an author to merge into...</option>
            {availableTargets.map((a) => (
              <option key={a.id} value={a.id}>
                {a.name} ({a.recordCount} titles)
              </option>
            ))}
          </select>
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" type="button" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="submit"
            disabled={!targetId || mergeMutation.isPending}
            className="bg-danger text-danger-fg hover:bg-danger-hover"
          >
            {mergeMutation.isPending ? "Merging..." : "Confirm Merge"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function PublisherModal({
  initial,
  onClose,
}: {
  initial?: PublisherAuthority;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [name, setName] = useState(initial?.name || "");
  const [place, setPlace] = useState(initial?.place || "");
  const [notes, setNotes] = useState(initial?.notes || "");
  const [marcTag, setMarcTag] = useState(initial?.marcTag || "264");
  const [error, setError] = useState<string | null>(null);

  const saveMutation = useMutation({
    mutationFn: classificationApi.savePublisher,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["classification-publishers"] });
      onClose();
    },
    onError: (err) => setError(toApiError(err).message),
  });

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    saveMutation.mutate({
      id: initial?.id ?? null,
      name: name.trim(),
      place: place.trim() || null,
      notes: notes.trim() || null,
      marcTag,
    });
  };

  return (
    <Modal
      title={initial ? `Edit Publisher Authority: ${initial.name}` : "Add Publisher Authority"}
      onClose={onClose}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {error && (
          <p role="alert" className="text-sm text-danger font-medium">
            {error}
          </p>
        )}
        <TextField
          label="Publisher Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. DC Books"
          required
        />
        <TextField
          label="Place of Publication"
          value={place}
          onChange={(e) => setPlace(e.target.value)}
          placeholder="e.g. Kottayam, Kerala"
        />
        <TextField
          label="MARC Tag"
          value={marcTag}
          onChange={(e) => setMarcTag(e.target.value)}
          placeholder="264 (RDA) or 260 (legacy)"
        />
        <TextField
          label="Notes"
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          placeholder="Imprint notes, history, or imprint codes"
        />
        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" type="button" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" disabled={saveMutation.isPending}>
            {saveMutation.isPending ? "Saving..." : "Save Publisher"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function MergePublisherModal({
  source,
  onClose,
}: {
  source: PublisherAuthority;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [targetId, setTargetId] = useState<number | "">("");
  const [error, setError] = useState<string | null>(null);

  const publishersQuery = useQuery({
    queryKey: ["classification-publishers", ""],
    queryFn: () => classificationApi.listPublishers({}),
  });

  const availableTargets =
    publishersQuery.data?.filter((p) => p.id !== source.id) || [];

  const mergeMutation = useMutation({
    mutationFn: ({ fromId, toId }: { fromId: number; toId: number }) =>
      classificationApi.mergePublishers(fromId, toId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["classification-publishers"] });
      onClose();
    },
    onError: (err) => setError(toApiError(err).message),
  });

  const handleMerge = (e: FormEvent) => {
    e.preventDefault();
    if (!targetId) return;
    setError(null);
    mergeMutation.mutate({ fromId: source.id, toId: Number(targetId) });
  };

  return (
    <Modal title={`Merge Publisher Authority: ${source.name}`} onClose={onClose}>
      <form onSubmit={handleMerge} className="space-y-4">
        {error && (
          <p role="alert" className="text-sm text-danger font-medium">
            {error}
          </p>
        )}
        <p className="text-sm text-muted">
          Merging <strong>{source.name}</strong> will update all {source.recordCount} catalogue
          records to point to the target publisher and delete the source authority.
        </p>

        <div>
          <label htmlFor="target-publisher-select" className="text-sm font-medium block mb-1">Target Publisher Authority:</label>
          <select
            id="target-publisher-select"
            value={targetId}
            onChange={(e) => setTargetId(Number(e.target.value))}
            className="w-full rounded border border-line bg-bg px-3 py-1.5 text-sm"
            required
          >
            <option value="">Select a publisher to merge into...</option>
            {availableTargets.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name} ({p.recordCount} titles)
              </option>
            ))}
          </select>
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" type="button" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="submit"
            disabled={!targetId || mergeMutation.isPending}
            className="bg-danger text-danger-fg hover:bg-danger-hover"
          >
            {mergeMutation.isPending ? "Merging..." : "Confirm Merge"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function SubjectModal({
  initial,
  onClose,
}: {
  initial?: SubjectAuthority;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [heading, setHeading] = useState(initial?.heading || "");
  const [scheme, setScheme] = useState(initial?.scheme || "local");
  const [scopeNote, setScopeNote] = useState(initial?.scopeNote || "");
  const [broaderTerm, setBroaderTerm] = useState(initial?.broaderTerm || "");
  const [narrowerTerms, setNarrowerTerms] = useState(initial?.narrowerTerms || "");
  const [seeAlso, setSeeAlso] = useState(initial?.seeAlso || "");
  const [marcTag, setMarcTag] = useState(initial?.marcTag || "650");
  const [error, setError] = useState<string | null>(null);

  const saveMutation = useMutation({
    mutationFn: classificationApi.saveSubject,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["classification-subjects"] });
      onClose();
    },
    onError: (err) => setError(toApiError(err).message),
  });

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    saveMutation.mutate({
      id: initial?.id ?? null,
      heading: heading.trim(),
      scheme,
      scopeNote: scopeNote.trim() || null,
      broaderTerm: broaderTerm.trim() || null,
      narrowerTerms: narrowerTerms.trim() || null,
      seeAlso: seeAlso.trim() || null,
      marcTag,
    });
  };

  return (
    <Modal
      title={initial ? `Edit Subject Authority: ${initial.heading}` : "Add Subject Authority"}
      onClose={onClose}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {error && (
          <p role="alert" className="text-sm text-danger font-medium">
            {error}
          </p>
        )}
        <TextField
          label="Subject Heading"
          value={heading}
          onChange={(e) => setHeading(e.target.value)}
          placeholder="e.g. Malayalam fiction"
          required
        />
        <div className="grid grid-cols-2 gap-3">
          <SelectField
            label="Controlled Scheme"
            value={scheme}
            onChange={(e) => setScheme(e.target.value)}
          >
            <option value="local">Local Library Scheme</option>
            <option value="lcsh">Library of Congress (LCSH)</option>
            <option value="fast">FAST Headings</option>
            <option value="mesh">Medical (MeSH)</option>
          </SelectField>
          <TextField
            label="MARC Tag"
            value={marcTag}
            onChange={(e) => setMarcTag(e.target.value)}
            placeholder="650 (Topical), 651 (Geographic)"
          />
        </div>
        <TextField
          label="Broader Term"
          value={broaderTerm}
          onChange={(e) => setBroaderTerm(e.target.value)}
          placeholder="e.g. Dravidian literature"
        />
        <TextField
          label="Narrower Terms"
          value={narrowerTerms}
          onChange={(e) => setNarrowerTerms(e.target.value)}
          placeholder="e.g. Malayalam short stories; Malayalam novels"
        />
        <TextField
          label="See Also (Related / Synonyms)"
          value={seeAlso}
          onChange={(e) => setSeeAlso(e.target.value)}
          placeholder="e.g. Malayalam literature"
        />
        <TextField
          label="Scope Note"
          value={scopeNote}
          onChange={(e) => setScopeNote(e.target.value)}
          placeholder="Scope note, usage guidance, or definitions"
        />
        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" type="button" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" disabled={saveMutation.isPending}>
            {saveMutation.isPending ? "Saving..." : "Save Subject"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

function MergeSubjectModal({
  source,
  onClose,
}: {
  source: SubjectAuthority;
  onClose: () => void;
}) {
  const qc = useQueryClient();
  const [targetId, setTargetId] = useState<number | "">("");
  const [error, setError] = useState<string | null>(null);

  const subjectsQuery = useQuery({
    queryKey: ["classification-subjects", ""],
    queryFn: () => classificationApi.listSubjects({}),
  });

  const availableTargets =
    subjectsQuery.data?.filter((s) => s.id !== source.id) || [];

  const mergeMutation = useMutation({
    mutationFn: ({ fromId, toId }: { fromId: number; toId: number }) =>
      classificationApi.mergeSubjects(fromId, toId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["classification-subjects"] });
      onClose();
    },
    onError: (err) => setError(toApiError(err).message),
  });

  const handleMerge = (e: FormEvent) => {
    e.preventDefault();
    if (!targetId) return;
    setError(null);
    mergeMutation.mutate({ fromId: source.id, toId: Number(targetId) });
  };

  return (
    <Modal title={`Merge Subject Heading: ${source.heading}`} onClose={onClose}>
      <form onSubmit={handleMerge} className="space-y-4">
        {error && (
          <p role="alert" className="text-sm text-danger font-medium">
            {error}
          </p>
        )}
        <p className="text-sm text-muted">
          Merging <strong>{source.heading}</strong> will point all {source.recordCount} catalogue
          records to the target subject heading and delete the source authority.
        </p>

        <div>
          <label htmlFor="target-subject-select" className="text-sm font-medium block mb-1">Target Subject Heading:</label>
          <select
            id="target-subject-select"
            value={targetId}
            onChange={(e) => setTargetId(Number(e.target.value))}
            className="w-full rounded border border-line bg-bg px-3 py-1.5 text-sm"
            required
          >
            <option value="">Select a subject heading to merge into...</option>
            {availableTargets.map((s) => (
              <option key={s.id} value={s.id}>
                {s.heading} ({s.recordCount} titles)
              </option>
            ))}
          </select>
        </div>

        <div className="flex justify-end gap-2 pt-2">
          <Button variant="secondary" type="button" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="submit"
            disabled={!targetId || mergeMutation.isPending}
            className="bg-danger text-danger-fg hover:bg-danger-hover"
          >
            {mergeMutation.isPending ? "Merging..." : "Confirm Merge"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
