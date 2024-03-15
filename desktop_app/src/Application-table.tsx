import { Row } from "./App";
import { getApplicationSVG } from "./Application-svg";
import { formatMinutes } from "./App";
import { Label } from "./components/ui/label";
function RowDescriptionForTable({ row }: { row: Row }) {
  return (
    <div class="flex items-center">
      {getApplicationSVG({ tool: row.application })}
      <div class="ml-4 space-y-1">
        <p class="text-lg font-medium leading-none">{row.application}</p>
      </div>
      <div class="ml-auto font-medium">{formatMinutes(row.duration)}</div>
    </div>
  );
}
function sortRows(rows: Row[]) {
  return rows.sort((a, b) => b.duration - a.duration);
}

function LabelForDay({ dayNumber }: { dayNumber: number }) {
  if (dayNumber < 0 || dayNumber > 6) {
    throw new Error("Invalid day number");
  }

  const dayArray = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
  ];
  return (
    <Label class="text-xl font-bold leading-none">{dayArray[dayNumber]}</Label>
  );
}
export function WeekApplicationTable({
  rows,
  dayNumber,
}: {
  rows: Row[];
  dayNumber: number;
}) {
  return (
    <div class="p-4">
      <LabelForDay dayNumber={dayNumber} />
      <ApplicationTable rows={rows} />
    </div>
  );
}
export function ApplicationTable({ rows }: { rows: Row[] }) {
  const sortedRows = sortRows(rows);
  return (
    <div class="space-y-8">
      {sortedRows.map((row) => (
        <RowDescriptionForTable row={row} />
      ))}
    </div>
  );
}
