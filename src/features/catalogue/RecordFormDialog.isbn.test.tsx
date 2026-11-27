import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RecordFormDialog } from "./RecordFormDialog";
import { configApi } from "@/lib/api";

vi.mock("@/lib/api", async () => {
  const actual = await vi.importActual<typeof import("@/lib/api")>("@/lib/api");
  return {
    ...actual,
    configApi: {
      ...actual.configApi,
      overview: vi.fn(),
    },
  };
});

describe("RecordFormDialog ISBN auto-fill", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.clearAllMocks();
  });

  it("fills empty fields from ISBN lookup without overwriting typed values", async () => {
    vi.mocked(configApi.overview).mockResolvedValue({
      library: { name: "Town Library", profile: "public", address: null, contact: null, currency: "INR", locale: "en-IN" },
      collections: [],
      locations: [],
      shelves: [],
      memberCategories: [],
      itemCategories: [],
      classificationSchemes: [{ id: 1, name: "Dewey Decimal", code: "DDC" }],
      policies: [],
    });

    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify({
      "ISBN:9780140328721": {
        title: "Matilda",
        authors: [{ name: "Roald Dahl" }],
        publishers: [{ name: "Puffin Books" }],
        publish_date: "1988",
        number_of_pages: 240,
        classifications: { dewey_decimal_class: ["823.914"] },
        subjects: [{ name: "Children's fiction" }],
      },
    }), { status: 200 })));

    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <QueryClientProvider client={qc}>
        <MemoryRouter>
          <RecordFormDialog onClose={vi.fn()} onSaved={vi.fn()} />
        </MemoryRouter>
      </QueryClientProvider>,
    );

    await userEvent.type(screen.getByLabelText(/^Publisher/i), "Custom Press");
    await userEvent.type(screen.getByLabelText(/^ISBN/i), "978-0-14-032872-1");
    await userEvent.click(screen.getByRole("button", { name: "Fill from ISBN" }));

    expect(await screen.findByRole("status")).toHaveTextContent(/Filled empty fields from OpenLibrary/i);
    expect(screen.getByLabelText(/^Title/i)).toHaveValue("Matilda");
    expect(screen.getByLabelText(/^Name 1/i)).toHaveValue("Roald Dahl");
    expect(screen.getByLabelText(/^Publisher/i)).toHaveValue("Custom Press");
    expect(screen.getByLabelText(/^Year/i)).toHaveValue("1988");
    expect(screen.getByLabelText(/^Classification number/i)).toHaveValue("823.914");
  });
});
