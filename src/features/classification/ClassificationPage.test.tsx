import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { MemoryRouter } from "react-router-dom";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import ClassificationPage from "./ClassificationPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(invoke).mockReset();
});

function renderWithClient(permissions = ["classification.read", "classification.write"]) {
  return render(
    <MemoryRouter>
      <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
        <SessionContext.Provider
          value={{
            session: {
              userId: 1,
              username: "admin",
              displayName: "Administrator",
              role: "administrator",
              permissions,
            },
            signOut: async () => {},
          }}
        >
          <ClassificationPage />
        </SessionContext.Provider>
      </QueryClientProvider>
    </MemoryRouter>
  );
}

it("renders classification schedules and allows code creation", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockSchemes = [
    {
      id: 1,
      code: "ddc",
      name: "Dewey Decimal Classification",
      description: "Standard Dewey scheme",
      isCustom: false,
      entryCount: 45,
      recordCount: 120,
    },
    {
      id: 2,
      code: "udc",
      name: "Universal Decimal Classification",
      description: "UDC index",
      isCustom: false,
      entryCount: 10,
      recordCount: 15,
    },
  ];

  const mockEntries = [
    {
      id: 101,
      schemeId: 1,
      schemeCode: "ddc",
      code: "500",
      parentCode: null,
      title: "Science",
      description: "Pure natural sciences",
      isCustom: false,
      recordCount: 42,
    },
    {
      id: 102,
      schemeId: 1,
      schemeCode: "ddc",
      code: "510",
      parentCode: "500",
      title: "Mathematics",
      description: "Algebra, calculus and statistics",
      isCustom: false,
      recordCount: 18,
    },
  ];

  vi.mocked(invoke).mockImplementation(async (cmd, args) => {
    if (cmd === "classification_list_schemes") return mockSchemes;
    if (cmd === "classification_list_entries") return mockEntries;
    if (cmd === "classification_save_entry") return 103;
    throw new Error(`Unhandled command: ${cmd} with ${JSON.stringify(args)}`);
  });

  renderWithClient();

  // Verify page header
  expect(await screen.findByRole("heading", { name: /Classification & Authorities/i })).toBeInTheDocument();

  // Verify schedule entries loaded
  expect(await screen.findByText("Science")).toBeInTheDocument();
  expect(screen.getByText("Mathematics")).toBeInTheDocument();
  expect(screen.getByText("42 titles")).toBeInTheDocument();

  // Open add code modal
  await user.click(screen.getByRole("button", { name: /\+ Add Code/i }));
  expect(screen.getByRole("heading", { name: /Add Classification Code/i })).toBeInTheDocument();

  await user.type(screen.getByLabelText("Classification Code"), "520");
  await user.type(screen.getByLabelText("Parent Code (Optional)"), "500");
  await user.type(screen.getByLabelText("Title / Heading"), "Astronomy");
  await user.click(screen.getByRole("button", { name: /Save Code/i }));

  expect(invoke).toHaveBeenCalledWith("classification_save_entry", {
    input: {
      id: null,
      schemeId: 1,
      code: "520",
      parentCode: "500",
      title: "Astronomy",
      description: null,
    },
  });
});

it("manages author authorities and supports add modal", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockAuthors = [
    {
      id: 1,
      name: "Basheer, Vaikom Muhammad",
      sortName: "Basheer, Vaikom Muhammad",
      kind: "personal",
      birthYear: 1908,
      deathYear: 1994,
      authorityNote: "Celebrated Malayalam writer",
      variantNames: "Beypore Sultan",
      marcTag: "100",
      recordCount: 8,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: null,
    },
  ];

  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "classification_list_schemes") return [];
    if (cmd === "classification_list_entries") return [];
    if (cmd === "classification_list_authors") return mockAuthors;
    if (cmd === "classification_save_author") return 2;
    return null;
  });

  renderWithClient();

  // Switch to Authors tab
  await user.click(screen.getByRole("tab", { name: /Author Authorities/i }));

  expect(await screen.findByText("Basheer, Vaikom Muhammad")).toBeInTheDocument();
  expect(screen.getByText("1908–1994")).toBeInTheDocument();
  expect(screen.getByText("8 titles")).toBeInTheDocument();

  // Open Add Author modal
  await user.click(screen.getByRole("button", { name: /\+ Add Author Authority/i }));
  expect(screen.getByRole("heading", { name: /Add Author Authority/i })).toBeInTheDocument();

  await user.type(screen.getByLabelText(/Author Name/i), "Thakazhi Sivasankara Pillai");
  await user.type(screen.getByLabelText(/Birth Year/i), "1912");
  await user.type(screen.getByLabelText(/Death Year/i), "1999");
  await user.click(screen.getByRole("button", { name: /Save Authority/i }));

  expect(invoke).toHaveBeenCalledWith("classification_save_author", {
    input: {
      id: null,
      name: "Thakazhi Sivasankara Pillai",
      sortName: null,
      kind: "personal",
      birthYear: 1912,
      deathYear: 1999,
      authorityNote: null,
      variantNames: null,
      marcTag: "100",
    },
  });
});

