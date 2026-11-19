import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import AcquisitionsPage from "./AcquisitionsPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(invoke).mockReset();
});

function renderWithClient(permissions = ["acquisitions.read", "acquisitions.write"]) {
  return render(
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
        <AcquisitionsPage />
      </SessionContext.Provider>
    </QueryClientProvider>
  );
}

it("renders requests and handles request creation and approval", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockRequests = [
    {
      id: 1,
      title: "Clean Code",
      author: "Robert C. Martin",
      isbn: "9780132350884",
      publisher: "Prentice Hall",
      publicationYear: 2008,
      estimatedPriceMinor: 250000,
      currency: "INR",
      copies: 2,
      requestedBy: "CS Department",
      requestReason: "Recommended reading",
      status: "pending",
      rejectionReason: null,
      budgetId: null,
      budgetName: null,
      createdAt: "2026-11-19T10:00:00Z",
    },
  ];

  const mockBudgets = [
    {
      id: 10,
      name: "Engineering 2026-27",
      code: "ENG-26",
      amountMinor: 10000000,
      currency: "INR",
      fiscalYear: "2026-27",
      allocatedMinor: 2000000,
      archived: false,
      createdAt: "2026-11-19T09:00:00Z",
    },
  ];

  vi.mocked(invoke).mockImplementation(async (command, _args) => {
    if (command === "config_overview") {
      return { library: { currency: "INR" } };
    }
    if (command === "acquisitions_list_requests") {
      return mockRequests;
    }
    if (command === "acquisitions_list_budgets") {
      return mockBudgets;
    }
    if (command === "acquisitions_create_request") {
      return 2;
    }
    if (command === "acquisitions_approve_request") {
      return undefined;
    }
    throw new Error(`Unexpected command: ${command}`);
  });

  renderWithClient();

  expect(await screen.findByText("Clean Code")).toBeInTheDocument();
  expect(screen.getByText("Robert C. Martin")).toBeInTheDocument();
  expect(screen.getByText("CS Department")).toBeInTheDocument();
  expect(screen.getByText("Pending")).toBeInTheDocument();

  // Test Approve Request
  const approveBtn = screen.getByRole("button", { name: "Approve" });
  await user.click(approveBtn);

  expect(await screen.findByText("Approve Request: Clean Code")).toBeInTheDocument();
  const submitApprove = screen.getByRole("button", { name: "Approve Request" });
  await user.click(submitApprove);

  expect(invoke).toHaveBeenCalledWith("acquisitions_approve_request", {
    id: 1,
    budgetId: null,
  });
});

it("manages vendors tab lifecycle and vendor modal", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockVendors = [
    {
      id: 5,
      name: "Prabhat Books",
      code: "PRABHAT",
      contactName: "Raman Nair",
      email: "raman@prabhat.example",
      phone: "+91 471 2334455",
      address: "Trivandrum",
      notes: "Preferred supplier",
      archived: false,
      createdAt: "2026-11-19T10:00:00Z",
    },
  ];

  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === "config_overview") return { library: { currency: "INR" } };
    if (command === "acquisitions_list_requests") return [];
    if (command === "acquisitions_list_vendors") return mockVendors;
    if (command === "acquisitions_save_vendor") return 6;
    if (command === "acquisitions_archive_vendor") return undefined;
    throw new Error(`Unexpected command: ${command}`);
  });

  renderWithClient();

  // Switch to Vendors tab
  const vendorsTab = screen.getByRole("tab", { name: "Vendors" });
  await user.click(vendorsTab);

  expect(await screen.findByText("Prabhat Books")).toBeInTheDocument();
  expect(screen.getByText("Raman Nair")).toBeInTheDocument();
  expect(screen.getByText("Active")).toBeInTheDocument();

  // Open Add Vendor modal
  const addVendorBtn = screen.getByRole("button", { name: "Add Vendor" });
  await user.click(addVendorBtn);

  expect(await screen.findByRole("dialog")).toBeInTheDocument();
  const nameInput = screen.getByLabelText("Vendor Name");
  await user.type(nameInput, "DC Books");

  const saveBtn = screen.getByRole("button", { name: "Save Vendor" });
  await user.click(saveBtn);

  expect(invoke).toHaveBeenCalledWith("acquisitions_save_vendor", {
    input: {
      id: undefined,
      name: "DC Books",
      code: null,
      contactName: null,
      email: null,
      phone: null,
      address: null,
      notes: null,
    },
  });
});

