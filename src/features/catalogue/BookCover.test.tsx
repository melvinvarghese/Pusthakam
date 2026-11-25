import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BookCover } from "./BookCover";
import { catalogueApi } from "@/lib/api";
import { fetchIsbnCover } from "@/lib/images";

vi.mock("@/lib/api", () => ({ catalogueApi: { getCover: vi.fn(), setCover: vi.fn() }, toApiError: (error: Error) => ({ message: error.message }) }));
vi.mock("@/lib/images", () => ({ fetchIsbnCover: vi.fn(), prepareImage: vi.fn() }));
function mount({ editable = true, compact = false, isbn = "9780140328721" as string | null } = {}) {
  return render(<QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })}>
    <BookCover id={1} title="Matilda" isbn={isbn} editable={editable} compact={compact} />
  </QueryClientProvider>);
}
beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(catalogueApi.getCover).mockResolvedValue(null);
  vi.mocked(catalogueApi.setCover).mockResolvedValue(undefined);
  vi.mocked(fetchIsbnCover).mockResolvedValue("cover-png");
  vi.spyOn(navigator, "onLine", "get").mockReturnValue(true);
});
describe("BookCover", () => {
  it("shows a saved cover without contacting the ISBN provider", async () => {
    vi.mocked(catalogueApi.getCover).mockResolvedValue("saved-png");
    mount();
    expect(await screen.findByAltText("Cover of Matilda")).toHaveAttribute("src", "data:image/png;base64,saved-png");
    expect(fetchIsbnCover).not.toHaveBeenCalled();
  });
  it("automatically retrieves and saves an ISBN cover for offline reuse", async () => {
    vi.mocked(catalogueApi.setCover).mockImplementation(async () => { vi.mocked(catalogueApi.getCover).mockResolvedValue("cover-png"); });
    mount();
    expect(await screen.findByAltText("Cover of Matilda")).toHaveAttribute("src", "data:image/png;base64,cover-png");
    expect(fetchIsbnCover).toHaveBeenCalledWith("9780140328721");
    expect(catalogueApi.setCover).toHaveBeenCalledWith(1, "cover-png");
  });
  it("shows covers in compact catalogue rows without requiring write permission", async () => {
    mount({ editable: false, compact: true });
    expect(await screen.findByAltText("Cover of Matilda")).toBeInTheDocument();
    expect(catalogueApi.setCover).not.toHaveBeenCalled();
    expect(screen.queryByRole("heading", { name: "Book cover" })).not.toBeInTheDocument();
  });
  it("tries a direct ISBN image after download conversion fails and explains failure with retry", async () => {
    const user = userEvent.setup();
    vi.mocked(fetchIsbnCover).mockRejectedValueOnce(new Error("Cover download failed"));
    mount({ editable: false });
    const direct = await screen.findByAltText("Cover of Matilda");
    expect(direct).toHaveAttribute("src", "https://covers.openlibrary.org/b/isbn/9780140328721-M.jpg?default=false");
    fireEvent.error(direct);
    expect(await screen.findByText("No cover available")).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("Cover download failed");
    await user.click(screen.getByRole("button", { name: "Retry ISBN cover" }));
    await waitFor(() => expect(screen.getByAltText("Cover of Matilda")).toHaveAttribute("src", "data:image/png;base64,cover-png"));
  });
  it("shows a useful placeholder when no ISBN or saved cover exists", async () => {
    mount({ isbn: null });
    expect(await screen.findByText("No cover available")).toBeInTheDocument();
    expect(fetchIsbnCover).not.toHaveBeenCalled();
  });
});