it("manages publisher authorities and subject authorities", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockPublishers = [
    {
      id: 1,
      name: "DC Books",
      place: "Kottayam, Kerala",
      notes: "Main imprint",
      marcTag: "264",
      recordCount: 15,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: null,
    },
  ];

  const mockSubjects = [
    {
      id: 1,
      heading: "Malayalam fiction",
      scheme: "lcsh",
      scopeNote: "Fiction written in Malayalam",
      broaderTerm: "Indic literature",
      narrowerTerms: "Short stories",
      seeAlso: "Malayalam literature",
      marcTag: "650",
      recordCount: 12,
      updatedAt: null,
    },
  ];

  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "classification_list_schemes") return [];
    if (cmd === "classification_list_entries") return [];
    if (cmd === "classification_list_publishers") return mockPublishers;
    if (cmd === "classification_list_subjects") return mockSubjects;
    if (cmd === "classification_save_publisher") return 2;
    if (cmd === "classification_save_subject") return 2;
    return null;
  });

  renderWithClient();

  // Test Publishers tab
  await user.click(screen.getByRole("tab", { name: /Publisher Authorities/i }));
  expect(await screen.findByText("DC Books")).toBeInTheDocument();
  expect(screen.getByText("Kottayam, Kerala")).toBeInTheDocument();
  expect(screen.getByText("15 titles")).toBeInTheDocument();

  // Test Subjects tab
  await user.click(screen.getByRole("tab", { name: /Subject Authorities/i }));
  expect(await screen.findByText("Malayalam fiction")).toBeInTheDocument();
  expect(screen.getByText("lcsh")).toBeInTheDocument();
  expect(screen.getByText("12 titles")).toBeInTheDocument();
});

it("inspects controlled MARC mappings and tests live MARC parsing", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockMappings = [
    {
      id: 1,
      authorityType: "author_personal",
      marcTag: "100",
      subfieldCode: "a",
      description: "Personal author heading",
      targetField: "name",
      isDefault: true,
    },
    {
      id: 2,
      authorityType: "classification_ddc",
      marcTag: "082",
      subfieldCode: "a",
      description: "DDC classification",
      targetField: "classification_number",
      isDefault: true,
    },
  ];

  const mockTestResult = {
    tag: "100",
    indicators: "1#",
    subfields: {
      a: "Basheer, Vaikom Muhammad",
      d: "1908-1994",
    },
    matchedMappings: [mockMappings[0]],
    mappedAuthorityType: "author_personal",
    targetFields: {
      name: "Basheer, Vaikom Muhammad",
    },
  };

  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "classification_list_schemes") return [];
    if (cmd === "classification_list_entries") return [];
    if (cmd === "classification_list_marc_mappings") return mockMappings;
    if (cmd === "classification_test_marc_mapping") return mockTestResult;
    return null;
  });

  renderWithClient();

  // Switch to MARC tab
  await user.click(screen.getByRole("tab", { name: /Controlled MARC Mappings/i }));

  expect(await screen.findByText("Personal author heading")).toBeInTheDocument();
  expect(screen.getByText("DDC classification")).toBeInTheDocument();

  // Click test mapping button
  await user.click(screen.getByRole("button", { name: /Test Mapping/i }));

  expect(await screen.findByText(/Resolved Authority Type:/i)).toBeInTheDocument();
  expect(screen.getAllByText("author_personal").length).toBeGreaterThanOrEqual(1);
  expect(screen.getByText("Basheer, Vaikom Muhammad")).toBeInTheDocument();
});