it("manages purchase orders tab and viewing details", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockOrders = [
    {
      id: 101,
      orderNumber: "PO-2026-0001",
      vendorId: 5,
      vendorName: "Prabhat Books",
      budgetId: 10,
      budgetName: "Engineering 2026-27",
      status: "draft",
      invoiceRef: null,
      orderedAt: null,
      itemsCount: 2,
      totalMinor: 48000,
      createdAt: "2026-11-19T11:00:00Z",
    },
  ];

  const mockDetail = {
    id: 101,
    orderNumber: "PO-2026-0001",
    vendorId: 5,
    vendorName: "Prabhat Books",
    budgetId: 10,
    budgetName: "Engineering 2026-27",
    status: "draft",
    invoiceRef: null,
    orderedAt: null,
    expectedAt: "2026-12-15",
    notes: "Direct delivery",
    items: [
      {
        id: 1,
        orderId: 101,
        requestId: null,
        title: "Clean Code",
        author: "Robert C. Martin",
        isbn: "9780132350884",
        quantity: 2,
        unitPriceMinor: 24000,
        receivedQuantity: 0,
        notes: null,
      },
    ],
    totalMinor: 48000,
    createdAt: "2026-11-19T11:00:00Z",
  };

  vi.mocked(invoke).mockImplementation(async (command, _args: unknown) => {
    if (command === "config_overview") return { library: { currency: "INR" } };
    if (command === "acquisitions_list_requests") return [];
    if (command === "acquisitions_list_orders") return mockOrders;
    if (command === "acquisitions_get_order") return mockDetail;
    if (command === "acquisitions_update_order_status") return undefined;
    throw new Error(`Unexpected command: ${command}`);
  });

  renderWithClient();

  // Switch to Purchase Orders tab
  const ordersTab = screen.getByRole("tab", { name: "Purchase Orders" });
  await user.click(ordersTab);

  expect(await screen.findByText("PO-2026-0001")).toBeInTheDocument();
  expect(screen.getByText("Prabhat Books")).toBeInTheDocument();
  expect(screen.getByText("2 items")).toBeInTheDocument();

  // Click View Details
  const viewDetailsBtn = screen.getByRole("button", { name: "View Details" });
  await user.click(viewDetailsBtn);

  expect(await screen.findByRole("dialog")).toBeInTheDocument();
  expect(screen.getByText("Line Items")).toBeInTheDocument();
  expect(screen.getByText("Place Order")).toBeInTheDocument();

  // Click Place Order
  const placeOrderBtn = screen.getByRole("button", { name: "Place Order" });
  await user.click(placeOrderBtn);

  expect(invoke).toHaveBeenCalledWith("acquisitions_update_order_status", {
    id: 101,
    status: "ordered",
    invoiceRef: null,
  });
});

it("receives stock against an ordered purchase order", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const user = userEvent.setup();

  const mockOrders = [
    {
      id: 102,
      orderNumber: "PO-2026-0002",
      vendorId: 5,
      vendorName: "DC Books",
      budgetId: 10,
      budgetName: "Engineering 2026-27",
      status: "ordered",
      invoiceRef: "INV-555",
      orderedAt: "2026-11-19T10:00:00Z",
      itemsCount: 1,
      totalMinor: 48000,
      createdAt: "2026-11-19T10:00:00Z",
    },
  ];

  const mockDetail = {
    id: 102,
    orderNumber: "PO-2026-0002",
    vendorId: 5,
    vendorName: "DC Books",
    budgetId: 10,
    budgetName: "Engineering 2026-27",
    status: "ordered",
    invoiceRef: "INV-555",
    orderedAt: "2026-11-19T10:00:00Z",
    expectedAt: "2026-12-15",
    notes: null,
    items: [
      {
        id: 201,
        orderId: 102,
        requestId: null,
        title: "Clean Architecture",
        author: "Robert C. Martin",
        isbn: "9780134494166",
        quantity: 2,
        unitPriceMinor: 24000,
        receivedQuantity: 0,
        notes: null,
      },
    ],
    totalMinor: 48000,
    createdAt: "2026-11-19T10:00:00Z",
  };

  vi.mocked(invoke).mockImplementation(async (command, _args: unknown) => {
    if (command === "config_overview") {
      return {
        library: { currency: "INR" },
        itemCategories: [{ id: 1, name: "Book", archived: false }],
        locations: [{ id: 1, name: "Main", archived: false }],
      };
    }
    if (command === "acquisitions_list_requests") return [];
    if (command === "acquisitions_list_orders") return mockOrders;
    if (command === "acquisitions_get_order") return mockDetail;
    if (command === "acquisitions_receive_items") {
      return {
        orderId: 102,
        itemId: 201,
        recordId: 50,
        receivedQuantity: 2,
        orderStatus: "received",
        createdCopies: [
          { copyId: 1, accessionNo: "000001", barcode: "000001" },
          { copyId: 2, accessionNo: "000002", barcode: "000002" },
        ],
      };
    }
    throw new Error(`Unexpected command: ${command}`);
  });

  renderWithClient();

  // Switch to Purchase Orders tab
  const ordersTab = screen.getByRole("tab", { name: "Purchase Orders" });
  await user.click(ordersTab);

  expect(await screen.findByText("PO-2026-0002")).toBeInTheDocument();

  // Open order detail
  await user.click(screen.getByRole("button", { name: "View Details" }));
  expect(await screen.findByText("Clean Architecture")).toBeInTheDocument();

  // Click Receive Stock
  const receiveBtn = screen.getByRole("button", { name: "Receive Stock" });
  await user.click(receiveBtn);

  expect(await screen.findByText("Receive Stock: Clean Architecture")).toBeInTheDocument();
  const confirmBtn = screen.getByRole("button", { name: "Confirm Receipt" });
  await user.click(confirmBtn);

  expect(invoke).toHaveBeenCalledWith("acquisitions_receive_items", {
    input: {
      orderId: 102,
      itemId: 201,
      quantity: 2,
      itemCategoryId: null,
      locationId: null,
    },
  });

  expect(await screen.findByText("Successfully received 2 copies!")).toBeInTheDocument();
  expect(screen.getByText("000001")).toBeInTheDocument();
  expect(screen.getByText("000002")).toBeInTheDocument();
});
