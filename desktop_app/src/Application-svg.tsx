import { Icons } from "./lib/icons";
export function getApplicationSVG({ tool }: { tool: string }) {
  switch (tool.toLowerCase()) {
    case "firefox":
      return <Icons.firefox />;
    case "gnome-terminal":
      return <Icons.terminal />;
    default:
      return <Icons.unknown />;
  }
}
export function getGeneralIconSVG({ tool }: { tool: string }) {
  switch (tool) {
    case "bell":
      return <Icons.bell />;
    case "check":
      return <Icons.check />;
    default:
      return <Icons.unknown />;
  }
}
