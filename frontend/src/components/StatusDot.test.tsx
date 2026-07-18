import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import StatusDot from "./StatusDot";

describe("StatusDot", () => {
  it("labels itself online when the agent is up", () => {
    render(<StatusDot online />);
    expect(screen.getByTitle("online")).toBeInTheDocument();
  });

  it("labels itself offline when the agent is down", () => {
    render(<StatusDot online={false} />);
    expect(screen.getByTitle("offline")).toBeInTheDocument();
  });
});
