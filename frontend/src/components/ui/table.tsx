import * as React from "react";
import type { PaginationMetaData } from "@/api/endpointGenerator";
import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination";
import { cn } from "@/lib/utils";

function Table({ className, ...props }: React.ComponentProps<"table">) {
  return (
    <div
      data-slot="table-container"
      className="relative w-full overflow-x-auto"
    >
      <table
        data-slot="table"
        className={cn("w-full caption-bottom text-sm border-separate border-spacing-0", className)}
        {...props}
      />
    </div>
  );
}

function TableHeader({ className, ...props }: React.ComponentProps<"thead">) {
  return (
    <thead
      data-slot="table-header"
      className={cn("[&_tr]:border-b", className)}
      {...props}
    />
  );
}

function TableBody({ className, ...props }: React.ComponentProps<"tbody">) {
  return (
    <tbody
      data-slot="table-body"
      className={cn("[&_tr:last-child]:border-0", className)}
      {...props}
    />
  );
}

function TableFooter({ className, ...props }: React.ComponentProps<"tfoot">) {
  return (
    <tfoot
      data-slot="table-footer"
      className={cn(
        "bg-muted/50 border-t font-medium [&>tr]:last:border-b-0",
        className,
      )}
      {...props}
    />
  );
}

function TableRow({ className, ...props }: React.ComponentProps<"tr">) {
  return (
    <tr
      data-slot="table-row"
      className={cn(
        "hover:bg-muted/50 data-[state=selected]:bg-muted border-b transition-colors",
        className,
      )}
      {...props}
    />
  );
}

function TableHead({ className, ...props }: React.ComponentProps<"th">) {
  return (
    <th
      data-slot="table-head"
      className={cn(
        "text-foreground h-10 px-2 text-left align-middle font-medium whitespace-nowrap [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]",
        className,
      )}
      {...props}
    />
  );
}

function TableCell({ className, ...props }: React.ComponentProps<"td">) {
  return (
    <td
      data-slot="table-cell"
      className={cn(
        "p-2 align-middle whitespace-nowrap [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]",
        className,
      )}
      {...props}
    />
  );
}

function TableCaption({
  className,
  ...props
}: React.ComponentProps<"caption">) {
  return (
    <caption
      data-slot="table-caption"
      className={cn("text-muted-foreground mt-4 text-sm", className)}
      {...props}
    />
  );
}

function TablePagination({
  currentPage,
  setCurrentPage,
  paginationData,
}: {
  setCurrentPage: (currentPageToSet: number) => void;
  currentPage: number;
  paginationData: PaginationMetaData;
}) {
  const MiddleItems = () => {
    if (paginationData.total_pages <= 2) {
      return null;
    }

    const middlePage =
      currentPage === 1
        ? 2
        : currentPage === paginationData.total_pages
          ? currentPage - 1
          : currentPage;

    return (
      <PaginationItem>
        <PaginationLink
          isActive={currentPage === middlePage}
          onClick={() => setCurrentPage(middlePage)}
        >
          {middlePage}
        </PaginationLink>
      </PaginationItem>
    );
  };

  return (
    <Pagination>
      <PaginationContent>
        {currentPage > 1 && (
          <PaginationItem>
            <PaginationPrevious
              onClick={() => setCurrentPage(currentPage - 1)}
            />
          </PaginationItem>
        )}
        <PaginationItem>
          <PaginationLink
            onClick={() => setCurrentPage(1)}
            isActive={currentPage === 1}
          >
            1
          </PaginationLink>
        </PaginationItem>
        {paginationData.total_pages > 3 && ![1, 2].includes(currentPage) && (
          <PaginationItem>
            <PaginationEllipsis />
          </PaginationItem>
        )}
        <MiddleItems />
        {paginationData.total_pages > 3 &&
          ![
            paginationData.total_pages,
            paginationData.total_pages - 1,
          ].includes(currentPage) && (
            <PaginationItem>
              <PaginationEllipsis />
            </PaginationItem>
          )}
        {paginationData.total_pages !== 1 && (
          <PaginationItem>
            <PaginationLink
              isActive={currentPage === paginationData.total_pages}
              onClick={() => setCurrentPage(paginationData.total_pages)}
            >
              {paginationData.total_pages}
            </PaginationLink>
          </PaginationItem>
        )}
        {paginationData?.has_more && (
          <PaginationItem>
            <PaginationNext onClick={() => setCurrentPage(currentPage + 1)} />
          </PaginationItem>
        )}
      </PaginationContent>
    </Pagination>
  );
}

export {
  Table,
  TableHeader,
  TableBody,
  TableFooter,
  TableHead,
  TableRow,
  TableCell,
  TableCaption,
  TablePagination,
};
