import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import MetricTile from "./MetricTile";

describe("MetricTile", () => {
  it("renders the label and value", () => {
    render(<MetricTile label="CPU" value="12.3 %" />);
    expect(screen.getByText("CPU")).toBeInTheDocument();
    expect(screen.getByText("12.3 %")).toBeInTheDocument();
  });

  it("renders the hint when provided", () => {
    render(<MetricTile label="Memory" value="2.85 GB" hint="of 15.49 GB" />);
    expect(screen.getByText("of 15.49 GB")).toBeInTheDocument();
  });

  it("omits the hint when not provided", () => {
    render(<MetricTile label="Load (1m)" value="0.42" />);
    expect(screen.queryByText(/of /)).not.toBeInTheDocument();
  });
});
