import { useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/Button";
import { Modal } from "@/components/ui/Modal";
import { configApi } from "@/lib/api";
import type { LoanDetails } from "@/lib/api/types";

export function DueSlip({ loan, renewed, reprint = false, onClose }: { loan: LoanDetails; renewed: boolean; reprint?: boolean; onClose: () => void }) {
  const { data } = useQuery({ queryKey: ["config"], queryFn: configApi.overview, retry: false });
  return (
    <Modal title="Loan due slip" onClose={onClose} footer={<>
      <Button onClick={onClose}>Close</Button>
      <Button variant="primary" onClick={() => window.print()}>Print slip</Button>
    </>}>
      <section className="print-area w-[80mm] max-w-full space-y-2 bg-white p-3 text-black">
        <header className="border-b border-black/20 pb-2">
          <p className="text-sm font-semibold">{data?.library?.name ?? "Library"}</p>
          <p className="text-xs">{reprint ? "Current loan" : renewed ? "Loan renewal" : "Loan issue"}</p>
        </header>
        <dl className="space-y-2 text-sm">
          <div><dt className="text-xs">Borrower</dt><dd>{loan.memberName}</dd></div>
          <div><dt className="text-xs">Title</dt><dd>{loan.title}</dd></div>
          <div><dt className="text-xs">Due date</dt><dd className="font-mono font-semibold">{loan.dueOn}</dd></div>
        </dl>
      </section>
    </Modal>
  );
}
