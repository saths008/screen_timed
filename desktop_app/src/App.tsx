import { createSignal, createEffect, Accessor, onMount } from "solid-js";
import { Input } from "./components/ui/input";
import { As } from "@kobalte/core";

import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "./components/ui/alert-dialog";
import { invoke } from "@tauri-apps/api/tauri";
import DatePicker, { PickerValue } from "@rnwonder/solid-date-picker";
import utils from "@rnwonder/solid-date-picker/utilities";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./components/ui/tabs";
import { Grid } from "./components/ui/grid";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardFooter,
} from "./components/ui/card";
import { BarChart, LineChart } from "./components/ui/charts";
import { WeekApplicationTable, ApplicationTable } from "./Application-table";
import { Button } from "./components/ui/button";
import { getGeneralIconSVG } from "./Application-svg";
export interface Row {
  timestamp: number;
  application: string;
  duration: number;
}
interface Chart {
  labels: string[];
  datasets: {
    label: string;
    data: number[];
  }[];
}

function formatDate(date: Date | undefined) {
  if (date === undefined) return "Loading...";
  const months = [
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec",
  ];
  const month: string = months[date.getMonth()];
  const day: number = date.getDate();
  const year: number = date.getFullYear();
  return `${month} ${day}, ${year}`;
}
function pickerValueToDate(pickerValue: PickerValue) {
  const date = pickerValue.value.selectedDateObject;
  if (date === undefined) return;
  const dateToFormat = utils().convertDateObjectToDate(date);
  return dateToFormat;
}
const DatePickerInput = (props: {
  value: Accessor<PickerValue>;
  showDate: () => void;
}) => {
  const todayFormattedDate = formatDate(new Date());
  return (
    <input
      readOnly
      onClick={props.showDate}
      placeholder={todayFormattedDate}
      value={props.value().label}
      style={{
        "box-shadow": "0 1.5rem 2rem rgba(156, 136, 255, 0.2)",
      }}
      class="text-black text-center rounded border-solid p-2"
    />
  );
};
function getDayOfWeek(date: Date | undefined) {
  if (date === undefined) return "Something went wrong!";
  const days = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
  ];
  return days[date.getDay()];
}
export function formatMinutes(seconds: number) {
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  if (hours == 0) {
    return `${remainingMinutes}m`;
  }
  return `${hours}h ${remainingMinutes}m`;
}

function getPreviousSunday() {
  const today = new Date();
  const dayOfWeek = today.getDay();
  const prevSunday = new Date(today);
  prevSunday.setDate(today.getDate() - dayOfWeek);
  prevSunday.setHours(0, 0, 0, 0);

  return prevSunday;
}
function MostUsedWeekApp({ rows }: { rows: Row[][] }) {
  let mostUsedApp = "";
  let mostUsedAppDuration = Number.MIN_VALUE;
  const flattened_rows = rows.flat();
  for (let r of flattened_rows) {
    if (r.duration > mostUsedAppDuration) {
      mostUsedApp = r.application;
      mostUsedAppDuration = r.duration;
    }
  }
  return (
    <div>
      <div class="text-2xl font-bold">{mostUsedApp}</div>
      <p class="text-xs text-muted-foreground">
        {formatMinutes(mostUsedAppDuration)}
      </p>
    </div>
  );
}
function LeastUsedWeekApp({ rows }: { rows: Row[][] }) {
  let leastUsedApp = "";
  let leastUsedAppDuration = Number.MAX_VALUE;
  const flattened_rows = rows.flat();
  for (let r of flattened_rows) {
    if (r.duration < leastUsedAppDuration) {
      leastUsedApp = r.application;
      leastUsedAppDuration = r.duration;
    }
  }
  return (
    <div>
      <div class="text-2xl font-bold">{leastUsedApp}</div>
      <p class="text-xs text-muted-foreground">
        {formatMinutes(leastUsedAppDuration)}
      </p>
    </div>
  );
}

