import { ColorType } from "lightweight-charts";
export interface ChartTheme {
  layout: {
    textColor: string;
    background: {
      type: ColorType;
      color: string;
    };
  };
  grid: {
    vertLines: {
      color: string;
    };
    horzLines: {
      color: string;
    };
  };
}

export const darkChartTheme = {
  layout: {
    textColor: "white",
    background: { type: ColorType.Solid, color: "transparent" },
  },
  grid: {
    vertLines: { color: "#484848" },
    horzLines: { color: "#484848" },
  },
};

export const lightChartTheme = {
  layout: {
    textColor: "black",
    background: { type: ColorType.Solid, color: "transparent" },
  },
  grid: {
    vertLines: { color: "#ECECEC" },
    horzLines: { color: "#ECECEC" },
  },
};