function App() {
  const [deleteMonths, setDeleteMonths] = createSignal<number>(1);
  const [deleteConfirm, setDeleteConfirm] = createSignal<boolean>(false);
  const [chartSet, setChartSet] = createSignal<boolean>(false);
  const [records, setRecords] = createSignal<Row[]>([]);
  //An array of size 7, first element is Sunday
  const [weekRecords, setWeekRecords] = createSignal<Row[][]>([[]]);
  const [alertScreenTime, setAlertScreenTime] = createSignal<number>(0);
  const today = new Date();
  const todayDateObject = {
    year: today.getFullYear(),
    month: today.getMonth(),
    day: today.getDate(),
  };
  const [weekDate, setWeekDate] = createSignal<Date>();

  onMount(() => {
    const prevSunday = getPreviousSunday();
    setWeekDate(prevSunday);
  });

  function addWeek() {
    const currentWeekDate = weekDate();
    if (currentWeekDate === undefined) {
      return;
    } else {
      const nextWeekDate = new Date(currentWeekDate.getTime());
      nextWeekDate.setDate(nextWeekDate.getDate() + 7);
      setWeekDate(nextWeekDate);
    }
  }
  function subtractWeek() {
    const currentWeekDate = weekDate();
    if (currentWeekDate === undefined) {
      return;
    } else {
      const prevWeekDate = new Date(currentWeekDate.getTime());
      prevWeekDate.setDate(prevWeekDate.getDate() - 7);
      setWeekDate(prevWeekDate);
    }
  }
  function WeekSlider() {
    return (
      <div class="flex items-center space-x-2">
        <Button
          class="flex-none bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-l"
          onClick={() => subtractWeek()}
        >
          &larr;
        </Button>
        <Button class="flex-auto bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4">
          Week Beginning: {formatDate(weekDate())}
        </Button>
        <Button
          class="flex-none bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-r"
          onClick={() => addWeek()}
        >
          &rarr;
        </Button>
      </div>
    );
  }
  const [date, setDate] = createSignal<PickerValue>({
    value: {
      selectedDateObject: todayDateObject,
    },
    label: "",
  });

  const minimumDate = { year: 2024, month: 1, day: 24 };
  const chartData: Chart = {
    labels: [],
    datasets: [
      {
        label: "Today's Screen Time",
        data: [],
      },
    ],
  };
  const weekChartData: Chart = {
    labels: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
    datasets: [
      {
        label: "Week Screen Time",
        data: [],
      },
    ],
  };
  function setWeekChartData() {
    weekChartData.datasets[0].data = [];

    for (let r of weekRecords()) {
      let total = 0;
      for (let record of r) {
        total += record.duration;
      }
      total = total / 60;
      weekChartData.datasets[0].data.push(total);
    }
  }

  function setChartData() {
    const sortedRecords = records().sort((row1, row2) => {
      const app1 = row1.application.toLowerCase();
      const app2 = row2.application.toLowerCase();

      if (app1 < app2) {
        return -1;
      }
      if (app1 > app2) {
        return 1;
      }
      return 0;
    });
    chartData.labels = [];
    chartData.datasets[0].data = [];

    for (let r of sortedRecords) {
      chartData.labels.push(r.application);
      chartData.datasets[0].data.push(r.duration / 60);
    }
  }
  function resetChartSet() {
    setChartSet(false);
    setChartSet(true);
  }
  const [deleteError, setDeleteError] = createSignal<string>();
  async function deleteMonthsData() {
    try {
      await invoke("send_delete_months_data_message", {
        months: deleteMonths(),
      });
    } catch (e) {
      setDeleteError(JSON.stringify(e));
      console.log("Error fetching screen time");
      console.log(e);
    }
  }

  async function getWeekScreenTime() {
    const start_of_week = weekDate();
    if (start_of_week === undefined) return;
    start_of_week.setHours(0, 0, 0, 0);
    const start_of_date_secs = Math.floor(start_of_week.getTime() / 1000);
    try {
      setWeekRecords(
        await invoke("get_week_screen_time", {
          start_of_date: start_of_date_secs,
        }),
      );
      setWeekChartData();
      resetChartSet();
    } catch (e) {
      console.log("Error fetching screen time");
      console.log(e);
    }
  }
  async function getAlertScreenTime() {
    try {
      setAlertScreenTime(await invoke("send_get_alert_screen_time_message"));
    } catch (e) {
      console.log("Error fetching screen time");
      console.log(e);
    }
  }

  async function getDateScreenTime() {
    const start_of_date = pickerValueToDate(date());
    if (start_of_date === undefined) return;
    start_of_date.setHours(0, 0, 0, 0);
    const start_of_date_secs = Math.floor(start_of_date.getTime() / 1000);

    try {
      setRecords(
        await invoke("get_date_screen_time", {
          start_of_date: start_of_date_secs,
        }),
      );
      setChartData();
      resetChartSet();
    } catch (e) {
      console.log("Error fetching screen time");
      console.log(e);
    }
  }
  const handleChange = (e: any) => {
    const input = e.target.value;
    setDeleteMonths(input);
  };
  const [updated, setUpdated] = createSignal(false);
  async function updateScreenTime() {
    try {
      await invoke("send_update_socket_message", {});
    } catch (e) {
      console.log("Error updating screen time");
      console.log(e);
    }
  }
  createEffect(async () => {
    if (deleteConfirm()) {
      await deleteMonthsData();
      setDeleteConfirm(false);
    }
  });

  createEffect(async () => {
    if (updated()) {
      await getWeekScreenTime();
    }
  });
  createEffect(async () => {
    if (updated()) {
      await getDateScreenTime();
    }
  });
  createEffect(async () => {
    await getAlertScreenTime();
  });

  onMount(async () => {
    await updateScreenTime();
    setUpdated(true);
  });
  return (
    <Tabs defaultValue="day" class="space-y-4">
      <TabsList class="grid w-full grid-cols-3 ">
        <TabsTrigger value="day">Day</TabsTrigger>
        <TabsTrigger value="week">Week</TabsTrigger>
        <TabsTrigger value="other">Other</TabsTrigger>
      </TabsList>

      <TabsContent value="day" class="space-y-4">
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <Card>
            <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle class="text-sm font-medium">
                Total Screen Time
              </CardTitle>
              <svg
                width="20px"
                height="20px"
                viewBox="0 0 0.75 0.75"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M0.375 0.375H0.35a0.025 0.025 0 0 0 0.007 0.018zm0 0.325A0.325 0.325 0 0 1 0.05 0.375H0A0.375 0.375 0 0 0 0.375 0.75zM0.7 0.375A0.325 0.325 0 0 1 0.375 0.7v0.05A0.375 0.375 0 0 0 0.75 0.375zM0.375 0.05A0.325 0.325 0 0 1 0.7 0.375h0.05A0.375 0.375 0 0 0 0.375 0zm0 -0.05A0.375 0.375 0 0 0 0 0.375h0.05A0.325 0.325 0 0 1 0.375 0.05zM0.35 0.15v0.225h0.05V0.15zm0.007 0.243 0.15 0.15 0.035 -0.035 -0.15 -0.15z"
                  fill="#FFFFFF"
                />
              </svg>
            </CardHeader>
            <CardContent>
              <div class="text-2xl font-bold">
                {formatMinutes(
                  records()
                    .map((r) => r.duration)
                    .reduce((partialSum, a) => partialSum + a, 0),
                )}
              </div>
            </CardContent>
          </Card>
        </div>
        <Grid colsMd={2} colsLg={7} class="gap-4">
          <Card class="col-span-4">
            <CardHeader>
              <CardTitle>{getDayOfWeek(pickerValueToDate(date()))}</CardTitle>
              <DatePicker
                value={date}
                setValue={setDate}
                maxDate={utils().getToday()}
                renderInput={({ value, showDate }) => (
                  <DatePickerInput showDate={showDate} value={value} />
                )}
                weekDaysType={"single"}
                inputWrapperWidth={"fit-content"}
              />
            </CardHeader>
            <CardContent class="pl-2">
              {chartSet() && <BarChart data={chartData} />}
            </CardContent>
          </Card>
          <Card class="col-span-3">
            <CardHeader>
              <CardTitle>Most Used</CardTitle>
            </CardHeader>
            <CardContent>
              {chartSet() && <ApplicationTable rows={records()} />}
            </CardContent>
          </Card>
        </Grid>
      </TabsContent>
      <TabsContent value="week" class="space-y-4">
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <Card>
            <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle class="text-sm font-medium">
                Total Screen Time
              </CardTitle>
              <svg
                width="20px"
                height="20px"
                viewBox="0 0 0.75 0.75"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M0.375 0.375H0.35a0.025 0.025 0 0 0 0.007 0.018zm0 0.325A0.325 0.325 0 0 1 0.05 0.375H0A0.375 0.375 0 0 0 0.375 0.75zM0.7 0.375A0.325 0.325 0 0 1 0.375 0.7v0.05A0.375 0.375 0 0 0 0.75 0.375zM0.375 0.05A0.325 0.325 0 0 1 0.7 0.375h0.05A0.375 0.375 0 0 0 0.375 0zm0 -0.05A0.375 0.375 0 0 0 0 0.375h0.05A0.325 0.325 0 0 1 0.375 0.05zM0.35 0.15v0.225h0.05V0.15zm0.007 0.243 0.15 0.15 0.035 -0.035 -0.15 -0.15z"
                  fill="#FFFFFF"
                />
              </svg>
            </CardHeader>
            <CardContent>
              <div class="text-2xl font-bold">
                {formatMinutes(
                  weekRecords()
                    .flat()
                    .map((r) => r.duration)
                    .reduce((partialSum, a) => partialSum + a, 0),
                )}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle class="text-sm font-medium">
                Average Time Per Day
              </CardTitle>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                class="size-4 text-muted-foreground"
              >
                <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
                <circle cx="9" cy="7" r="4" />
                <path d="M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75" />
              </svg>
            </CardHeader>
            <CardContent>
              <div class="text-2xl font-bold">
                {formatMinutes(
                  weekRecords()
                    .flat()
                    .map((r) => r.duration)
                    .reduce((partialSum, a) => partialSum + a, 0) / 7,
                )}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle class="text-sm font-medium">
                Most Used Weekly App
              </CardTitle>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                class="size-4 text-muted-foreground"
              >
                <rect width="20" height="14" x="2" y="5" rx="2" />{" "}
                <path d="M2 10h20" />{" "}
              </svg>
            </CardHeader>
            <CardContent>
              {chartSet() && <MostUsedWeekApp rows={weekRecords()} />}
            </CardContent>
          </Card>
          <Card>
            <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle class="text-sm font-medium">
                Least Used Weekly App{" "}
              </CardTitle>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                class="size-4 text-muted-foreground"
              >
                <path d="M22 12h-4l-3 9L9 3l-3 9H2" />
              </svg>
            </CardHeader>
            <CardContent>
              {chartSet() && <LeastUsedWeekApp rows={weekRecords()} />}
            </CardContent>
          </Card>
        </div>
        <Grid colsMd={2} colsLg={7} class="gap-4">
          <Card class="col-span-4">
            <CardHeader>
              <CardTitle>{getDayOfWeek(pickerValueToDate(date()))}</CardTitle>
              <WeekSlider />
            </CardHeader>
            <CardContent class="pl-2">
              {chartSet() && <LineChart data={weekChartData} />}
            </CardContent>
          </Card>
          <Card class="col-span-3">
            <CardHeader></CardHeader>
            <CardContent>
              {chartSet() &&
                weekRecords().map((dayRows, index) => (
                  <WeekApplicationTable rows={dayRows} dayNumber={index} />
                ))}
            </CardContent>
          </Card>
        </Grid>
      </TabsContent>
      <TabsContent value="other" class="space-y-4">
        <div class="grid w-full grid-cols-2 ">
          <Card class="w-[380px]">
            <CardHeader>
              <CardTitle>Notifications</CardTitle>
            </CardHeader>
            <CardContent class="grid gap-4">
              <div class=" flex items-center space-x-4 rounded-md border p-4">
                {getGeneralIconSVG({ tool: "bell" })}
                <div class="flex-1 space-y-1">
                  <p class="text-sm font-medium leading-none">
                    Current Screen Time Alert: {alertScreenTime()}
                  </p>
                  <p class="text-sm text-muted-foreground"></p>
                </div>
              </div>
            </CardContent>
            <CardFooter></CardFooter>
          </Card>
          <Card class="w-[380px]">
            <Input
              type="number"
              min={1}
              value={deleteMonths()}
              onInput={handleChange}
            />
            <AlertDialog>
              <AlertDialogTrigger asChild>
                <As component={Button} variant="destructive">
                  <div class="bg-red-500">Delete {deleteMonths()} months</div>
                </As>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogTitle>Are you sure?</AlertDialogTitle>
                <AlertDialogDescription>
                  You are about to delete the oldest {deleteMonths()} months of
                  data. This action cannot be undone. Are you sure you want to
                  proceed?
                </AlertDialogDescription>
                <Button
                  onClick={() => setDeleteConfirm(true)}
                  class="bg-red-500"
                >
                  Confirm
                </Button>
                {deleteConfirm() && <p>Delete Request Sent.</p>}
              </AlertDialogContent>
            </AlertDialog>
            {deleteError() && <p>{deleteError()}</p>}
          </Card>
        </div>
      </TabsContent>
    </Tabs>
  );
}

export default App;
